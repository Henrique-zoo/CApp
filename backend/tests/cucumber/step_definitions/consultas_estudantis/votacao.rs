//! Participação nas enquetes das consultas estudantis.
//!
//! Steps exclusivamente pendentes; não simulam estado nem comportamento do produto.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: que o estudante pode acessar a consulta e participar do chat, mas não pertence ao público de votação.
#[given(
    "que o estudante pode acessar a consulta e participar do chat, mas não pertence ao público de votação"
)]
fn student_has_chat_access_without_voting_permission(_world: &mut AppWorld) {
    pending_step(
        "que o estudante pode acessar a consulta e participar do chat, mas não pertence ao público de votação",
    );
}

/// Pendente: o estudante tenta confirmar um voto nessa consulta.
#[when("o estudante tenta confirmar um voto nessa consulta")]
fn student_attempts_to_vote(_world: &mut AppWorld) {
    pending_step("o estudante tenta confirmar um voto nessa consulta");
}

/// Pendente: o voto é recusado porque o estudante não pertence ao público elegível.
#[then("o voto é recusado porque o estudante não pertence ao público elegível")]
fn ineligible_vote_is_denied(_world: &mut AppWorld) {
    pending_step("o voto é recusado porque o estudante não pertence ao público elegível");
}

/// Pendente: nenhum voto dessa tentativa é registrado.
#[then("nenhum voto dessa tentativa é registrado")]
fn attempt_does_not_record_vote(_world: &mut AppWorld) {
    pending_step("nenhum voto dessa tentativa é registrado");
}

/// Pendente: o estudante abre essa consulta.
#[when("o estudante abre essa consulta")]
fn student_opens_consultation(_world: &mut AppWorld) {
    pending_step("o estudante abre essa consulta");
}

/// Pendente: ele visualiza a proposta apresentada pelo Centro Acadêmico.
#[then("ele visualiza a proposta apresentada pelo Centro Acadêmico")]
fn student_reads_proposal(_world: &mut AppWorld) {
    pending_step("ele visualiza a proposta apresentada pelo Centro Acadêmico");
}

/// Pendente: visualiza o período de votação e as opções de voto disponíveis.
#[then("visualiza o período de votação e as opções de voto disponíveis")]
fn student_reads_period_and_options(_world: &mut AppWorld) {
    pending_step("visualiza o período de votação e as opções de voto disponíveis");
}

/// Pendente: que ele ainda não votou nessa enquete.
#[given("que ele ainda não votou nessa enquete")]
fn student_has_not_voted(_world: &mut AppWorld) {
    pending_step("que ele ainda não votou nessa enquete");
}

/// Pendente: o estudante seleciona a opção {string} e confirma seu voto.
#[when(expr = "o estudante seleciona a opção {string} e confirma seu voto")]
fn student_confirms_selected_option(_world: &mut AppWorld, _value_1: String) {
    pending_step("o estudante seleciona a opção {string} e confirma seu voto");
}

/// Pendente: sua participação na consulta é confirmada.
#[then("sua participação na consulta é confirmada")]
fn participation_is_confirmed(_world: &mut AppWorld) {
    pending_step("sua participação na consulta é confirmada");
}

/// Pendente: a opção {string} recebe esse voto.
#[then(expr = "a opção {string} recebe esse voto")]
fn selected_option_receives_vote(_world: &mut AppWorld, _value_1: String) {
    pending_step("a opção {string} recebe esse voto");
}

/// Pendente: que o período de votação começa daqui a 1 dia.
#[given("que o período de votação começa daqui a 1 dia")]
fn voting_period_has_not_started(_world: &mut AppWorld) {
    pending_step("que o período de votação começa daqui a 1 dia");
}

/// Pendente: o voto é recusado porque o período de votação ainda não começou.
#[then("o voto é recusado porque o período de votação ainda não começou")]
fn vote_is_denied_before_period(_world: &mut AppWorld) {
    pending_step("o voto é recusado porque o período de votação ainda não começou");
}

/// Pendente: o voto é recusado porque o período de votação terminou.
#[then("o voto é recusado porque o período de votação terminou")]
fn vote_is_denied_after_period(_world: &mut AppWorld) {
    pending_step("o voto é recusado porque o período de votação terminou");
}

/// Pendente: que o estudante já registrou seu voto nessa enquete.
#[given("que o estudante já registrou seu voto nessa enquete")]
fn student_has_voted(_world: &mut AppWorld) {
    pending_step("que o estudante já registrou seu voto nessa enquete");
}

/// Pendente: o estudante tenta confirmar outro voto nessa enquete.
#[when("o estudante tenta confirmar outro voto nessa enquete")]
fn student_attempts_second_vote(_world: &mut AppWorld) {
    pending_step("o estudante tenta confirmar outro voto nessa enquete");
}

/// Pendente: a tentativa é recusada porque ele já votou.
#[then("a tentativa é recusada porque ele já votou")]
fn duplicate_vote_is_denied(_world: &mut AppWorld) {
    pending_step("a tentativa é recusada porque ele já votou");
}

/// Pendente: o voto original é preservado sem acrescentar outro voto.
#[then("o voto original é preservado sem acrescentar outro voto")]
fn original_vote_is_preserved(_world: &mut AppWorld) {
    pending_step("o voto original é preservado sem acrescentar outro voto");
}

/// Pendente: que o estudante não pertence ao público definido pela gestão para votar nessa enquete.
#[given("que o estudante não pertence ao público definido pela gestão para votar nessa enquete")]
fn student_is_outside_voting_audience(_world: &mut AppWorld) {
    pending_step(
        "que o estudante não pertence ao público definido pela gestão para votar nessa enquete",
    );
}

/// Pendente: que existe uma consulta cuja enquete foi configurada pela gestão com voto identificado.
#[given("que existe uma consulta cuja enquete foi configurada pela gestão com voto identificado")]
fn poll_uses_identified_votes(_world: &mut AppWorld) {
    pending_step(
        "que existe uma consulta cuja enquete foi configurada pela gestão com voto identificado",
    );
}

/// Pendente: o voto permanece associado à identidade do estudante.
#[then("o voto permanece associado à identidade do estudante")]
fn vote_is_associated_with_student(_world: &mut AppWorld) {
    pending_step("o voto permanece associado à identidade do estudante");
}

/// Pendente: que existe uma consulta cuja enquete foi configurada pela gestão com voto secreto.
#[given("que existe uma consulta cuja enquete foi configurada pela gestão com voto secreto")]
fn poll_uses_secret_votes(_world: &mut AppWorld) {
    pending_step(
        "que existe uma consulta cuja enquete foi configurada pela gestão com voto secreto",
    );
}

/// Pendente: a escolha registrada não permite identificar o estudante.
#[then("a escolha registrada não permite identificar o estudante")]
fn secret_vote_does_not_identify_student(_world: &mut AppWorld) {
    pending_step("a escolha registrada não permite identificar o estudante");
}

/// Pendente: uma nova tentativa de voto desse estudante é recusada.
#[then("uma nova tentativa de voto desse estudante é recusada")]
fn secret_poll_rejects_second_vote(_world: &mut AppWorld) {
    pending_step("uma nova tentativa de voto desse estudante é recusada");
}

/// Pendente: que existe uma consulta aberta com enquete e sem prazo de encerramento.
#[given("que existe uma consulta aberta com enquete e sem prazo de encerramento")]
fn open_poll_has_no_deadline(_world: &mut AppWorld) {
    pending_step("que existe uma consulta aberta com enquete e sem prazo de encerramento");
}
