//! Caso de uso de consulta do mandato vigente de um Centro Acadêmico.

use uuid::Uuid;

use crate::modules::ca::{
    application::{
        error::MandateApplicationError,
        ports::{AcademicCenterRepository, MandateRepository},
    },
    domain::entities::Mandate,
};

/// Consulta o mandato atualmente ativo de um Centro Acadêmico.
///
/// # Regras
///
/// - Retorna [`MandateApplicationError::AcademicCenterNotFound`] se o CA não existir.
/// - Retorna `Ok(Some(mandate))` se houver mandato ativo.
/// - Retorna `Ok(None)` se o CA existir mas ainda não possuir mandato ativo registrado.
pub async fn get_current_mandate(
    ca_repo: &impl AcademicCenterRepository,
    mandate_repo: &impl MandateRepository,
    academic_center_id: Uuid,
) -> Result<Option<Mandate>, MandateApplicationError> {
    let ca = ca_repo.find_by_id(academic_center_id).await?;
    if ca.is_none() {
        return Err(MandateApplicationError::AcademicCenterNotFound(
            academic_center_id,
        ));
    }

    let active_mandate = mandate_repo
        .find_active_by_academic_center_id(academic_center_id)
        .await?;

    Ok(active_mandate)
}

/// Lista todos os mandatos associados a um Centro Acadêmico (histórico).
pub async fn list_mandates(
    ca_repo: &impl AcademicCenterRepository,
    mandate_repo: &impl MandateRepository,
    academic_center_id: Uuid,
) -> Result<Vec<Mandate>, MandateApplicationError> {
    get_mandates_history(ca_repo, mandate_repo, academic_center_id).await
}

/// Consulta o histórico completo de gestões de um Centro Acadêmico ordenado cronologicamente.
///
/// # Regras
///
/// - Retorna [`MandateApplicationError::AcademicCenterNotFound`] se o CA não existir.
/// - Retorna a lista de mandatos ordenados do mais recente para o mais antigo.
/// - Retorna `Ok(vec![])` quando o CA não possuir gestões registradas desde sua adesão.
pub async fn get_mandates_history(
    ca_repo: &impl AcademicCenterRepository,
    mandate_repo: &impl MandateRepository,
    academic_center_id: Uuid,
) -> Result<Vec<Mandate>, MandateApplicationError> {
    let ca = ca_repo.find_by_id(academic_center_id).await?;
    if ca.is_none() {
        return Err(MandateApplicationError::AcademicCenterNotFound(
            academic_center_id,
        ));
    }

    let mandates = mandate_repo
        .find_all_by_academic_center_id(academic_center_id)
        .await?;

    Ok(mandates)
}

/// Consulta os detalhes de uma gestão específica do Centro Acadêmico pelo seu identificador.
///
/// # Regras
///
/// - Retorna [`MandateApplicationError::AcademicCenterNotFound`] se o CA não existir.
/// - Retorna [`MandateApplicationError::MandateNotFound`] se o mandato não existir ou pertencer a outro CA.
pub async fn get_mandate_by_id(
    ca_repo: &impl AcademicCenterRepository,
    mandate_repo: &impl MandateRepository,
    academic_center_id: Uuid,
    mandate_id: Uuid,
) -> Result<Mandate, MandateApplicationError> {
    let ca = ca_repo.find_by_id(academic_center_id).await?;
    if ca.is_none() {
        return Err(MandateApplicationError::AcademicCenterNotFound(
            academic_center_id,
        ));
    }

    let mandate = mandate_repo
        .find_by_id(mandate_id)
        .await?
        .ok_or(MandateApplicationError::MandateNotFound(mandate_id))?;

    if mandate.academic_center_id() != academic_center_id {
        return Err(MandateApplicationError::MandateNotFound(mandate_id));
    }

    Ok(mandate)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use async_trait::async_trait;
    use chrono::{NaiveDate, Utc};

    use super::*;
    use crate::modules::ca::{
        application::ports::{AcademicCenterRepositoryError, MandateRepositoryError},
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
            self.mandates.write().unwrap().push(mandate.clone());
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
    async fn should_return_empty_history_when_ca_has_no_mandates() {
        let ca_id = Uuid::new_v4();
        let now = Utc::now();
        let ca = AcademicCenter::reconstitute(
            ca_id,
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
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

        let history = get_mandates_history(&ca_repo, &mandate_repo, ca_id)
            .await
            .expect("should query history");
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn should_return_chronologically_ordered_history_and_details() {
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

        // Mandato 2024 (concluído)
        let mut m2024 = Mandate::new(
            ca_id,
            user_id,
            Some("Gestão 2024"),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        )
        .unwrap();
        let conclude_time = m2024.created_at() + chrono::Duration::seconds(1);
        m2024.conclude(conclude_time).unwrap();
        mandate_repo.save(&m2024).await.unwrap();

        // Mandato 2025 (ativo)
        let m2025 = Mandate::new(
            ca_id,
            user_id,
            Some("Gestão 2025"),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        )
        .unwrap();
        mandate_repo.save(&m2025).await.unwrap();

        let history = get_mandates_history(&ca_repo, &mandate_repo, ca_id)
            .await
            .expect("should query history");
        assert_eq!(history.len(), 2);
        // Primeiro o mais recente (2025)
        assert_eq!(history[0].id(), m2025.id());
        assert_eq!(history[0].status(), MandateStatus::Active);
        // Depois o anterior (2024)
        assert_eq!(history[1].id(), m2024.id());
        assert_eq!(history[1].status(), MandateStatus::Concluded);

        // Consulta detalhes pelo ID
        let detail = get_mandate_by_id(&ca_repo, &mandate_repo, ca_id, m2024.id())
            .await
            .expect("should find by id");
        assert_eq!(detail.id(), m2024.id());
        assert_eq!(detail.title(), Some("Gestão 2024"));
    }

    #[tokio::test]
    async fn should_enforce_isolation_in_mandate_history_between_cas() {
        let ca1_id = Uuid::new_v4();
        let ca2_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let ca1 = AcademicCenter::reconstitute(
            ca1_id,
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

        let ca2 = AcademicCenter::reconstitute(
            ca2_id,
            Uuid::new_v4(),
            Some(user_id),
            "CA Matemática".into(),
            Some("CAMAT".into()),
            "Descrição".into(),
            AcademicCenterStatus::Active,
            now,
            now,
        )
        .unwrap();

        let ca_repo = MockCaRepo::default();
        ca_repo.centers.write().unwrap().insert(ca1_id, ca1);
        ca_repo.centers.write().unwrap().insert(ca2_id, ca2);

        let mandate_repo = MockMandateRepo::default();
        let m2 = Mandate::new(
            ca2_id,
            user_id,
            Some("Gestão CAMAT 2025"),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        )
        .unwrap();
        mandate_repo.save(&m2).await.unwrap();

        // Histórico de CA1 deve ser vazio e não vazar mandatos de CA2
        let history1 = get_mandates_history(&ca_repo, &mandate_repo, ca1_id)
            .await
            .unwrap();
        assert!(history1.is_empty());

        // Consulta de mandato de CA2 com CA1 deve falhar por isolamento
        let err = get_mandate_by_id(&ca_repo, &mandate_repo, ca1_id, m2.id())
            .await
            .unwrap_err();
        assert_eq!(err, MandateApplicationError::MandateNotFound(m2.id()));
    }

    #[tokio::test]
    async fn should_return_none_when_ca_has_no_active_mandate() {
        let ca_id = Uuid::new_v4();
        let now = Utc::now();
        let ca = AcademicCenter::reconstitute(
            ca_id,
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
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

        let current = get_current_mandate(&ca_repo, &mandate_repo, ca_id)
            .await
            .expect("should query");
        assert!(current.is_none());
    }

    #[tokio::test]
    async fn should_return_active_mandate_when_it_exists() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        let ca = AcademicCenter::reconstitute(
            ca_id,
            Uuid::new_v4(),
            Some(user_id),
            "CA Redes".into(),
            Some("CAER".into()),
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

        let current = get_current_mandate(&ca_repo, &mandate_repo, ca_id)
            .await
            .expect("should query");
        assert!(current.is_some());
        assert_eq!(current.unwrap().id(), mandate.id());
    }

    #[tokio::test]
    async fn should_return_error_when_querying_non_existent_ca() {
        let ca_repo = MockCaRepo::default();
        let mandate_repo = MockMandateRepo::default();
        let fake_id = Uuid::new_v4();

        let err = get_current_mandate(&ca_repo, &mandate_repo, fake_id)
            .await
            .unwrap_err();
        assert_eq!(
            err,
            MandateApplicationError::AcademicCenterNotFound(fake_id)
        );
    }
}
