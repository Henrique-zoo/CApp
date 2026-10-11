//! Regras de domínio aplicáveis às preferências de navegação do usuário.
//!
//! # Regras de Negócio (Issue #44 / I01)
//! 1. Conveniência de Navegação: O Centro Acadêmico favorito serve como ponto de partida
//!    para a experiência do usuário, determinando o contexto inicial exibido.
//! 2. Desacoplamento de Vínculo: Ter um CA como favorito não estabelece vínculo acadêmico
//!    com a instituição ou o curso daquele CA.
//! 3. Sem Direitos Políticos: A preferência não confere elegibilidade eleitoral (direito a voto).
//! 4. Sem Poderes Administrativos: A preferência não confere cargos nem permissões administrativas.

use uuid::Uuid;

use crate::modules::identity::domain::entities::user_preference::{
    UserPreference, UserPreferenceError,
};

/// Valida a atribuição de um Centro Acadêmico favorito para navegação.
///
/// # Erros
///
/// Retorna [`UserPreferenceError::NilFavoriteCaId`] se o identificador do CA for nulo.
pub fn validate_favorite_ca_selection(
    _preference: &UserPreference,
    target_ca_id: Uuid,
) -> Result<(), UserPreferenceError> {
    if target_ca_id.is_nil() {
        return Err(UserPreferenceError::NilFavoriteCaId);
    }
    Ok(())
}

/// Assegura que o registro de preferência mantenha estritamente o escopo de navegação.
///
/// Sempre retorna `Ok(())` quando a preferência é consistente com sua natureza de navegação.
pub fn assert_navigation_only_scope(
    preference: &UserPreference,
) -> Result<(), UserPreferenceError> {
    if !preference.is_navigation_convenience_only() {
        return Err(UserPreferenceError::PrivilegeConferralAttempt);
    }
    Ok(())
}

/// Avalia se a preferência de CA concede direito a voto ou participação eleitoral.
///
/// Sempre retorna `false`. A elegibilidade eleitoral depende do censo e das regras
/// de cada comissão eleitoral, nunca da preferência de interface.
pub fn is_eligible_to_vote_via_preference(
    _preference: &UserPreference,
    _election_ca_id: &Uuid,
) -> bool {
    false
}

/// Avalia se a preferência de CA concede privilégios administrativos.
///
/// Sempre retorna `false`. Permissões decorrem exclusivamente de atribuições de gestão
/// vigentes (`management_assignment`).
pub fn has_administrative_privileges_via_preference(
    _preference: &UserPreference,
    _ca_id: &Uuid,
) -> bool {
    false
}

/// Avalia se a preferência de CA confere vínculo acadêmico com o curso representado.
///
/// Sempre retorna `false`. O vínculo acadêmico deriva exclusivamente do registro institucional
/// (`academic_enrollment`).
pub fn has_academic_enrollment_via_preference(_preference: &UserPreference, _ca_id: &Uuid) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAER_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0001);
    const CACC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0002);
    const USER_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);

    #[test]
    fn should_validate_favorite_selection_with_valid_id() {
        let pref = UserPreference::default_for_user(USER_ID).expect("valid pref");
        assert!(validate_favorite_ca_selection(&pref, CAER_ID).is_ok());
        assert!(validate_favorite_ca_selection(&pref, CACC_ID).is_ok());
    }

    #[test]
    fn should_reject_favorite_selection_with_nil_id() {
        let pref = UserPreference::default_for_user(USER_ID).expect("valid pref");
        assert_eq!(
            validate_favorite_ca_selection(&pref, Uuid::nil()).unwrap_err(),
            UserPreferenceError::NilFavoriteCaId
        );
    }

    #[test]
    fn should_assert_strict_domain_rules_prohibiting_privilege_escalation() {
        let pref = UserPreference::new(USER_ID, Some(CAER_ID)).expect("valid pref");

        assert!(assert_navigation_only_scope(&pref).is_ok());
        assert!(!is_eligible_to_vote_via_preference(&pref, &CAER_ID));
        assert!(!has_administrative_privileges_via_preference(
            &pref, &CAER_ID
        ));
        assert!(!has_academic_enrollment_via_preference(&pref, &CAER_ID));
    }
}
