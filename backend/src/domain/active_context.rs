//! Reexportação dos tipos e contratos da entidade de domínio Contexto Ativo de CA.

pub use crate::modules::ca::domain::entities::active_context::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_access_active_context_from_domain_reexport() {
        let user_id = uuid::Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);
        let ca_id = uuid::Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0001);

        let context = ActiveCaContext::new(user_id, ca_id).expect("valid context");
        assert_eq!(context.user_id(), user_id);
        assert_eq!(context.active_ca_id(), ca_id);
        assert!(context.is_navigation_only());
        assert!(!context.confers_administrative_permissions());
    }
}
