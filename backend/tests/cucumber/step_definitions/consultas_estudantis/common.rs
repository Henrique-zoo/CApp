//! Passos compartilhados entre as funcionalidades de consultas estudantis.
//!
//! Steps exclusivamente pendentes; não simulam estado nem comportamento do produto.

use cucumber::given;

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: que o {string} está cadastrado no CApp.
#[given(expr = "que o {string} está cadastrado no CApp")]
fn academic_center_is_registered(_world: &mut AppWorld, _value_1: String) {
    pending_step("que o {string} está cadastrado no CApp");
}

/// Pendente: que esse Centro Acadêmico é o contexto ativo do usuário.
#[given("que esse Centro Acadêmico é o contexto ativo do usuário")]
fn academic_center_is_active_context(_world: &mut AppWorld) {
    pending_step("que esse Centro Acadêmico é o contexto ativo do usuário");
}

/// Pendente: que existe uma consulta aberta com enquete e chat no Centro Acadêmico ativo.
#[given("que existe uma consulta aberta com enquete e chat no Centro Acadêmico ativo")]
fn open_consultation_has_poll_and_chat(_world: &mut AppWorld) {
    pending_step("que existe uma consulta aberta com enquete e chat no Centro Acadêmico ativo");
}

/// Pendente: que o período de votação começou há 1 dia e termina daqui a 1 dia.
#[given("que o período de votação começou há 1 dia e termina daqui a 1 dia")]
fn voting_period_is_open(_world: &mut AppWorld) {
    pending_step("que o período de votação começou há 1 dia e termina daqui a 1 dia");
}

/// Pendente: que existe uma consulta com enquete no Centro Acadêmico ativo.
#[given("que existe uma consulta com enquete no Centro Acadêmico ativo")]
fn consultation_has_poll(_world: &mut AppWorld) {
    pending_step("que existe uma consulta com enquete no Centro Acadêmico ativo");
}

/// Pendente: que o estudante pertence ao público definido pela gestão para votar nessa enquete.
#[given("que o estudante pertence ao público definido pela gestão para votar nessa enquete")]
fn student_is_in_voting_audience(_world: &mut AppWorld) {
    pending_step(
        "que o estudante pertence ao público definido pela gestão para votar nessa enquete",
    );
}

/// Pendente: que o período de votação terminou há 1 hora.
#[given("que o período de votação terminou há 1 hora")]
fn voting_period_has_ended(_world: &mut AppWorld) {
    pending_step("que o período de votação terminou há 1 hora");
}
