//! Steps pendentes da especificação de administração dos integrantes da gestão (#62).
//!
//! Registra todos os padrões Gherkin de
//! `features/gestao_ca/integrantes_da_gestao.feature`. Enquanto a persistência e
//! os casos de uso do módulo não estão conectados ao [`AppWorld`], cada step
//! falha explicitamente por meio de [`pending_step`].

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

// ============================================================================
// Contexto (Pré-condições de CA, cargos, permissões e integrantes)
// ============================================================================

/// Pendente: que o Centro Acadêmico {string} está cadastrado na UnB.
#[given(expr = "que o Centro Acadêmico {string} está cadastrado na UnB")]
async fn academic_center_is_registered(_world: &mut AppWorld, _ca_name: String) {
    pending_step("que o Centro Acadêmico {string} está cadastrado na UnB");
}

/// Pendente: o CA {string} possui os cargos {string}, {string} e {string}.
#[given(expr = "o CA {string} possui os cargos {string}, {string} e {string}")]
async fn academic_center_has_roles(
    _world: &mut AppWorld,
    _ca: String,
    _first_role: String,
    _second_role: String,
    _third_role: String,
) {
    pending_step("o CA {string} possui os cargos {string}, {string} e {string}");
}

/// Pendente: o CA {string} não possui o cargo {string}.
#[given(expr = "o CA {string} não possui o cargo {string}")]
async fn academic_center_does_not_have_role(_world: &mut AppWorld, _ca: String, _role: String) {
    pending_step("o CA {string} não possui o cargo {string}");
}

/// Pendente: o cargo {string} possui a permissão de {string}.
#[given(expr = "que o cargo {string} possui a permissão de {string}")]
#[given(expr = "o cargo {string} possui a permissão de {string}")]
async fn role_has_permission(_world: &mut AppWorld, _role: String, _permission: String) {
    pending_step("o cargo {string} possui a permissão de {string}");
}

/// Pendente: o cargo {string} não possui a permissão de {string}.
#[given(expr = "o cargo {string} não possui a permissão de {string}")]
async fn role_does_not_have_permission(_world: &mut AppWorld, _role: String, _permission: String) {
    pending_step("o cargo {string} não possui a permissão de {string}");
}

/// Pendente: que o estudante {string} possui perfil no CApp.
#[given(expr = "que o estudante {string} possui perfil no CApp")]
#[given(expr = "o estudante {string} possui perfil no CApp")]
async fn student_has_profile(_world: &mut AppWorld, _student_email: String) {
    pending_step("que o estudante {string} possui perfil no CApp");
}

/// Pendente: que não existe perfil no CApp para o e-mail {string}.
#[given(expr = "que não existe perfil no CApp para o e-mail {string}")]
async fn profile_does_not_exist(_world: &mut AppWorld, _email: String) {
    pending_step("que não existe perfil no CApp para o e-mail {string}");
}

/// Pendente: o estudante {string} exerce o cargo de {string} em {string}.
#[given(expr = "que o estudante {string} exerce o cargo de {string} em {string}")]
#[given(expr = "o estudante {string} exerce o cargo de {string} em {string}")]
async fn student_holds_role_in_ca(
    _world: &mut AppWorld,
    _student_email: String,
    _role: String,
    _ca: String,
) {
    pending_step("o estudante {string} exerce o cargo de {string} em {string}");
}

/// Pendente: não integra a gestão de {string}.
#[given(expr = "não integra a gestão de {string}")]
async fn student_is_not_in_management(_world: &mut AppWorld, _ca: String) {
    pending_step("não integra a gestão de {string}");
}

/// Pendente: o estudante {string} não possui cargo de gestão em {string}.
#[given(expr = "o estudante {string} não possui cargo de gestão em {string}")]
async fn student_has_no_management_role_in_ca(
    _world: &mut AppWorld,
    _student_email: String,
    _ca: String,
) {
    pending_step("o estudante {string} não possui cargo de gestão em {string}");
}

/// Pendente: que {string} é o único integrante de {string} com a permissão de {string}.
#[given(expr = "que {string} é o único integrante de {string} com a permissão de {string}")]
async fn student_is_sole_member_with_permission(
    _world: &mut AppWorld,
    _student_email: String,
    _ca: String,
    _permission: String,
) {
    pending_step("que {string} é o único integrante de {string} com a permissão de {string}");
}

// ============================================================================
// Ações (Vinculação, alteração e remoção de integrantes)
// ============================================================================

/// Pendente: o gestor {string} vincula {string} ao cargo {string} em {string}.
#[when(expr = "o gestor {string} vincula {string} ao cargo {string} em {string}")]
async fn manager_assigns_student_to_role(
    _world: &mut AppWorld,
    _manager_email: String,
    _student_email: String,
    _role: String,
    _ca: String,
) {
    pending_step("o gestor {string} vincula {string} ao cargo {string} em {string}");
}

/// Pendente: tentativa de vincular um usuário a um cargo específico em um CA.
#[when(expr = "o gestor {string} tenta vincular {string} ao cargo {string} em {string}")]
#[when(expr = "{string} tenta vincular {string} ao cargo {string} em {string}")]
async fn actor_attempts_to_assign_student_to_role(
    _world: &mut AppWorld,
    _actor_email: String,
    _student_email: String,
    _role: String,
    _ca: String,
) {
    pending_step("o gestor {string} tenta vincular {string} ao cargo {string} em {string}");
}

/// Pendente: o gestor {string} tenta vincular {string} a um cargo em {string}.
#[when(expr = "o gestor {string} tenta vincular {string} a um cargo em {string}")]
async fn manager_attempts_to_assign_student_in_other_ca(
    _world: &mut AppWorld,
    _manager_email: String,
    _student_email: String,
    _ca: String,
) {
    pending_step("o gestor {string} tenta vincular {string} a um cargo em {string}");
}

/// Pendente: o gestor {string} altera o cargo de {string} para {string} em {string}.
#[when(expr = "o gestor {string} altera o cargo de {string} para {string} em {string}")]
async fn manager_changes_member_role(
    _world: &mut AppWorld,
    _manager_email: String,
    _student_email: String,
    _new_role: String,
    _ca: String,
) {
    pending_step("o gestor {string} altera o cargo de {string} para {string} em {string}");
}

/// Pendente: tentativa de alterar o cargo de um usuário em um CA.
#[when(expr = "o gestor {string} tenta alterar o cargo de {string} para {string} em {string}")]
#[when(expr = "{string} tenta alterar o cargo de {string} para {string} em {string}")]
async fn actor_attempts_to_change_member_role(
    _world: &mut AppWorld,
    _actor_email: String,
    _student_email: String,
    _new_role: String,
    _ca: String,
) {
    pending_step("o gestor {string} tenta alterar o cargo de {string} para {string} em {string}");
}

/// Pendente: o gestor {string} remove {string} da gestão de {string}.
#[when(expr = "o gestor {string} remove {string} da gestão de {string}")]
async fn manager_removes_member_from_management(
    _world: &mut AppWorld,
    _manager_email: String,
    _student_email: String,
    _ca: String,
) {
    pending_step("o gestor {string} remove {string} da gestão de {string}");
}

/// Pendente: tentativa de remover um integrante da gestão de um CA.
#[when(expr = "o gestor {string} tenta remover {string} da gestão de {string}")]
#[when(expr = "{string} tenta remover {string} da gestão de {string}")]
async fn actor_attempts_to_remove_member_from_management(
    _world: &mut AppWorld,
    _actor_email: String,
    _student_email: String,
    _ca: String,
) {
    pending_step("o gestor {string} tenta remover {string} da gestão de {string}");
}

// ============================================================================
// Verificações (Composição da gestão, cargos, permissões e recusas)
// ============================================================================

/// Pendente: {string} passa a integrar a gestão de {string} no cargo {string}.
#[then(expr = "{string} passa a integrar a gestão de {string} no cargo {string}")]
async fn student_joins_management_in_role(
    _world: &mut AppWorld,
    _student_email: String,
    _ca: String,
    _role: String,
) {
    pending_step("{string} passa a integrar a gestão de {string} no cargo {string}");
}

/// Pendente: {string} passa a exercer o cargo {string} em {string}.
#[then(expr = "{string} passa a exercer o cargo {string} em {string}")]
async fn student_starts_holding_role(
    _world: &mut AppWorld,
    _student_email: String,
    _role: String,
    _ca: String,
) {
    pending_step("{string} passa a exercer o cargo {string} em {string}");
}

/// Pendente: {string} permanece no cargo {string} em {string}.
#[then(expr = "{string} permanece somente no cargo {string} em {string}")]
#[then(expr = "{string} permanece no cargo {string} em {string}")]
async fn student_remains_in_role(
    _world: &mut AppWorld,
    _student_email: String,
    _role: String,
    _ca: String,
) {
    pending_step("{string} permanece no cargo {string} em {string}");
}

/// Pendente: {string} não passa a integrar ou permanece fora da gestão de {string}.
#[then(expr = "{string} não passa a integrar a gestão de {string}")]
#[then(expr = "{string} permanece sem integrar a gestão de {string}")]
async fn student_remains_outside_management(
    _world: &mut AppWorld,
    _student_email: String,
    _ca: String,
) {
    pending_step("{string} não passa a integrar a gestão de {string}");
}

/// Pendente: {string} deixa de integrar a gestão de {string}.
#[then(expr = "{string} deixa de integrar a gestão de {string}")]
async fn student_leaves_management(_world: &mut AppWorld, _student_email: String, _ca: String) {
    pending_step("{string} deixa de integrar a gestão de {string}");
}

/// Pendente: a composição da gestão de {string} permanece inalterada.
#[then(expr = "a composição da gestão de {string} permanece inalterada")]
async fn management_composition_remains_unchanged(_world: &mut AppWorld, _ca: String) {
    pending_step("a composição da gestão de {string} permanece inalterada");
}

/// Pendente: {string} passa a possuir as permissões do cargo {string} em {string}.
#[then(expr = "{string} passa a possuir as permissões do cargo {string} em {string}")]
async fn student_receives_role_permissions(
    _world: &mut AppWorld,
    _student_email: String,
    _role: String,
    _ca: String,
) {
    pending_step("{string} passa a possuir as permissões do cargo {string} em {string}");
}

/// Pendente: {string} deixa de possuir as permissões do cargo {string} em {string}.
#[then(expr = "{string} deixa de possuir as permissões do cargo {string} em {string}")]
async fn student_loses_role_permissions(
    _world: &mut AppWorld,
    _student_email: String,
    _role: String,
    _ca: String,
) {
    pending_step("{string} deixa de possuir as permissões do cargo {string} em {string}");
}

/// Pendente: {string} passa ou permanece com a permissão de {string} em {string}.
#[then(expr = "{string} passa a possuir a permissão de {string} em {string}")]
#[then(expr = "{string} permanece com a permissão de {string} em {string}")]
async fn student_has_specific_permission_in_ca(
    _world: &mut AppWorld,
    _student_email: String,
    _permission: String,
    _ca: String,
) {
    pending_step("{string} passa a possuir a permissão de {string} em {string}");
}

/// Pendente: {string} deixa de possuir a permissão de {string} em {string}.
#[then(expr = "{string} deixa de possuir a permissão de {string} em {string}")]
async fn student_loses_specific_permission_in_ca(
    _world: &mut AppWorld,
    _student_email: String,
    _permission: String,
    _ca: String,
) {
    pending_step("{string} deixa de possuir a permissão de {string} em {string}");
}

/// Pendente: {string} continua podendo acessar {string} como estudante.
#[then(expr = "{string} continua podendo acessar {string} como estudante")]
async fn student_retains_regular_access_to_ca(
    _world: &mut AppWorld,
    _student_email: String,
    _ca: String,
) {
    pending_step("{string} continua podendo acessar {string} como estudante");
}

/// Pendente: a operação é recusada por falta de permissão.
#[then("a operação é recusada por falta de permissão")]
async fn operation_is_rejected_for_lack_of_permission(_world: &mut AppWorld) {
    pending_step("a operação é recusada por falta de permissão");
}

/// Pendente: a operação é recusada informando que o usuário não foi encontrado.
#[then("a operação é recusada informando que o usuário não foi encontrado")]
async fn operation_is_rejected_user_not_found(_world: &mut AppWorld) {
    pending_step("a operação é recusada informando que o usuário não foi encontrado");
}

/// Pendente: a operação é recusada informando que o cargo não existe no CA.
#[then("a operação é recusada informando que o cargo não existe no CA")]
async fn operation_is_rejected_role_not_found(_world: &mut AppWorld) {
    pending_step("a operação é recusada informando que o cargo não existe no CA");
}

/// Pendente: a operação é recusada informando que o usuário já integra a gestão do CA.
#[then("a operação é recusada informando que o usuário já integra a gestão do CA")]
async fn operation_is_rejected_user_already_in_management(_world: &mut AppWorld) {
    pending_step("a operação é recusada informando que o usuário já integra a gestão do CA");
}

/// Pendente: a operação é recusada informando que o usuário não integra a gestão do CA.
#[then("a operação é recusada informando que o usuário não integra a gestão do CA")]
async fn operation_is_rejected_user_not_in_management(_world: &mut AppWorld) {
    pending_step("a operação é recusada informando que o usuário não integra a gestão do CA");
}

/// Pendente: a operação é recusada informando que o integrante já exerce esse cargo.
#[then("a operação é recusada informando que o integrante já exerce esse cargo")]
async fn operation_is_rejected_member_already_holds_role(_world: &mut AppWorld) {
    pending_step("a operação é recusada informando que o integrante já exerce esse cargo");
}

/// Pendente: a operação é recusada informando que a gestão precisa manter ao menos um integrante com essa permissão.
#[then(
    "a operação é recusada informando que a gestão precisa manter ao menos um integrante com essa permissão"
)]
async fn operation_is_rejected_last_manager_protection(_world: &mut AppWorld) {
    pending_step(
        "a operação é recusada informando que a gestão precisa manter ao menos um integrante com essa permissão",
    );
}

// ============================================================================
// Histórico (Contexto prévio, ações e verificações do histórico administrativo)
// ============================================================================

/// Pendente: que o estudante {string} foi vinculado ao cargo {string} em {string}.
#[given(expr = "que o estudante {string} foi vinculado ao cargo {string} em {string}")]
async fn student_was_previously_assigned_to_role(
    _world: &mut AppWorld,
    _student_email: String,
    _role: String,
    _ca: String,
) {
    pending_step("que o estudante {string} foi vinculado ao cargo {string} em {string}");
}

/// Pendente: posteriormente teve o cargo alterado para {string} em {string}.
#[given(expr = "posteriormente teve o cargo alterado para {string} em {string}")]
async fn student_subsequently_had_role_changed(
    _world: &mut AppWorld,
    _new_role: String,
    _ca: String,
) {
    pending_step("posteriormente teve o cargo alterado para {string} em {string}");
}

/// Pendente: que o histórico de {string} registra a remoção de {string} do cargo {string}.
#[given(expr = "que o histórico de {string} registra a remoção de {string} do cargo {string}")]
async fn history_contains_removal_record(
    _world: &mut AppWorld,
    _ca: String,
    _student_email: String,
    _role: String,
) {
    pending_step("que o histórico de {string} registra a remoção de {string} do cargo {string}");
}

/// Pendente: que o histórico de {string} registra a vinculação de {string} ao cargo {string}.
#[given(expr = "que o histórico de {string} registra a vinculação de {string} ao cargo {string}")]
async fn history_contains_assignment_record(
    _world: &mut AppWorld,
    _ca: String,
    _student_email: String,
    _role: String,
) {
    pending_step("que o histórico de {string} registra a vinculação de {string} ao cargo {string}");
}

/// Pendente: o gestor {string} tenta {word} esse registro do histórico.
#[when(expr = "o gestor {string} tenta {word} esse registro do histórico")]
async fn manager_attempts_to_modify_history_record(
    _world: &mut AppWorld,
    _manager_email: String,
    _action: String,
) {
    pending_step("o gestor {string} tenta {word} esse registro do histórico");
}

/// Pendente: o histórico administrativo de {string} registra a vinculação de {string} ao cargo {string}.
#[then(
    expr = "o histórico administrativo de {string} registra a vinculação de {string} ao cargo {string}"
)]
async fn history_records_member_assignment(
    _world: &mut AppWorld,
    _ca: String,
    _student_email: String,
    _role: String,
) {
    pending_step(
        "o histórico administrativo de {string} registra a vinculação de {string} ao cargo {string}",
    );
}

/// Pendente: o histórico administrativo de {string} registra a alteração de {string} do cargo {string} para {string}.
#[then(
    expr = "o histórico administrativo de {string} registra a alteração de {string} do cargo {string} para {string}"
)]
async fn history_records_role_change(
    _world: &mut AppWorld,
    _ca: String,
    _student_email: String,
    _old_role: String,
    _new_role: String,
) {
    pending_step(
        "o histórico administrativo de {string} registra a alteração de {string} do cargo {string} para {string}",
    );
}

/// Pendente: o histórico administrativo de {string} registra a remoção de {string} do cargo {string}.
#[then(
    expr = "o histórico administrativo de {string} registra a remoção de {string} do cargo {string}"
)]
async fn history_records_member_removal(
    _world: &mut AppWorld,
    _ca: String,
    _student_email: String,
    _role: String,
) {
    pending_step(
        "o histórico administrativo de {string} registra a remoção de {string} do cargo {string}",
    );
}

/// Pendente: o registro identifica o gestor {string} como responsável e a data da operação.
#[then(expr = "o registro identifica o gestor {string} como responsável e a data da vinculação")]
#[then(expr = "o registro identifica o gestor {string} como responsável e a data da alteração")]
#[then(expr = "o registro identifica o gestor {string} como responsável e a data da remoção")]
async fn history_record_identifies_responsible_manager_and_date(
    _world: &mut AppWorld,
    _manager_email: String,
) {
    pending_step("o registro identifica o gestor {string} como responsável e a data da operação");
}

/// Pendente: o histórico administrativo de {string} apresenta, em ordem cronológica, a vinculação, a alteração e a remoção de {string}.
#[then(
    expr = "o histórico administrativo de {string} apresenta, em ordem cronológica, a vinculação, a alteração e a remoção de {string}"
)]
async fn history_presents_chronological_lifecycle(
    _world: &mut AppWorld,
    _ca: String,
    _student_email: String,
) {
    pending_step(
        "o histórico administrativo de {string} apresenta, em ordem cronológica, a vinculação, a alteração e a remoção de {string}",
    );
}

/// Pendente: os registros anteriores à remoção permanecem inalterados.
#[then("os registros anteriores à remoção permanecem inalterados")]
async fn records_prior_to_removal_remain_unchanged(_world: &mut AppWorld) {
    pending_step("os registros anteriores à remoção permanecem inalterados");
}

/// Pendente: o histórico administrativo de {string} registra a nova vinculação.
#[then(expr = "o histórico administrativo de {string} registra a nova vinculação")]
async fn history_records_new_assignment(_world: &mut AppWorld, _ca: String) {
    pending_step("o histórico administrativo de {string} registra a nova vinculação");
}

/// Pendente: os registros da passagem anterior de {string} pela gestão permanecem disponíveis.
#[then(expr = "os registros da passagem anterior de {string} pela gestão permanecem disponíveis")]
async fn previous_tenure_records_remain_available(_world: &mut AppWorld, _student_email: String) {
    pending_step(
        "os registros da passagem anterior de {string} pela gestão permanecem disponíveis",
    );
}

/// Pendente: a operação é recusada informando que o histórico administrativo não pode ser modificado.
#[then("a operação é recusada informando que o histórico administrativo não pode ser modificado")]
async fn operation_is_rejected_history_is_immutable(_world: &mut AppWorld) {
    pending_step(
        "a operação é recusada informando que o histórico administrativo não pode ser modificado",
    );
}

/// Pendente: o registro permanece inalterado no histórico de {string}.
#[then(expr = "o registro permanece inalterado no histórico de {string}")]
async fn history_record_remains_unchanged(_world: &mut AppWorld, _ca: String) {
    pending_step("o registro permanece inalterado no histórico de {string}");
}

/// Pendente: nenhum novo registro é adicionado ao histórico administrativo de {string}.
#[then(expr = "nenhum novo registro é adicionado ao histórico administrativo de {string}")]
async fn no_new_record_is_added_to_history(_world: &mut AppWorld, _ca: String) {
    pending_step("nenhum novo registro é adicionado ao histórico administrativo de {string}");
}
