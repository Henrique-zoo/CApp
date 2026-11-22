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
        domain::entities::{AcademicCenter, AcademicCenterStatus},
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
