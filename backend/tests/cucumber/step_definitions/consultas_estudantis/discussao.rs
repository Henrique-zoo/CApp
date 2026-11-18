//! Discussão de propostas no chat das consultas estudantis.
//!
//! Steps exclusivamente pendentes; não simulam estado nem comportamento do produto.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: que existe uma consulta aberta com chat habilitado e sem enquete no Centro Acadêmico ativo.
#[given(
    "que existe uma consulta aberta com chat habilitado e sem enquete no Centro Acadêmico ativo"
)]
fn open_consultation_has_chat_without_poll(_world: &mut AppWorld) {
    pending_step(
        "que existe uma consulta aberta com chat habilitado e sem enquete no Centro Acadêmico ativo",
    );
}

/// Pendente: que o usuário pertence ao público de acesso à consulta, mas não ao público de participação no chat.
#[given(
    "que o usuário pertence ao público de acesso à consulta, mas não ao público de participação no chat"
)]
fn user_has_access_without_chat_permission(_world: &mut AppWorld) {
    pending_step(
        "que o usuário pertence ao público de acesso à consulta, mas não ao público de participação no chat",
    );
}

/// Pendente: ele tenta enviar uma mensagem no chat da consulta.
#[when("ele tenta enviar uma mensagem no chat da consulta")]
fn user_attempts_chat_message(_world: &mut AppWorld) {
    pending_step("ele tenta enviar uma mensagem no chat da consulta");
}

/// Pendente: o envio é recusado por falta de permissão para participar do chat.
#[then("o envio é recusado por falta de permissão para participar do chat")]
fn chat_participation_is_denied(_world: &mut AppWorld) {
    pending_step("o envio é recusado por falta de permissão para participar do chat");
}

/// Pendente: nenhuma mensagem dessa tentativa é publicada.
#[then("nenhuma mensagem dessa tentativa é publicada")]
fn no_chat_message_is_published(_world: &mut AppWorld) {
    pending_step("nenhuma mensagem dessa tentativa é publicada");
}

/// Pendente: que o usuário está autorizado a acessar essa consulta e participar do chat.
#[given("que o usuário está autorizado a acessar essa consulta e participar do chat")]
fn user_can_access_consultation_and_chat(_world: &mut AppWorld) {
    pending_step("que o usuário está autorizado a acessar essa consulta e participar do chat");
}

/// Pendente: ele envia a mensagem {string} no chat da consulta.
#[when(expr = "ele envia a mensagem {string} no chat da consulta")]
fn user_sends_chat_message(_world: &mut AppWorld, _value_1: String) {
    pending_step("ele envia a mensagem {string} no chat da consulta");
}

/// Pendente: a mensagem é publicada no chat associado à proposta.
#[then("a mensagem é publicada no chat associado à proposta")]
fn message_is_published_in_proposal_chat(_world: &mut AppWorld) {
    pending_step("a mensagem é publicada no chat associado à proposta");
}

/// Pendente: outros usuários autorizados a acessar o chat podem ler essa mensagem.
#[then("outros usuários autorizados a acessar o chat podem ler essa mensagem")]
fn authorized_users_can_read_message(_world: &mut AppWorld) {
    pending_step("outros usuários autorizados a acessar o chat podem ler essa mensagem");
}

/// Pendente: a publicação da mensagem não registra um voto.
#[then("a publicação da mensagem não registra um voto")]
fn message_does_not_register_vote(_world: &mut AppWorld) {
    pending_step("a publicação da mensagem não registra um voto");
}

/// Pendente: que o usuário não está autorizado a acessar essa consulta.
#[given("que o usuário não está autorizado a acessar essa consulta")]
fn user_cannot_access_consultation(_world: &mut AppWorld) {
    pending_step("que o usuário não está autorizado a acessar essa consulta");
}

/// Pendente: o envio é recusado por falta de acesso à consulta.
#[then("o envio é recusado por falta de acesso à consulta")]
fn chat_message_is_denied(_world: &mut AppWorld) {
    pending_step("o envio é recusado por falta de acesso à consulta");
}

/// Pendente: que uma consulta foi encerrada manualmente e possui chat habilitado.
#[given("que uma consulta foi encerrada manualmente e possui chat habilitado")]
fn consultation_was_closed_manually_with_chat(_world: &mut AppWorld) {
    pending_step("que uma consulta foi encerrada manualmente e possui chat habilitado");
}

/// Pendente: a consulta permanece encerrada.
#[then("a consulta permanece encerrada")]
fn consultation_remains_closed(_world: &mut AppWorld) {
    pending_step("a consulta permanece encerrada");
}

/// Pendente: que uma consulta foi encerrada pelo prazo e possui chat habilitado.
#[given("que uma consulta foi encerrada pelo prazo e possui chat habilitado")]
fn consultation_was_closed_by_deadline_with_chat(_world: &mut AppWorld) {
    pending_step("que uma consulta foi encerrada pelo prazo e possui chat habilitado");
}
