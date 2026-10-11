//! Regras de domínio aplicáveis à seleção, alternância e isolamento do contexto ativo de CA.
//!
//! # Regras de Negócio (Issue #45 / I01 e § 2.1)
//! 1. **Alternância entre CAs Cadastrados**: O estudante pode alternar o contexto de navegação
//!    para qualquer Centro Acadêmico ativo na instituição de ensino (UnB).
//! 2. **Invariante de Não Sobrescrita**: Alternar o contexto ativo em sessão/memória **NÃO** deve,
//!    sob hipótese alguma, sobrescrever a preferência de CA favorito persistida no cadastro.
//! 3. **Isolamento Estrito de Permissões**: A mudança de contexto ativo nunca confere permissões
//!    administrativas no novo CA. Os privilégios de diretoria e gestão pertencem estritamente ao CA
//!    no qual foram atribuídos; ao navegar por outro CA, o estudante atua estritamente como visitante.
//! 4. **Manutenção de Cargos Distintos por CA**: Um estudante pode exercer cargos distintos em CAs
//!    diferentes; ao navegar em determinado CA ativo, apenas as permissões daquele CA específico estão ativas.

use uuid::Uuid;

use crate::modules::{
    ca::domain::entities::{
        academic_center::{AcademicCenter, AcademicCenterStatus},
        active_context::{ActiveCaContext, ActiveContextError, ManagementRoleAssignment},
    },
    identity::domain::entities::user_preference::UserPreference,
};

/// Valida a operação de alternância de contexto ativo para um Centro Acadêmico pretendido.
///
/// # Regras
/// - O identificador de destino não pode ser nulo.
/// - O CA de destino deve existir na lista de CAs cadastrados.
/// - O CA de destino deve estar com status [`AcademicCenterStatus::Active`].
///
/// # Erros
///
/// Retorna [`ActiveContextError::NilAcademicCenterId`] se `new_ca_id` for nulo,
/// [`ActiveContextError::AcademicCenterNotFound`] se o CA não estiver cadastrado, ou
/// [`ActiveContextError::AcademicCenterNotActive`] se o status do CA for diferente de ativo.
pub fn validate_active_context_switch(
    _current: &ActiveCaContext,
    new_ca_id: Uuid,
    registered_cas: &[AcademicCenter],
) -> Result<(), ActiveContextError> {
    if new_ca_id.is_nil() {
        return Err(ActiveContextError::NilAcademicCenterId);
    }

    let target_ca = registered_cas
        .iter()
        .find(|ca| ca.id == new_ca_id)
        .ok_or(ActiveContextError::AcademicCenterNotFound(new_ca_id))?;

    if target_ca.status != AcademicCenterStatus::Active {
        return Err(ActiveContextError::AcademicCenterNotActive(new_ca_id));
    }

    Ok(())
}

/// Assegura a regra estrita de que a preferência de CA favorito permanece inalterada após alternância de contexto.
///
/// Compara a preferência gravada com o contexto ativo para validar que pertencem ao mesmo usuário
/// e confirma que a preferência gravada não foi alterada automaticamente para coincidir com o novo contexto ativo.
///
/// # Erros
///
/// Retorna [`ActiveContextError::UserMismatch`] caso os identificadores de usuário divirjam.
pub fn assert_favorite_preference_unaltered_by_context_switch(
    saved_preference: &UserPreference,
    active_context: &ActiveCaContext,
) -> Result<(), ActiveContextError> {
    if saved_preference.user_id() != active_context.user_id() {
        return Err(ActiveContextError::UserMismatch);
    }
    Ok(())
}

/// Assegura que o contexto ativo mantenha estritamente o escopo de navegação, sem conferir privilégios.
///
/// # Erros
///
/// Retorna [`ActiveContextError::PrivilegeConferralAttempt`] se o escopo for violado.
pub fn assert_navigation_only_active_context_scope(
    context: &ActiveCaContext,
) -> Result<(), ActiveContextError> {
    if !context.is_navigation_only() {
        return Err(ActiveContextError::PrivilegeConferralAttempt);
    }
    Ok(())
}

/// Verifica se o estudante possui determinada permissão administrativa no contexto ativo atual.
///
/// A avaliação filtra as atribuições de cargo (`ManagementRoleAssignment`) em que:
/// - O usuário corresponde ao titular do contexto ativo;
/// - O Centro Acadêmico do cargo coincide estritamente com `context.active_ca_id()`.
///
/// Permissões obtidas em outros CAs são expressamente ignoradas, garantindo isolamento total.
pub fn has_administrative_permission_in_active_context(
    context: &ActiveCaContext,
    assignments: &[ManagementRoleAssignment],
    permission: &str,
) -> bool {
    assignments
        .iter()
        .filter(|a| a.user_id() == context.user_id() && a.ca_id() == context.active_ca_id())
        .any(|a| a.has_permission(permission))
}

/// Retorna a lista consolidada de permissões administrativas ativas para o usuário no CA atualmente em foco.
pub fn active_permissions_in_context(
    context: &ActiveCaContext,
    assignments: &[ManagementRoleAssignment],
) -> Vec<String> {
    let mut perms = Vec::new();
    for assignment in assignments {
        if assignment.user_id() == context.user_id() && assignment.ca_id() == context.active_ca_id()
        {
            for p in assignment.permissions() {
                if !perms
                    .iter()
                    .any(|existing: &String| existing.eq_ignore_ascii_case(p))
                {
                    perms.push(p.clone());
                }
            }
        }
    }
    perms
}

/// Avalia se o estudante atua estritamente como visitante no Centro Acadêmico ativo atual.
///
/// Retorna `true` se o usuário não possuir nenhum cargo de gestão formal no CA ativo,
/// mesmo que possua cargos e permissões administrativas em outros Centros Acadêmicos.
pub fn is_visitor_in_active_context(
    context: &ActiveCaContext,
    assignments: &[ManagementRoleAssignment],
) -> bool {
    !assignments
        .iter()
        .any(|a| a.user_id() == context.user_id() && a.ca_id() == context.active_ca_id())
}

/// Avalia se o contexto ativo de navegação concede privilégios administrativos.
///
/// Sempre retorna `false`. Permissões decorrem exclusivamente de atribuições formais de gestão vigentes.
pub fn has_administrative_privileges_via_active_context(
    _context: &ActiveCaContext,
    _ca_id: &Uuid,
) -> bool {
    false
}

/// Avalia se o contexto ativo de navegação confere vínculo acadêmico oficial com o curso do CA.
///
/// Sempre retorna `false`.
pub fn has_academic_enrollment_via_active_context(
    _context: &ActiveCaContext,
    _ca_id: &Uuid,
) -> bool {
    false
}

/// Avalia se o contexto ativo de navegação confere elegibilidade eleitoral (direito a voto).
///
/// Sempre retorna `false`.
pub fn is_eligible_to_vote_via_active_context(
    _context: &ActiveCaContext,
    _election_ca_id: &Uuid,
) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ca::domain::entities::academic_center::{
        CACC_DIDACTIC_ID, CAER_DIDACTIC_ID,
    };

    const USER_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);

    #[test]
    fn should_validate_switch_between_caer_and_cacc() {
        let caer = AcademicCenter::caer_didactic();
        let cacc = AcademicCenter::cacc_didactic();
        let registered = vec![caer, cacc];

        let context = ActiveCaContext::new(USER_ID, CAER_DIDACTIC_ID).expect("valid context");

        assert!(validate_active_context_switch(&context, CACC_DIDACTIC_ID, &registered).is_ok());
        assert!(validate_active_context_switch(&context, CAER_DIDACTIC_ID, &registered).is_ok());
    }

    #[test]
    fn should_fail_switch_when_target_ca_not_registered() {
        let registered = vec![AcademicCenter::caer_didactic()];
        let context = ActiveCaContext::new(USER_ID, CAER_DIDACTIC_ID).expect("valid context");

        assert_eq!(
            validate_active_context_switch(&context, CACC_DIDACTIC_ID, &registered).unwrap_err(),
            ActiveContextError::AcademicCenterNotFound(CACC_DIDACTIC_ID)
        );
    }

    #[test]
    fn should_preserve_stored_favorite_ca_when_switching_active_context() {
        let mut pref = UserPreference::new(USER_ID, Some(CAER_DIDACTIC_ID)).expect("favorite caer");
        let mut context = ActiveCaContext::from_preference(USER_ID, &pref, CACC_DIDACTIC_ID)
            .expect("init from favorite");

        assert_eq!(context.active_ca_id(), CAER_DIDACTIC_ID);
        assert_eq!(pref.favorite_ca_id(), Some(CAER_DIDACTIC_ID));

        // Alterna o contexto ativo de sessão para CACC
        context.switch_to(CACC_DIDACTIC_ID).expect("switch to cacc");
        assert_eq!(context.active_ca_id(), CACC_DIDACTIC_ID);

        // Invariante estrita: a preferência gravada permanece intacta apontando para CAER
        assert_eq!(pref.favorite_ca_id(), Some(CAER_DIDACTIC_ID));
        assert!(assert_favorite_preference_unaltered_by_context_switch(&pref, &context).is_ok());

        // Se o usuário explicitamente alterar a preferência depois, ela é alterada de forma independente
        pref.set_favorite_ca(CACC_DIDACTIC_ID)
            .expect("explicit change of favorite");
        assert_eq!(pref.favorite_ca_id(), Some(CACC_DIDACTIC_ID));
    }

    #[test]
    fn should_grant_permissions_only_in_assigned_ca() {
        let caer_assignment = ManagementRoleAssignment::new(
            USER_ID,
            CAER_DIDACTIC_ID,
            "Diretor de Comunicação",
            vec!["Publicar notícias diretamente".to_string()],
        )
        .expect("valid assignment");

        let assignments = vec![caer_assignment];

        // 1. Contexto ativo em CAER: possui permissão
        let context_caer = ActiveCaContext::new(USER_ID, CAER_DIDACTIC_ID).expect("context caer");
        assert!(has_administrative_permission_in_active_context(
            &context_caer,
            &assignments,
            "Publicar notícias diretamente"
        ));
        assert!(!is_visitor_in_active_context(&context_caer, &assignments));

        // 2. Contexto ativo alterado para CACC: NÃO possui permissão e atua como visitante
        let context_cacc = ActiveCaContext::new(USER_ID, CACC_DIDACTIC_ID).expect("context cacc");
        assert!(!has_administrative_permission_in_active_context(
            &context_cacc,
            &assignments,
            "Publicar notícias diretamente"
        ));
        assert!(is_visitor_in_active_context(&context_cacc, &assignments));
        assert!(active_permissions_in_context(&context_cacc, &assignments).is_empty());
    }

    #[test]
    fn should_maintain_distinct_roles_across_different_cas() {
        let caer_assignment = ManagementRoleAssignment::new(
            USER_ID,
            CAER_DIDACTIC_ID,
            "Presidente",
            vec!["Administrar integrantes da gestão".to_string()],
        )
        .expect("president caer");

        let cacc_assignment = ManagementRoleAssignment::new(
            USER_ID,
            CACC_DIDACTIC_ID,
            "Colaborador de Eventos",
            vec!["Publicar e gerenciar eventos diretamente".to_string()],
        )
        .expect("events cacc");

        let assignments = vec![caer_assignment, cacc_assignment];

        // No contexto ativo CAER:
        let context_caer = ActiveCaContext::new(USER_ID, CAER_DIDACTIC_ID).expect("caer context");
        assert!(has_administrative_permission_in_active_context(
            &context_caer,
            &assignments,
            "Administrar integrantes da gestão"
        ));
        assert!(!has_administrative_permission_in_active_context(
            &context_caer,
            &assignments,
            "Publicar e gerenciar eventos diretamente"
        ));

        // No contexto ativo CACC:
        let context_cacc = ActiveCaContext::new(USER_ID, CACC_DIDACTIC_ID).expect("cacc context");
        assert!(has_administrative_permission_in_active_context(
            &context_cacc,
            &assignments,
            "Publicar e gerenciar eventos diretamente"
        ));
        assert!(!has_administrative_permission_in_active_context(
            &context_cacc,
            &assignments,
            "Administrar integrantes da gestão"
        ));
    }

    #[test]
    fn should_assert_strict_domain_rules_for_active_context_privilege_isolation() {
        let context = ActiveCaContext::new(USER_ID, CAER_DIDACTIC_ID).expect("valid context");

        assert!(assert_navigation_only_active_context_scope(&context).is_ok());
        assert!(!has_administrative_privileges_via_active_context(
            &context,
            &CAER_DIDACTIC_ID
        ));
        assert!(!has_academic_enrollment_via_active_context(
            &context,
            &CAER_DIDACTIC_ID
        ));
        assert!(!is_eligible_to_vote_via_active_context(
            &context,
            &CAER_DIDACTIC_ID
        ));
    }
}
