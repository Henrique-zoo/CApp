use mockall::mock;
use uuid::Uuid;

use crate::modules::identity::{
    application::{
        error::{
            AuthenticationError, InstitutionRepositoryError,
            InstitutionalVerificationRepositoryError,
        },
        ports::{
            IdentityTokenVerifier, InstitutionRepository, InstitutionalDirectory,
            InstitutionalProfile, InstitutionalVerificationRepository, VerifiedIdentity,
        },
    },
    domain::entities::{
        institution::Institution, institutional_verification::InstitutionalVerification,
    },
};

mock! {
    pub Verifier {}

    #[async_trait::async_trait]
    impl IdentityTokenVerifier for Verifier {
        async fn verify(&self, token: &str) -> Result<VerifiedIdentity, AuthenticationError>;
    }
}
mock! {
    pub Directory {}

    #[async_trait::async_trait]
    impl InstitutionalDirectory for Directory {
        async fn get_profile(&self, token: &str, is_student: bool) -> Result<InstitutionalProfile, AuthenticationError>;
        async fn get_tenant_id(&self, token: &str) -> Result<Uuid, AuthenticationError>;
        async fn is_student_member(&self, token: &str, group_id: Uuid) -> Result<bool, AuthenticationError>;
    }
}
mock! {
    // Um só mock implementa AS DUAS traits, como exige a assinatura atual
    // de verify_institutional_identity. Não precisa alterar o caso de uso.
    pub Repositories {}

    #[async_trait::async_trait]
    impl InstitutionRepository for Repositories {
        async fn find_by_microsoft_tenant_id(&self, tenant_id: Uuid) -> Result<Option<Institution>, InstitutionRepositoryError>;
    }

    #[async_trait::async_trait]
    impl InstitutionalVerificationRepository for Repositories {
        async fn save(&self, verification: &InstitutionalVerification) -> Result<bool, InstitutionalVerificationRepositoryError>;
        async fn find_valid_by_firebase_uid(&self, firebase_uid: &str) -> Result<Option<InstitutionalVerification>, InstitutionalVerificationRepositoryError>;
    }
}
