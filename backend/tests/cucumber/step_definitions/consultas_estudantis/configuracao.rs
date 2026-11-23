//! Configuração e encerramento de consultas estudantis.
//!
//! Steps exclusivamente pendentes; não simulam estado nem comportamento do produto.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: que o usuário integra a gestão do Centro Acadêmico ativo.
#[given("que o usuário integra a gestão do Centro Acadêmico ativo")]
fn user_is_board_member(_world: &mut AppWorld) {
    pending_step("que o usuário integra a gestão do Centro Acadêmico ativo");
}

/// Pendente: possui a permissão {string} nesse Centro Acadêmico.
#[given(expr = "possui a permissão {string} nesse Centro Acadêmico")]
fn user_has_permission(_world: &mut AppWorld, _value_1: String) {
    pending_step("possui a permissão {string} nesse Centro Acadêmico");
}

/// Pendente: ele cadastra uma consulta estudantil com enquete e os seguintes dados:.
#[when("ele cadastra uma consulta estudantil com enquete e os seguintes dados:")]
fn user_creates_consultation_with_poll(_world: &mut AppWorld) {
    pending_step("ele cadastra uma consulta estudantil com enquete e os seguintes dados:");
}

/// Pendente: a consulta é registrada no Centro Acadêmico ativo.
#[then("a consulta é registrada no Centro Acadêmico ativo")]
fn consultation_belongs_to_active_center(_world: &mut AppWorld) {
    pending_step("a consulta é registrada no Centro Acadêmico ativo");
}

/// Pendente: preserva a proposta, o período e as opções de voto informados.
#[then("preserva a proposta, o período e as opções de voto informados")]
fn consultation_preserves_configuration(_world: &mut AppWorld) {
    pending_step("preserva a proposta, o período e as opções de voto informados");
}

/// Pendente: não possui a permissão {string} nesse Centro Acadêmico.
#[given(expr = "não possui a permissão {string} nesse Centro Acadêmico")]
fn user_lacks_permission(_world: &mut AppWorld, _value_1: String) {
    pending_step("não possui a permissão {string} nesse Centro Acadêmico");
}

/// Pendente: ele tenta cadastrar uma consulta estudantil nesse Centro Acadêmico.
#[when("ele tenta cadastrar uma consulta estudantil nesse Centro Acadêmico")]
fn user_attempts_creation_in_active_center(_world: &mut AppWorld) {
    pending_step("ele tenta cadastrar uma consulta estudantil nesse Centro Acadêmico");
}

/// Pendente: a criação da consulta é recusada por falta da permissão {string}.
#[then(expr = "a criação da consulta é recusada por falta da permissão {string}")]
fn creation_is_denied(_world: &mut AppWorld, _value_1: String) {
    pending_step("a criação da consulta é recusada por falta da permissão {string}");
}

/// Pendente: nenhuma nova consulta é registrada nesse Centro Acadêmico.
#[then("nenhuma nova consulta é registrada nesse Centro Acadêmico")]
fn no_consultation_is_created(_world: &mut AppWorld) {
    pending_step("nenhuma nova consulta é registrada nesse Centro Acadêmico");
}

/// Pendente: que o usuário é estudante e não integra a gestão do Centro Acadêmico ativo.
#[given("que o usuário é estudante e não integra a gestão do Centro Acadêmico ativo")]
fn student_is_not_board_member(_world: &mut AppWorld) {
    pending_step("que o usuário é estudante e não integra a gestão do Centro Acadêmico ativo");
}

/// Pendente: que o usuário possui a permissão {string} somente no {string}.
#[given(expr = "que o usuário possui a permissão {string} somente no {string}")]
fn user_has_permission_only_in_other_center(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que o usuário possui a permissão {string} somente no {string}");
}

/// Pendente: ele tenta cadastrar uma consulta estudantil no {string}.
#[when(expr = "ele tenta cadastrar uma consulta estudantil no {string}")]
fn user_attempts_creation_in_named_center(_world: &mut AppWorld, _value_1: String) {
    pending_step("ele tenta cadastrar uma consulta estudantil no {string}");
}

/// Pendente: ele cadastra uma consulta com proposta e chat habilitado, sem enquete.
#[when("ele cadastra uma consulta com proposta e chat habilitado, sem enquete")]
fn user_creates_discussion_without_poll(_world: &mut AppWorld) {
    pending_step("ele cadastra uma consulta com proposta e chat habilitado, sem enquete");
}

/// Pendente: nenhuma opção de voto é exigida para essa consulta.
#[then("nenhuma opção de voto é exigida para essa consulta")]
fn poll_options_are_not_required(_world: &mut AppWorld) {
    pending_step("nenhuma opção de voto é exigida para essa consulta");
}

/// Pendente: a consulta oferece discussão pelo chat, sem votação.
#[then("a consulta oferece discussão pelo chat, sem votação")]
fn consultation_offers_chat_without_poll(_world: &mut AppWorld) {
    pending_step("a consulta oferece discussão pelo chat, sem votação");
}

/// Pendente: ele cadastra uma consulta com proposta e enquete, sem habilitar o chat.
#[when("ele cadastra uma consulta com proposta e enquete, sem habilitar o chat")]
fn user_creates_poll_without_chat(_world: &mut AppWorld) {
    pending_step("ele cadastra uma consulta com proposta e enquete, sem habilitar o chat");
}

/// Pendente: a consulta oferece votação, sem chat.
#[then("a consulta oferece votação, sem chat")]
fn consultation_offers_poll_without_chat(_world: &mut AppWorld) {
    pending_step("a consulta oferece votação, sem chat");
}

/// Pendente: ele cadastra uma consulta com proposta, enquete e chat habilitado.
#[when("ele cadastra uma consulta com proposta, enquete e chat habilitado")]
fn user_creates_poll_and_chat(_world: &mut AppWorld) {
    pending_step("ele cadastra uma consulta com proposta, enquete e chat habilitado");
}

/// Pendente: a enquete e o chat estão associados à mesma proposta.
#[then("a enquete e o chat estão associados à mesma proposta")]
fn poll_and_chat_share_proposal(_world: &mut AppWorld) {
    pending_step("a enquete e o chat estão associados à mesma proposta");
}

/// Pendente: que um integrante autorizado da gestão está configurando uma consulta com enquete e chat.
#[given("que um integrante autorizado da gestão está configurando uma consulta com enquete e chat")]
fn authorized_member_configures_consultation(_world: &mut AppWorld) {
    pending_step(
        "que um integrante autorizado da gestão está configurando uma consulta com enquete e chat",
    );
}

/// Pendente: ele define separadamente quem pode acessar a consulta, participar do chat, votar e visualizar resultados.
#[when(
    "ele define separadamente quem pode acessar a consulta, participar do chat, votar e visualizar resultados"
)]
fn member_sets_independent_audiences(_world: &mut AppWorld) {
    pending_step(
        "ele define separadamente quem pode acessar a consulta, participar do chat, votar e visualizar resultados",
    );
}

/// Pendente: cada público informado é salvo para a finalidade correspondente.
#[then("cada público informado é salvo para a finalidade correspondente")]
fn audiences_are_saved_independently(_world: &mut AppWorld) {
    pending_step("cada público informado é salvo para a finalidade correspondente");
}

/// Pendente: configurar um desses públicos não concede automaticamente as demais permissões.
#[then("configurar um desses públicos não concede automaticamente as demais permissões")]
fn audience_does_not_grant_other_permissions(_world: &mut AppWorld) {
    pending_step("configurar um desses públicos não concede automaticamente as demais permissões");
}

/// Pendente: que o usuário integra a gestão e possui permissão para encerrar consultas nesse Centro Acadêmico.
#[given(
    "que o usuário integra a gestão e possui permissão para encerrar consultas nesse Centro Acadêmico"
)]
fn board_member_can_close_consultation(_world: &mut AppWorld) {
    pending_step(
        "que o usuário integra a gestão e possui permissão para encerrar consultas nesse Centro Acadêmico",
    );
}

/// Pendente: ele solicita o encerramento antecipado da consulta.
#[when("ele solicita o encerramento antecipado da consulta")]
fn user_requests_early_closure(_world: &mut AppWorld) {
    pending_step("ele solicita o encerramento antecipado da consulta");
}

/// Pendente: a consulta é encerrada.
#[then("a consulta é encerrada")]
fn consultation_is_closed(_world: &mut AppWorld) {
    pending_step("a consulta é encerrada");
}

/// Pendente: novos votos nessa enquete são recusados.
#[then("novos votos nessa enquete são recusados")]
fn poll_rejects_new_votes(_world: &mut AppWorld) {
    pending_step("novos votos nessa enquete são recusados");
}

/// Pendente: que o usuário integra a gestão, mas não possui permissão para encerrar consultas nesse Centro Acadêmico.
#[given(
    "que o usuário integra a gestão, mas não possui permissão para encerrar consultas nesse Centro Acadêmico"
)]
fn board_member_cannot_close_consultation(_world: &mut AppWorld) {
    pending_step(
        "que o usuário integra a gestão, mas não possui permissão para encerrar consultas nesse Centro Acadêmico",
    );
}

/// Pendente: o encerramento é recusado por falta de permissão.
#[then("o encerramento é recusado por falta de permissão")]
fn closure_is_denied(_world: &mut AppWorld) {
    pending_step("o encerramento é recusado por falta de permissão");
}

/// Pendente: a consulta permanece aberta.
#[then("a consulta permanece aberta")]
fn consultation_remains_open(_world: &mut AppWorld) {
    pending_step("a consulta permanece aberta");
}

/// Pendente: ele cadastra uma consulta com proposta e chat, sem enquete e sem prazo de encerramento.
#[when("ele cadastra uma consulta com proposta e chat, sem enquete e sem prazo de encerramento")]
fn user_creates_chat_without_poll_or_deadline(_world: &mut AppWorld) {
    pending_step(
        "ele cadastra uma consulta com proposta e chat, sem enquete e sem prazo de encerramento",
    );
}

/// Pendente: ela permanece aberta até que seja encerrada por um integrante autorizado da gestão.
#[then("ela permanece aberta até que seja encerrada por um integrante autorizado da gestão")]
fn consultation_stays_open_until_authorized_closure(_world: &mut AppWorld) {
    pending_step(
        "ela permanece aberta até que seja encerrada por um integrante autorizado da gestão",
    );
}

/// Pendente: que existe uma consulta aberta com chat habilitado, sem enquete e sem prazo de encerramento.
#[given(
    "que existe uma consulta aberta com chat habilitado, sem enquete e sem prazo de encerramento"
)]
fn open_chat_has_no_poll_or_deadline(_world: &mut AppWorld) {
    pending_step(
        "que existe uma consulta aberta com chat habilitado, sem enquete e sem prazo de encerramento",
    );
}

/// Pendente: ele solicita o encerramento da consulta.
#[when("ele solicita o encerramento da consulta")]
fn user_requests_closure(_world: &mut AppWorld) {
    pending_step("ele solicita o encerramento da consulta");
}

/// Pendente: o chat continua habilitado para o público autorizado.
#[then("o chat continua habilitado para o público autorizado")]
fn chat_remains_enabled(_world: &mut AppWorld) {
    pending_step("o chat continua habilitado para o público autorizado");
}

/// Pendente: que a consulta tem encerramento previsto para daqui a 1 hora.
#[given("que a consulta tem encerramento previsto para daqui a 1 hora")]
fn consultation_deadline_is_in_one_hour(_world: &mut AppWorld) {
    pending_step("que a consulta tem encerramento previsto para daqui a 1 hora");
}

/// Pendente: o prazo de encerramento da consulta é atingido.
#[when("o prazo de encerramento da consulta é atingido")]
fn consultation_deadline_is_reached(_world: &mut AppWorld) {
    pending_step("o prazo de encerramento da consulta é atingido");
}
