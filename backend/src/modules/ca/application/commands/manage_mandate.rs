//! Casos de uso de conclusão, transição e gerenciamento de integrantes da gestão (Issue #298).
//!
//! Garante a preservação imutável do histórico de mandatos anteriores e
//! a rastreabilidade da participação dos integrantes mesmo após desvinculação.

use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use crate::modules::ca::{
    application::{
        error::MandateApplicationError,
        ports::{AcademicCenterAuthorizationPort, AcademicCenterRepository, MandateRepository},
    },
    domain::entities::{AcademicCenterStatus, Mandate, MandateMember},
};

/// Dados para encerramento/conclusão do mandato ativo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcludeActiveMandateCommand {
    /// Identificador do Centro Acadêmico.
    pub academic_center_id: Uuid,
    /// Identificador do usuário que está solicitando o encerramento.
    pub actor_user_id: Uuid,
}

/// Encerra e conclui o mandato atualmente ativo, preservando seu histórico de forma imutável.
pub async fn conclude_active_mandate(
    ca_repo: &impl AcademicCenterRepository,
    mandate_repo: &impl MandateRepository,
    auth_port: &impl AcademicCenterAuthorizationPort,
    command: ConcludeActiveMandateCommand,
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

    let mut mandate = mandate_repo
        .find_active_by_academic_center_id(command.academic_center_id)
        .await?
        .ok_or(MandateApplicationError::ActiveMandateNotFound(
            command.academic_center_id,
        ))?;

    let now = Utc::now();
    mandate.conclude(now)?;
    mandate_repo.save(&mandate).await?;

    Ok(mandate)
}

/// Dados para comando de transição de gestão.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionMandateCommand {
    /// Identificador do Centro Acadêmico.
    pub academic_center_id: Uuid,
    /// Identificador do usuário que está realizando a transição.
    pub actor_user_id: Uuid,
    /// Título opcional da nova chapa ou gestão.
    pub title: Option<String>,
    /// Data de início do novo mandato (AAAA-MM-DD).
    pub start_date: NaiveDate,
    /// Data de término do novo mandato (AAAA-MM-DD).
    pub end_date: NaiveDate,
}

/// Executa a transição entre gestões, preservando automaticamente o mandato anterior no histórico.
pub async fn transition_mandate(
    ca_repo: &impl AcademicCenterRepository,
    mandate_repo: &impl MandateRepository,
    auth_port: &impl AcademicCenterAuthorizationPort,
    command: TransitionMandateCommand,
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

    let now = Utc::now();

    // Preserva automaticamente o mandato ativo anterior, se houver
    if let Some(mut previous) = mandate_repo
        .find_active_by_academic_center_id(command.academic_center_id)
        .await?
    {
        previous.conclude(now)?;
        mandate_repo.save(&previous).await?;
    }

    // Cria e ativa a nova gestão
    let new_mandate = Mandate::new(
        command.academic_center_id,
        command.actor_user_id,
        command.title.as_deref(),
        command.start_date,
        command.end_date,
    )?;

    mandate_repo.save(&new_mandate).await?;

    Ok(new_mandate)
}

/// Dados para vinculação de integrante à gestão.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddMandateMemberCommand {
    /// Identificador do Centro Acadêmico.
    pub academic_center_id: Uuid,
    /// Identificador do mandato ao qual o integrante será vinculado.
    pub mandate_id: Uuid,
    /// Identificador do usuário que está realizando a vinculação.
    pub actor_user_id: Uuid,
    /// Identificador opcional da conta do integrante no aplicativo.
    pub member_user_id: Option<Uuid>,
    /// Nome do integrante.
    pub name: String,
    /// Cargo do integrante na gestão.
    pub role: String,
}

/// Vincula um integrante à gestão especificada.
pub async fn add_mandate_member(
    ca_repo: &impl AcademicCenterRepository,
    mandate_repo: &impl MandateRepository,
    auth_port: &impl AcademicCenterAuthorizationPort,
    command: AddMandateMemberCommand,
) -> Result<MandateMember, MandateApplicationError> {
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

    let mut mandate = mandate_repo
        .find_by_id(command.mandate_id)
        .await?
        .ok_or(MandateApplicationError::MandateNotFound(command.mandate_id))?;

    if mandate.academic_center_id() != command.academic_center_id {
        return Err(MandateApplicationError::MandateNotFound(command.mandate_id));
    }

    let member = MandateMember::new(&command.name, &command.role, command.member_user_id)?;
    mandate.add_member(member.clone());
    mandate_repo.save(&mandate).await?;

    Ok(member)
}

/// Dados para desvinculação de integrante da gestão.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisaffiliateMandateMemberCommand {
    /// Identificador do Centro Acadêmico.
    pub academic_center_id: Uuid,
    /// Identificador do mandato correspondente.
    pub mandate_id: Uuid,
    /// Identificador do vínculo do integrante a ser encerrado.
    pub member_id: Uuid,
    /// Identificador do usuário que está solicitando a desvinculação.
    pub actor_user_id: Uuid,
}

/// Desvincula um integrante preservando sua participação histórica.
pub async fn disaffiliate_mandate_member(
    ca_repo: &impl AcademicCenterRepository,
    mandate_repo: &impl MandateRepository,
    auth_port: &impl AcademicCenterAuthorizationPort,
    command: DisaffiliateMandateMemberCommand,
) -> Result<MandateMember, MandateApplicationError> {
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

    let mut mandate = mandate_repo
        .find_by_id(command.mandate_id)
        .await?
        .ok_or(MandateApplicationError::MandateNotFound(command.mandate_id))?;

    if mandate.academic_center_id() != command.academic_center_id {
        return Err(MandateApplicationError::MandateNotFound(command.mandate_id));
    }

    let now = Utc::now();
    mandate.disaffiliate_member(command.member_id, now)?;
    mandate_repo.save(&mandate).await?;

    let member = mandate
        .members()
        .iter()
        .find(|m| m.id() == command.member_id)
        .cloned()
        .ok_or(MandateApplicationError::MemberNotFound(command.member_id))?;

    Ok(member)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use async_trait::async_trait;

    use super::*;
    use crate::modules::ca::{
        application::ports::{
            AcademicCenterRepositoryError, DefaultAcademicCenterAuthorization,
            MandateRepositoryError,
        },
        domain::entities::{AcademicCenter, AcademicCenterStatus, MandateStatus},
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
            let mut list: Vec<Mandate> = self
                .mandates
                .read()
                .unwrap()
                .iter()
                .filter(|m| m.academic_center_id() == academic_center_id)
                .cloned()
                .collect();
            list.sort_by(|a, b| {
                b.start_date()
                    .cmp(&a.start_date())
                    .then_with(|| b.created_at().cmp(&a.created_at()))
            });
            Ok(list)
        }
    }

    #[tokio::test]
    async fn should_conclude_active_mandate_successfully() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let ca = AcademicCenter::reconstitute(
            ca_id,
            Uuid::new_v4(),
            Some(user_id),
            "CA Computação".into(),
            Some("CACC".into()),
            "Descrição".into(),
            AcademicCenterStatus::Active,
            now,
            now,
        )
        .unwrap();

        let ca_repo = MockCaRepo::default();
        ca_repo.centers.write().unwrap().insert(ca_id, ca);

        let mandate_repo = MockMandateRepo::default();
        let mandate = Mandate::new(
            ca_id,
            user_id,
            Some("Gestão 2025"),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        )
        .unwrap();
        mandate_repo.save(&mandate).await.unwrap();

        let auth = DefaultAcademicCenterAuthorization;
        let command = ConcludeActiveMandateCommand {
            academic_center_id: ca_id,
            actor_user_id: user_id,
        };

        let concluded = conclude_active_mandate(&ca_repo, &mandate_repo, &auth, command)
            .await
            .expect("should conclude");

        assert_eq!(concluded.status(), MandateStatus::Concluded);
        assert!(concluded.concluded_at().is_some());
    }

    #[tokio::test]
    async fn should_transition_mandate_and_preserve_previous_mandate_in_history() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let ca = AcademicCenter::reconstitute(
            ca_id,
            Uuid::new_v4(),
            Some(user_id),
            "CA Computação".into(),
            Some("CACC".into()),
            "Descrição".into(),
            AcademicCenterStatus::Active,
            now,
            now,
        )
        .unwrap();

        let ca_repo = MockCaRepo::default();
        ca_repo.centers.write().unwrap().insert(ca_id, ca);

        let mandate_repo = MockMandateRepo::default();
        let mut previous = Mandate::new(
            ca_id,
            user_id,
            Some("Gestão 2025"),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        )
        .unwrap();
        let member_ana = MandateMember::new("Ana", "Presidente", Some(Uuid::new_v4())).unwrap();
        previous.add_member(member_ana);
        mandate_repo.save(&previous).await.unwrap();

        let auth = DefaultAcademicCenterAuthorization;
        let command = TransitionMandateCommand {
            academic_center_id: ca_id,
            actor_user_id: user_id,
            title: Some("Gestão 2026".into()),
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        };

        let new_mandate = transition_mandate(&ca_repo, &mandate_repo, &auth, command)
            .await
            .expect("should transition");

        assert_eq!(new_mandate.status(), MandateStatus::Active);
        assert_eq!(new_mandate.title(), Some("Gestão 2026"));

        // Verifica que o mandato anterior foi preservado e concluído
        let saved_previous = mandate_repo
            .find_by_id(previous.id())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(saved_previous.status(), MandateStatus::Concluded);
        assert!(saved_previous.concluded_at().is_some());
        assert_eq!(saved_previous.members().len(), 1);
        assert_eq!(saved_previous.members()[0].name(), "Ana");
        assert_eq!(saved_previous.members()[0].role(), "Presidente");

        // Todos os mandatos continuam no histórico
        let all = mandate_repo
            .find_all_by_academic_center_id(ca_id)
            .await
            .unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn should_add_member_and_preserve_history_after_disaffiliation() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let ca = AcademicCenter::reconstitute(
            ca_id,
            Uuid::new_v4(),
            Some(user_id),
            "CA Computação".into(),
            Some("CACC".into()),
            "Descrição".into(),
            AcademicCenterStatus::Active,
            now,
            now,
        )
        .unwrap();

        let ca_repo = MockCaRepo::default();
        ca_repo.centers.write().unwrap().insert(ca_id, ca);

        let mandate_repo = MockMandateRepo::default();
        let mandate = Mandate::new(
            ca_id,
            user_id,
            Some("Gestão 2026"),
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        )
        .unwrap();
        mandate_repo.save(&mandate).await.unwrap();

        let auth = DefaultAcademicCenterAuthorization;

        // Adiciona integrante
        let add_cmd = AddMandateMemberCommand {
            academic_center_id: ca_id,
            mandate_id: mandate.id(),
            actor_user_id: user_id,
            member_user_id: Some(Uuid::new_v4()),
            name: "Bruno".into(),
            role: "Tesoureiro".into(),
        };

        let member = add_mandate_member(&ca_repo, &mandate_repo, &auth, add_cmd)
            .await
            .expect("should add member");
        assert_eq!(member.name(), "Bruno");
        assert_eq!(member.role(), "Tesoureiro");
        assert!(member.is_active());

        // Desvincula integrante
        let disaffiliate_cmd = DisaffiliateMandateMemberCommand {
            academic_center_id: ca_id,
            mandate_id: mandate.id(),
            member_id: member.id(),
            actor_user_id: user_id,
        };

        let disaffiliated =
            disaffiliate_mandate_member(&ca_repo, &mandate_repo, &auth, disaffiliate_cmd)
                .await
                .expect("should disaffiliate");
        assert!(!disaffiliated.is_active());
        assert!(disaffiliated.left_at().is_some());

        // Histórico preserva a participação
        let updated_mandate = mandate_repo
            .find_by_id(mandate.id())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_mandate.members().len(), 1);
        assert_eq!(updated_mandate.active_members().count(), 0);
        assert_eq!(updated_mandate.members()[0].name(), "Bruno");
        assert_eq!(updated_mandate.members()[0].role(), "Tesoureiro");
    }
}
