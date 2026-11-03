//! Visualização dos resultados das consultas estudantis.
//!
//! Steps exclusivamente pendentes; não simulam estado nem comportamento do produto.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: que ele pode acessar a consulta, mas não pertence ao público de visualização dos resultados.
#[given(
    "que ele pode acessar a consulta, mas não pertence ao público de visualização dos resultados"
)]
fn student_has_access_without_results_permission(_world: &mut AppWorld) {
    pending_step(
        "que ele pode acessar a consulta, mas não pertence ao público de visualização dos resultados",
    );
}

/// Pendente: ele consulta o resultado da enquete.
#[when("ele consulta o resultado da enquete")]
fn user_requests_poll_results(_world: &mut AppWorld) {
    pending_step("ele consulta o resultado da enquete");
}

/// Pendente: o acesso ao resultado é recusado.
#[then("o acesso ao resultado é recusado")]
fn results_access_is_denied(_world: &mut AppWorld) {
    pending_step("o acesso ao resultado é recusado");
}

/// Pendente: os totais de votos não são apresentados.
#[then("os totais de votos não são apresentados")]
fn vote_totals_are_hidden(_world: &mut AppWorld) {
    pending_step("os totais de votos não são apresentados");
}

/// Pendente: que a enquete recebeu 2 votos em {string} e 1 voto em {string}.
#[given(expr = "que a enquete recebeu 2 votos em {string} e 1 voto em {string}")]
fn poll_has_votes_for_options(_world: &mut AppWorld, _value_1: String, _value_2: String) {
    pending_step("que a enquete recebeu 2 votos em {string} e 1 voto em {string}");
}

/// Pendente: que o usuário pertence ao público definido pela gestão para visualizar os resultados.
#[given("que o usuário pertence ao público definido pela gestão para visualizar os resultados")]
fn user_is_in_results_audience(_world: &mut AppWorld) {
    pending_step(
        "que o usuário pertence ao público definido pela gestão para visualizar os resultados",
    );
}

/// Pendente: ele visualiza 2 votos em {string} e 1 voto em {string}.
#[then(expr = "ele visualiza 2 votos em {string} e 1 voto em {string}")]
fn user_sees_vote_totals(_world: &mut AppWorld, _value_1: String, _value_2: String) {
    pending_step("ele visualiza 2 votos em {string} e 1 voto em {string}");
}

/// Pendente: que o usuário não pertence ao público definido pela gestão para visualizar os resultados.
#[given("que o usuário não pertence ao público definido pela gestão para visualizar os resultados")]
fn user_is_outside_results_audience(_world: &mut AppWorld) {
    pending_step(
        "que o usuário não pertence ao público definido pela gestão para visualizar os resultados",
    );
}
