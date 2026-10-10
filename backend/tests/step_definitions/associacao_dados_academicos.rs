//! Steps pendentes da especificação de associação dos dados acadêmicos (#5).
//!
//! Os padrões Gherkin estão registrados, mas a automação ainda não foi
//! implementada. Cada step falha explicitamente com uma mensagem de pendência;
//! nenhum resultado de consulta ou associação é simulado ou aprovado
//! antecipadamente. A implementação posterior deverá consultar a fonte
//! institucional, associar os dados ao perfil e verificar os resultados reais.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: o estudante possui uma identidade institucional autenticada.
#[given("que o estudante possui uma identidade institucional autenticada")]
fn student_has_authenticated_institutional_identity(_world: &mut AppWorld) {
    pending_step("que o estudante possui uma identidade institucional autenticada");
}

/// Pendente: a instituição disponibiliza o curso e a situação acadêmica.
#[given("a instituição disponibiliza o curso e a situação acadêmica do estudante")]
fn institution_provides_academic_data(_world: &mut AppWorld) {
    pending_step("a instituição disponibiliza o curso e a situação acadêmica do estudante");
}

/// Pendente: a instituição não disponibiliza os dados acadêmicos necessários.
#[given(
    "a instituição não disponibiliza os dados acadêmicos necessários para identificar seu contexto"
)]
fn institution_does_not_provide_academic_data(_world: &mut AppWorld) {
    pending_step(
        "a instituição não disponibiliza os dados acadêmicos necessários para identificar seu contexto",
    );
}

/// Pendente: a instituição não disponibiliza uma informação acadêmica específica.
#[given(expr = "a instituição não disponibiliza a informação {string} do estudante")]
fn institution_does_not_provide_specific_data(_world: &mut AppWorld, _information: String) {
    pending_step("a instituição não disponibiliza a informação {string} do estudante");
}

/// Pendente: não foi possível obter os dados acadêmicos da instituição.
#[given("não foi possível obter os dados acadêmicos da instituição")]
fn academic_data_could_not_be_obtained(_world: &mut AppWorld) {
    pending_step("não foi possível obter os dados acadêmicos da instituição");
}

/// Pendente: o sistema obtém os dados acadêmicos do estudante.
#[when("o sistema obtém os dados acadêmicos do estudante")]
fn system_fetches_academic_data(_world: &mut AppWorld) {
    pending_step("o sistema obtém os dados acadêmicos do estudante");
}

/// Pendente: o sistema tenta obter os dados acadêmicos do estudante.
#[when("o sistema tenta obter os dados acadêmicos do estudante")]
fn system_attempts_to_fetch_academic_data(_world: &mut AppWorld) {
    pending_step("o sistema tenta obter os dados acadêmicos do estudante");
}

/// Pendente: o sistema tenta associar os dados acadêmicos ao perfil.
#[when("o sistema tenta associar os dados acadêmicos ao perfil")]
fn system_attempts_to_associate_academic_data(_world: &mut AppWorld) {
    pending_step("o sistema tenta associar os dados acadêmicos ao perfil");
}

/// Pendente: o sistema tenta concluir a associação dos dados acadêmicos.
#[when("o sistema tenta concluir a associação dos dados acadêmicos")]
fn system_attempts_to_complete_association(_world: &mut AppWorld) {
    pending_step("o sistema tenta concluir a associação dos dados acadêmicos");
}

/// Pendente: o curso fica associado ao perfil do estudante.
#[then("o curso do estudante fica associado ao seu perfil")]
fn course_is_associated_with_profile(_world: &mut AppWorld) {
    pending_step("o curso do estudante fica associado ao seu perfil");
}

/// Pendente: a situação acadêmica fica associada ao perfil do estudante.
#[then("a situação acadêmica do estudante fica associada ao seu perfil")]
fn academic_status_is_associated_with_profile(_world: &mut AppWorld) {
    pending_step("a situação acadêmica do estudante fica associada ao seu perfil");
}

/// Pendente: os dados ficam disponíveis para as regras de acesso.
#[then("essas informações ficam disponíveis para as regras de acesso do aplicativo")]
fn academic_data_is_available_for_access_rules(_world: &mut AppWorld) {
    pending_step("essas informações ficam disponíveis para as regras de acesso do aplicativo");
}

/// Pendente: o sistema não associa dados acadêmicos que não foram recuperados.
#[then("o sistema não associa dados acadêmicos que não foram recuperados")]
fn unavailable_data_is_not_associated(_world: &mut AppWorld) {
    pending_step("o sistema não associa dados acadêmicos que não foram recuperados");
}

/// Pendente: o perfil não apresenta dados ausentes como confirmados.
#[then("o perfil não apresenta esses dados como se estivessem confirmados")]
fn profile_does_not_present_missing_data_as_confirmed(_world: &mut AppWorld) {
    pending_step("o perfil não apresenta esses dados como se estivessem confirmados");
}

/// Pendente: as regras de acesso não tratam dados ausentes como confirmados.
#[then("as regras de acesso não tratam os dados ausentes como informações acadêmicas confirmadas")]
fn access_rules_do_not_trust_missing_data(_world: &mut AppWorld) {
    pending_step(
        "as regras de acesso não tratam os dados ausentes como informações acadêmicas confirmadas",
    );
}

/// Pendente: uma informação acadêmica específica não é apresentada como confirmada.
#[then(expr = "a informação {string} não é apresentada como confirmada")]
fn academic_information_is_not_confirmed(_world: &mut AppWorld, _information: String) {
    pending_step("a informação {string} não é apresentada como confirmada");
}

/// Pendente: o sistema não considera completo o contexto acadêmico.
#[then("o sistema não considera completo o contexto acadêmico do estudante")]
fn academic_context_is_not_complete(_world: &mut AppWorld) {
    pending_step("o sistema não considera completo o contexto acadêmico do estudante");
}

/// Pendente: o sistema não confirma uma associação que não foi concluída.
#[then("o sistema não confirma uma associação que não foi concluída")]
fn incomplete_association_is_not_confirmed(_world: &mut AppWorld) {
    pending_step("o sistema não confirma uma associação que não foi concluída");
}

/// Pendente: os dados não obtidos ficam indisponíveis para as regras de acesso.
#[then("as informações acadêmicas não obtidas permanecem indisponíveis para as regras de acesso")]
fn unobtained_academic_data_remains_unavailable(_world: &mut AppWorld) {
    pending_step(
        "as informações acadêmicas não obtidas permanecem indisponíveis para as regras de acesso",
    );
}
