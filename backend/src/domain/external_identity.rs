//! Reexportação dos tipos e contratos da entidade de domínio Identidade Externa (`external_identity`).

pub use crate::modules::identity::domain::entities::external_identity::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_access_external_identity_from_domain_reexport() {
        let user_id = uuid::Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);
        let identity = ExternalIdentity::veterano_didactic(user_id);
        assert_eq!(identity.user_id(), user_id);
        assert_eq!(identity.id(), VETERANO_1_EXTERNAL_IDENTITY_ID);
    }
}
