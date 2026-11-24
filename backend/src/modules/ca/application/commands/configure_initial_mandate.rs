//! Caso de uso: Configuração inicial do mandato da gestão do Centro Acadêmico (Issue #296).
//!
//! Permite que a gestão autorizada defina o período de atuação do mandato inicial,
//! validando a existência e atividade do CA, autorização do usuário, consistência
//! temporal das datas e ausência de mandato ativo preexistente.

use chrono::NaiveDate;
use uuid::Uuid;

use crate::modules::ca::{
    application::{
        error::MandateApplicationError,
        ports::{AcademicCenterAuthorizationPort, AcademicCenterRepository, MandateRepository},
    },
    domain::entities::{AcademicCenterStatus, Mandate},
};

/// Dados para execução do comando de configuração inicial do mandato.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigureInitialMandateCommand {
    /// Identificador do Centro Acadêmico.
    pub academic_center_id: Uuid,
    /// Identificador do usuário que está solicitando a configuração.
    pub actor_user_id: Uuid,
    /// Título opcional da chapa ou gestão (ex.: "Gestão 2026-2027").
    pub title: Option<String>,
    /// Data de início do mandato (AAAA-MM-DD).
    pub start_date: NaiveDate,
    /// Data de término do mandato (AAAA-MM-DD), estritamente posterior ao início.
    pub end_date: NaiveDate,
}

/// Executa a configuração inicial do mandato da gestão.
///
/// # Regras de Negócio e Invariantes
///
/// 1. O Centro Acadêmico deve existir; caso contrário retorna [`MandateApplicationError::AcademicCenterNotFound`].
/// 2. O Centro Acadêmico deve estar ativo; se arquivado, retorna [`MandateApplicationError::AcademicCenterInactive`].
/// 3. Apenas usuário com permissão de gestão no CA pode configurar o mandato; caso contrário retorna [`MandateApplicationError::UserNotPermittedInCa`].
/// 4. As datas devem ser cronologicamente consistentes (`end_date > start_date`); caso contrário retorna [`MandateApplicationError::InvalidMandateDates`].
/// 5. O CA não pode possuir outro mandato ativo; caso contrário retorna [`MandateApplicationError::ActiveMandateAlreadyExists`].
pub async fn configure_initial_mandate(
    ca_repo: &impl AcademicCenterRepository,
    mandate_repo: &impl MandateRepository,
    auth_port: &impl AcademicCenterAuthorizationPort,
    command: ConfigureInitialMandateCommand,
) -> Result<Mandate, MandateApplicationError> {
    let ca = ca_repo
        .find_by_id(command.academic_center_id)
        .await?
        .ok_or(MandateApplicationError::AcademicCenterNotFound(
            command.academic_center_id,
        ))?;

    if ca.status() != AcademicCenterStatus::Active {
        return Err(MandateApplicationError::AcademicCenterInactive(
            command.academic_center_id,
        ));
    }

    let is_permitted = auth_port
        .is_user_permitted(command.actor_user_id, &ca)
        .await?;

    if !is_permitted {
        return Err(MandateApplicationError::UserNotPermittedInCa {
            user_id: command.actor_user_id,
            academic_center_id: command.academic_center_id,
        });
    }

    if command.end_date <= command.start_date {
        return Err(MandateApplicationError::InvalidMandateDates(format!(
            "a data de término ({}) deve ser estritamente posterior à data de início ({})",
            command.end_date, command.start_date
        )));
    }

    if let Some(_existing) = mandate_repo
        .find_active_by_academic_center_id(command.academic_center_id)
        .await?
    {
        return Err(MandateApplicationError::ActiveMandateAlreadyExists(
            command.academic_center_id,
        ));
    }

    let mandate = Mandate::new(
        command.academic_center_id,
        command.actor_user_id,
        command.title.as_deref(),
        command.start_date,
        command.end_date,
    )?;

    mandate_repo.save(&mandate).await?;

    Ok(mandate)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use async_trait::async_trait;
    use chrono::Utc;

    use super::*;
    use crate::modules::ca::{
        application::ports::{
            AcademicCenterRepositoryError, DefaultAcademicCenterAuthorization,
            MandateRepositoryError,
        },
        domain::entities::AcademicCenter,
    };

    #[derive(Default, Clone)]
    struct MockCaRepo {
        centers: Arc<RwLock<HashMap<Uuid, AcademicCenter>>>,
    }

    #[async_trait]
    impl AcademicCenterRepository for MockCaRepo {
        async fn find_by_id(
            &self,
            id: Uuid,
        ) -> Result<Option<AcademicCenter>, AcademicCenterRepositoryError> {
            Ok(self.centers.read().unwrap().get(&id).cloned())
        }
    }

    #[derive(Default, Clone)]
    struct MockMandateRepo {
        mandates: Arc<RwLock<Vec<Mandate>>>,
    }

    #[async_trait]
    impl MandateRepository for MockMandateRepo {
        async fn save(&self, mandate: &Mandate) -> Result<(), MandateRepositoryError> {
            let mut list = self.mandates.write().unwrap();
            list.retain(|m| m.id() != mandate.id());
            list.push(mandate.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Mandate>, MandateRepositoryError> {
            Ok(self
                .mandates
                .read()
                .unwrap()
                .iter()
                .find(|m| m.id() == id)
                .cloned())
        }

        async fn find_active_by_academic_center_id(
            &self,
            academic_center_id: Uuid,
        ) -> Result<Option<Mandate>, MandateRepositoryError> {
            Ok(self
                .mandates
                .read()
                .unwrap()
                .iter()
                .find(|m| m.academic_center_id() == academic_center_id && m.is_active())
                .cloned())
        }

        async fn find_all_by_academic_center_id(
            &self,
            academic_center_id: Uuid,
        ) -> Result<Vec<Mandate>, MandateRepositoryError> {
            Ok(self
                .mandates
                .read()
                .unwrap()
                .iter()
                .filter(|m| m.academic_center_id() == academic_center_id)
                .cloned()
                .collect())
        }
    }

    fn create_test_ca(
        id: Uuid,
        created_by_id: Option<Uuid>,
        status: AcademicCenterStatus,
    ) -> AcademicCenter {
        let now = Utc::now();
        AcademicCenter::reconstitute(
            id,
            Uuid::new_v4(),
            created_by_id,
            "Centro Acadêmico Teste".into(),
            Some("CAT".into()),
            "Descrição do teste".into(),
            status,
            now,
            now,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn should_configure_initial_mandate_successfully() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let ca = create_test_ca(ca_id, Some(user_id), AcademicCenterStatus::Active);

        let ca_repo = MockCaRepo::default();
        ca_repo.centers.write().unwrap().insert(ca_id, ca);

        let mandate_repo = MockMandateRepo::default();
        let auth = DefaultAcademicCenterAuthorization;

        let cmd = ConfigureInitialMandateCommand {
            academic_center_id: ca_id,
            actor_user_id: user_id,
            title: Some("Gestão 2026-2027".into()),
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        };

        let result = configure_initial_mandate(&ca_repo, &mandate_repo, &auth, cmd)
            .await
            .expect("should succeed");

        assert_eq!(result.academic_center_id(), ca_id);
        assert_eq!(result.created_by_id(), user_id);
        assert_eq!(result.title(), Some("Gestão 2026-2027"));
        assert_eq!(result.duration_in_days(), 364);
        assert!(result.is_active());

        // Verificação de persistência
        let saved = mandate_repo
            .find_active_by_academic_center_id(ca_id)
            .await
            .unwrap();
        assert!(saved.is_some());
        assert_eq!(saved.unwrap().id(), result.id());
    }

    #[tokio::test]
    async fn should_reject_configuration_when_user_is_not_permitted_in_ca() {
        let ca_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();
        let ca = create_test_ca(ca_id, Some(creator_id), AcademicCenterStatus::Active);

        let ca_repo = MockCaRepo::default();
        ca_repo.centers.write().unwrap().insert(ca_id, ca);

        let mandate_repo = MockMandateRepo::default();
        let auth = DefaultAcademicCenterAuthorization;

        let cmd = ConfigureInitialMandateCommand {
            academic_center_id: ca_id,
            actor_user_id: other_user_id,
            title: None,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        };

        let err = configure_initial_mandate(&ca_repo, &mandate_repo, &auth, cmd)
            .await
            .unwrap_err();

        assert_eq!(
            err,
            MandateApplicationError::UserNotPermittedInCa {
                user_id: other_user_id,
                academic_center_id: ca_id,
            }
        );
    }

    #[tokio::test]
    async fn should_reject_configuration_when_dates_are_invalid() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let ca = create_test_ca(ca_id, Some(user_id), AcademicCenterStatus::Active);

        let ca_repo = MockCaRepo::default();
        ca_repo.centers.write().unwrap().insert(ca_id, ca);

        let mandate_repo = MockMandateRepo::default();
        let auth = DefaultAcademicCenterAuthorization;

        let cmd = ConfigureInitialMandateCommand {
            academic_center_id: ca_id,
            actor_user_id: user_id,
            title: None,
            start_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        };

        let err = configure_initial_mandate(&ca_repo, &mandate_repo, &auth, cmd)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            MandateApplicationError::InvalidMandateDates(_)
        ));
    }

    #[tokio::test]
    async fn should_reject_duplicate_configuration_when_active_mandate_exists() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let ca = create_test_ca(ca_id, Some(user_id), AcademicCenterStatus::Active);

        let ca_repo = MockCaRepo::default();
        ca_repo.centers.write().unwrap().insert(ca_id, ca);

        let mandate_repo = MockMandateRepo::default();
        let existing = Mandate::new(
            ca_id,
            user_id,
            Some("Gestão Vigente"),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        )
        .unwrap();
        mandate_repo.save(&existing).await.unwrap();

        let auth = DefaultAcademicCenterAuthorization;

        let cmd = ConfigureInitialMandateCommand {
            academic_center_id: ca_id,
            actor_user_id: user_id,
            title: Some("Nova Gestão".into()),
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        };

        let err = configure_initial_mandate(&ca_repo, &mandate_repo, &auth, cmd)
            .await
            .unwrap_err();

        assert_eq!(
            err,
            MandateApplicationError::ActiveMandateAlreadyExists(ca_id)
        );
    }

    #[tokio::test]
    async fn should_reject_when_ca_not_found() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let ca_repo = MockCaRepo::default();
        let mandate_repo = MockMandateRepo::default();
        let auth = DefaultAcademicCenterAuthorization;

        let cmd = ConfigureInitialMandateCommand {
            academic_center_id: ca_id,
            actor_user_id: user_id,
            title: None,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        };

        let err = configure_initial_mandate(&ca_repo, &mandate_repo, &auth, cmd)
            .await
            .unwrap_err();

        assert_eq!(err, MandateApplicationError::AcademicCenterNotFound(ca_id));
    }

    #[tokio::test]
    async fn should_reject_when_ca_is_archived() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let ca = create_test_ca(ca_id, Some(user_id), AcademicCenterStatus::Archived);

        let ca_repo = MockCaRepo::default();
        ca_repo.centers.write().unwrap().insert(ca_id, ca);

        let mandate_repo = MockMandateRepo::default();
        let auth = DefaultAcademicCenterAuthorization;

        let cmd = ConfigureInitialMandateCommand {
            academic_center_id: ca_id,
            actor_user_id: user_id,
            title: None,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        };

        let err = configure_initial_mandate(&ca_repo, &mandate_repo, &auth, cmd)
            .await
            .unwrap_err();

        assert_eq!(err, MandateApplicationError::AcademicCenterInactive(ca_id));
    }

    #[tokio::test]
    async fn should_enforce_isolation_between_different_cas() {
        let ca1_id = Uuid::new_v4();
        let ca2_id = Uuid::new_v4();
        let gestor1_id = Uuid::new_v4();
        let gestor2_id = Uuid::new_v4();

        let ca1 = create_test_ca(ca1_id, Some(gestor1_id), AcademicCenterStatus::Active);
        let ca2 = create_test_ca(ca2_id, Some(gestor2_id), AcademicCenterStatus::Active);

        let ca_repo = MockCaRepo::default();
        ca_repo.centers.write().unwrap().insert(ca1_id, ca1);
        ca_repo.centers.write().unwrap().insert(ca2_id, ca2);

        let mandate_repo = MockMandateRepo::default();
        let auth = DefaultAcademicCenterAuthorization;

        // Gestor 1 tenta configurar mandato do CA 2
        let cmd = ConfigureInitialMandateCommand {
            academic_center_id: ca2_id,
            actor_user_id: gestor1_id,
            title: None,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        };

        let err = configure_initial_mandate(&ca_repo, &mandate_repo, &auth, cmd)
            .await
            .unwrap_err();

        assert_eq!(
            err,
            MandateApplicationError::UserNotPermittedInCa {
                user_id: gestor1_id,
                academic_center_id: ca2_id,
            }
        );
    }
}
