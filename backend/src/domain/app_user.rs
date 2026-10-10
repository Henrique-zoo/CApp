//! Reexportação dos tipos e contratos da entidade de domínio Usuário.

pub use crate::modules::identity::domain::entities::app_user::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_access_app_user_from_domain_reexport() {
        let user = AppUser::veterano_didactic();
        assert_eq!(user.id, VETERANO_1_DIDACTIC_ID);
        assert_eq!(user.institutional_email, "231012345@aluno.unb.br");
    }
}
