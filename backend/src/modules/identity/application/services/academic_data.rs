//! Casos de uso e consultas para associação e verificação de dados acadêmicos.
//!
//! Permite que os módulos e regras de negócio da aplicação consultem o contexto
//! acadêmico associado a um usuário (curso, vínculo e elegibilidade estudantil),
//! satisfazendo § 1.3 do Relatório de Features e os critérios da Issue #8.

use uuid::Uuid;

use crate::modules::identity::{
    application::{
        error::InstitutionalVerificationRepositoryError, ports::InstitutionalVerificationRepository,
    },
    domain::entities::academic_data::AcademicData,
};

/// Consulta os dados acadêmicos vigentes associados ao ID interno de um usuário.
///
/// # Parâmetros
///
/// - `repository`: repositório de persistência de evidências institucionais.
/// - `user_id`: identificador interno do usuário.
///
/// # Retorno
///
/// `Ok(Some(AcademicData))` se houver verificação institucional vigente e compatível;
/// `Ok(None)` se ausente ou expirada; `Err(...)` para falhas técnicas de infraestrutura.
pub async fn consult_academic_data_by_user_id<R: InstitutionalVerificationRepository>(
    repository: &R,
    user_id: Uuid,
) -> Result<Option<AcademicData>, InstitutionalVerificationRepositoryError> {
    let verification = repository.find_valid_by_user_id(user_id).await?;
    Ok(verification.map(|v| v.academic_data()))
}

/// Consulta os dados acadêmicos vigentes a partir do UID Firebase da identidade autenticada.
///
/// # Parâmetros
///
/// - `repository`: repositório de persistência de evidências institucionais.
/// - `firebase_uid`: UID do Firebase autenticado.
pub async fn consult_academic_data_by_firebase_uid<R: InstitutionalVerificationRepository>(
    repository: &R,
    firebase_uid: &str,
) -> Result<Option<AcademicData>, InstitutionalVerificationRepositoryError> {
    let verification = repository.find_valid_by_firebase_uid(firebase_uid).await?;
    Ok(verification.map(|v| v.academic_data()))
}

/// Avalia se os dados acadêmicos autorizam o acesso a regras e funcionalidades discentes.
pub fn can_access_student_features(academic_data: Option<&AcademicData>) -> bool {
    academic_data.is_some_and(|data| data.allows_academic_access())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::identity::domain::entities::{
        academic_data::AcademicStanding, institutional_verification::InstitutionalVerification,
    };
    use chrono::{Duration, Utc};

    struct MockRepo {
        verification: Option<InstitutionalVerification>,
    }

    #[async_trait::async_trait]
    impl InstitutionalVerificationRepository for MockRepo {
        async fn save(
            &self,
            _v: &InstitutionalVerification,
        ) -> Result<bool, InstitutionalVerificationRepositoryError> {
            Ok(true)
        }

        async fn find_valid_by_firebase_uid(
            &self,
            _uid: &str,
        ) -> Result<Option<InstitutionalVerification>, InstitutionalVerificationRepositoryError>
        {
            Ok(self.verification.clone())
        }

        async fn find_valid_by_user_id(
            &self,
            _user_id: Uuid,
        ) -> Result<Option<InstitutionalVerification>, InstitutionalVerificationRepositoryError>
        {
            Ok(self.verification.clone())
        }
    }

    #[tokio::test]
    async fn should_consult_academic_data_successfully() {
        let user_id = Uuid::new_v4();
        let institution_id = Uuid::new_v4();
        let checked_at = Utc::now();
        let verification = InstitutionalVerification::new(
            user_id,
            Some("Estudante Teste".into()),
            Some("Engenharia de Software".into()),
            institution_id,
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            true,
            checked_at,
            checked_at + Duration::days(120),
        )
        .unwrap();

        let repo = MockRepo {
            verification: Some(verification),
        };

        let by_user = consult_academic_data_by_user_id(&repo, user_id)
            .await
            .unwrap();
        assert!(by_user.is_some());
        let data = by_user.unwrap();
        assert_eq!(data.user_id(), user_id);
        assert_eq!(data.institution_id(), institution_id);
        assert_eq!(data.course_name(), Some("Engenharia de Software"));
        assert_eq!(data.standing(), AcademicStanding::ActiveStudent);

        let by_firebase = consult_academic_data_by_firebase_uid(&repo, "firebase-uid-123")
            .await
            .unwrap();
        assert!(by_firebase.is_some());
    }

    #[tokio::test]
    async fn should_return_none_when_no_valid_verification() {
        let repo = MockRepo { verification: None };

        let by_user = consult_academic_data_by_user_id(&repo, Uuid::new_v4())
            .await
            .unwrap();
        assert!(by_user.is_none());

        let by_firebase = consult_academic_data_by_firebase_uid(&repo, "firebase-uid-absent")
            .await
            .unwrap();
        assert!(by_firebase.is_none());
    }

    #[test]
    fn should_allow_student_features_only_for_valid_active_student() {
        let student_data = AcademicData::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some("Engenharia de Software".into()),
            AcademicStanding::ActiveStudent,
        );
        assert!(can_access_student_features(Some(&student_data)));

        let non_student_data = AcademicData::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            AcademicStanding::NonStudent,
        );
        assert!(!can_access_student_features(Some(&non_student_data)));

        assert!(!can_access_student_features(None));
    }
}
