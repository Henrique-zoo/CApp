//! Reexportação dos tipos e contratos da entidade de domínio Preferência do Usuário (`user_preference`).

pub use crate::modules::identity::domain::entities::user_preference::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_access_user_preference_from_domain_reexport() {
        let user_id = uuid::Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);
        let pref = UserPreference::default_for_user(user_id).expect("preference");
        assert_eq!(pref.user_id(), user_id);
        assert_eq!(pref.favorite_ca_id(), None);
    }
}
