//! Steps pendentes da especificação de consultas estudantis (#302).
//!
//! Registra os padrões Gherkin sem simular consultas, permissões, votos ou resultados.
//! Cada step sinaliza a pendência explicitamente até que a automação exercite o
//! comportamento real do backend. A feature usa `@pending_backend` enquanto isso.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: que o {string} está ativo e regular no CApp.
#[given(expr = "que o {string} está ativo e regular no CApp")]
fn academic_center_is_active_and_regular(_world: &mut AppWorld, _center: String) {
    pending_step("que o {string} está ativo e regular no CApp");
}

/// Pendente: que o Centro Acadêmico possui estudantes associados matriculados no curso.
#[given("que o Centro Acadêmico possui estudantes associados matriculados no curso")]
fn academic_center_has_associated_students(_world: &mut AppWorld) {
    pending_step("que o Centro Acadêmico possui estudantes associados matriculados no curso");
}

/// Pendente: que o usuário é um membro da gestão do CA com permissão de governança.
#[given("que o usuário é um membro da gestão do CA com permissão de governança")]
fn user_is_ca_board_member_with_governance_permission(_world: &mut AppWorld) {
    pending_step("que o usuário é um membro da gestão do CA com permissão de governança");
}

/// Pendente: ele cadastra uma nova consulta estudantil definindo:.
#[when("ele cadastra uma nova consulta estudantil definindo:")]
fn user_registers_new_student_consultation(_world: &mut AppWorld) {
    pending_step("ele cadastra uma nova consulta estudantil definindo:");
}

/// Pendente: a consulta estudantil é registrada com sucesso no sistema.
#[then("a consulta estudantil é registrada com sucesso no sistema")]
fn consultation_is_registered_successfully(_world: &mut AppWorld) {
    pending_step("a consulta estudantil é registrada com sucesso no sistema");
}

/// Pendente: fica vinculada ao Centro Acadêmico no estado {string} aguardando o início do período.
#[then(
    expr = "fica vinculada ao Centro Acadêmico no estado {string} aguardando o início do período"
)]
fn consultation_is_linked_in_scheduled_state(_world: &mut AppWorld, _expected_state: String) {
    pending_step(
        "fica vinculada ao Centro Acadêmico no estado {string} aguardando o início do período",
    );
}

/// Pendente: que o usuário é um estudante regularmente matriculado, mas não integra a gestão do CA.
#[given("que o usuário é um estudante regularmente matriculado, mas não integra a gestão do CA")]
fn user_is_student_not_in_ca_board(_world: &mut AppWorld) {
    pending_step(
        "que o usuário é um estudante regularmente matriculado, mas não integra a gestão do CA",
    );
}

/// Pendente: ele tenta criar uma consulta estudantil para o Centro Acadêmico.
#[when("ele tenta criar uma consulta estudantil para o Centro Acadêmico")]
fn user_attempts_to_create_consultation_without_permission(_world: &mut AppWorld) {
    pending_step("ele tenta criar uma consulta estudantil para o Centro Acadêmico");
}

/// Pendente: a criação é recusada por falta de permissão administrativa.
#[then("a criação é recusada por falta de permissão administrativa")]
fn creation_is_refused_due_to_missing_permission(_world: &mut AppWorld) {
    pending_step("a criação é recusada por falta de permissão administrativa");
}

/// Pendente: nenhuma nova consulta é registrada para o Centro Acadêmico.
#[then("nenhuma nova consulta é registrada para o Centro Acadêmico")]
fn no_new_consultation_is_registered(_world: &mut AppWorld) {
    pending_step("nenhuma nova consulta é registrada para o Centro Acadêmico");
}

/// Pendente: que existe uma consulta estudantil em andamento no Centro Acadêmico.
#[given("que existe uma consulta estudantil em andamento no Centro Acadêmico")]
fn consultation_in_progress_in_ca(_world: &mut AppWorld) {
    pending_step("que existe uma consulta estudantil em andamento no Centro Acadêmico");
}

/// Pendente: que o estudante atende a todos os critérios de elegibilidade da consulta.
#[given("que o estudante atende a todos os critérios de elegibilidade da consulta")]
fn student_meets_all_eligibility_criteria(_world: &mut AppWorld) {
    pending_step("que o estudante atende a todos os critérios de elegibilidade da consulta");
}

/// Pendente: que o estudante ainda não participou dessa consulta.
#[given("que o estudante ainda não participou dessa consulta")]
fn student_has_not_participated_yet(_world: &mut AppWorld) {
    pending_step("que o estudante ainda não participou dessa consulta");
}

/// Pendente: o estudante escolhe uma das opções válidas e confirma seu voto.
#[when("o estudante escolhe uma das opções válidas e confirma seu voto")]
fn student_chooses_valid_option_and_confirms_vote(_world: &mut AppWorld) {
    pending_step("o estudante escolhe uma das opções válidas e confirma seu voto");
}

/// Pendente: a sua participação é confirmada com sucesso.
#[then("a sua participação é confirmada com sucesso")]
fn participation_confirmed_successfully(_world: &mut AppWorld) {
    pending_step("a sua participação é confirmada com sucesso");
}

/// Pendente: o voto é computado de forma anônima no cômputo da consulta.
#[then("o voto é computado de forma anônima no cômputo da consulta")]
fn vote_is_computed_anonymously(_world: &mut AppWorld) {
    pending_step("o voto é computado de forma anônima no cômputo da consulta");
}

/// Pendente: que existe uma consulta estudantil em andamento com critério exclusivo para {string}.
#[given(
    expr = "que existe uma consulta estudantil em andamento com critério exclusivo para {string}"
)]
fn consultation_with_exclusive_criterion(_world: &mut AppWorld, _criterion: String) {
    pending_step(
        "que existe uma consulta estudantil em andamento com critério exclusivo para {string}",
    );
}

/// Pendente: que o estudante é calouro e não atende aos critérios de elegibilidade.
#[given("que o estudante é calouro e não atende aos critérios de elegibilidade")]
fn student_is_freshman_and_ineligible(_world: &mut AppWorld) {
    pending_step("que o estudante é calouro e não atende aos critérios de elegibilidade");
}

/// Pendente: o estudante tenta submeter um voto nessa consulta.
#[when("o estudante tenta submeter um voto nessa consulta")]
fn student_attempts_to_submit_vote(_world: &mut AppWorld) {
    pending_step("o estudante tenta submeter um voto nessa consulta");
}

/// Pendente: a participação é rejeitada informando que o estudante não é elegível.
#[then("a participação é rejeitada informando que o estudante não é elegível")]
fn participation_rejected_ineligible(_world: &mut AppWorld) {
    pending_step("a participação é rejeitada informando que o estudante não é elegível");
}

/// Pendente: nenhum voto é computado para a consulta.
#[then("nenhum voto é computado para a consulta")]
fn no_vote_is_computed_for_consultation(_world: &mut AppWorld) {
    pending_step("nenhum voto é computado para a consulta");
}

/// Pendente: que uma estudante elegível já registrou seu voto em uma consulta em andamento.
#[given("que uma estudante elegível já registrou seu voto em uma consulta em andamento")]
fn eligible_student_already_voted(_world: &mut AppWorld) {
    pending_step("que uma estudante elegível já registrou seu voto em uma consulta em andamento");
}

/// Pendente: ela tenta submeter um novo voto na mesma consulta.
#[when("ela tenta submeter um novo voto na mesma consulta")]
fn student_attempts_to_submit_new_vote_in_same_consultation(_world: &mut AppWorld) {
    pending_step("ela tenta submeter um novo voto na mesma consulta");
}

/// Pendente: a nova tentativa é recusada por duplicidade de voto.
#[then("a nova tentativa é recusada por duplicidade de voto")]
fn new_attempt_refused_for_duplicate_vote(_world: &mut AppWorld) {
    pending_step("a nova tentativa é recusada por duplicidade de voto");
}

/// Pendente: o cômputo da consulta permanece inalterado com apenas o voto original.
#[then("o cômputo da consulta permanece inalterado com apenas o voto original")]
fn vote_tally_remains_unchanged(_world: &mut AppWorld) {
    pending_step("o cômputo da consulta permanece inalterado com apenas o voto original");
}

/// Pendente: que existe uma consulta estudantil agendada com período de votação iniciando no dia seguinte.
#[given(
    "que existe uma consulta estudantil agendada com período de votação iniciando no dia seguinte"
)]
fn consultation_scheduled_starting_next_day(_world: &mut AppWorld) {
    pending_step(
        "que existe uma consulta estudantil agendada com período de votação iniciando no dia seguinte",
    );
}

/// Pendente: um estudante elegível tenta registrar seu voto antecipadamente.
#[when("um estudante elegível tenta registrar seu voto antecipadamente")]
fn eligible_student_attempts_vote_in_advance(_world: &mut AppWorld) {
    pending_step("um estudante elegível tenta registrar seu voto antecipadamente");
}

/// Pendente: a ação é bloqueada com a indicação de que o período de votação ainda não foi iniciado.
#[then("a ação é bloqueada com a indicação de que o período de votação ainda não foi iniciado")]
fn action_blocked_voting_not_started(_world: &mut AppWorld) {
    pending_step(
        "a ação é bloqueada com a indicação de que o período de votação ainda não foi iniciado",
    );
}

/// Pendente: que o período de votação de uma consulta estudantil encerrou há 1 hora.
#[given("que o período de votação de uma consulta estudantil encerrou há 1 hora")]
fn consultation_ended_one_hour_ago(_world: &mut AppWorld) {
    pending_step("que o período de votação de uma consulta estudantil encerrou há 1 hora");
}

/// Pendente: um estudante elegível tenta registrar um voto.
#[when("um estudante elegível tenta registrar um voto")]
fn eligible_student_attempts_to_vote(_world: &mut AppWorld) {
    pending_step("um estudante elegível tenta registrar um voto");
}

/// Pendente: a ação é bloqueada com a indicação de que a consulta já está encerrada.
#[then("a ação é bloqueada com a indicação de que a consulta já está encerrada")]
fn action_blocked_consultation_already_closed(_world: &mut AppWorld) {
    pending_step("a ação é bloqueada com a indicação de que a consulta já está encerrada");
}

/// Pendente: o voto não é aceito.
#[then("o voto não é aceito")]
fn vote_not_accepted(_world: &mut AppWorld) {
    pending_step("o voto não é aceito");
}

/// Pendente: que uma consulta estudantil possui período de votação até o horário atual.
#[given("que uma consulta estudantil possui período de votação até o horário atual")]
fn consultation_voting_period_until_current_time(_world: &mut AppWorld) {
    pending_step("que uma consulta estudantil possui período de votação até o horário atual");
}

/// Pendente: o tempo limite de votação é atingido.
#[when("o tempo limite de votação é atingido")]
fn voting_time_limit_reached(_world: &mut AppWorld) {
    pending_step("o tempo limite de votação é atingido");
}

/// Pendente: a consulta transiciona automaticamente para o estado {string}.
#[then(expr = "a consulta transiciona automaticamente para o estado {string}")]
fn consultation_transitions_automatically_to_state(_world: &mut AppWorld, _expected_state: String) {
    pending_step("a consulta transiciona automaticamente para o estado {string}");
}

/// Pendente: novas tentativas de participação passam a ser bloqueadas imediatamente.
#[then("novas tentativas de participação passam a ser bloqueadas imediatamente")]
fn new_participation_attempts_blocked_immediately(_world: &mut AppWorld) {
    pending_step("novas tentativas de participação passam a ser bloqueadas imediatamente");
}

/// Pendente: que existe uma consulta estudantil em andamento.
#[given("que existe uma consulta estudantil em andamento")]
fn consultation_in_progress(_world: &mut AppWorld) {
    pending_step("que existe uma consulta estudantil em andamento");
}

/// Pendente: ele solicita o encerramento manual da consulta com justificativa registrada.
#[when("ele solicita o encerramento manual da consulta com justificativa registrada")]
fn user_requests_manual_closure_with_justification(_world: &mut AppWorld) {
    pending_step("ele solicita o encerramento manual da consulta com justificativa registrada");
}

/// Pendente: a consulta é finalizada com o estado {string}.
#[then(expr = "a consulta é finalizada com o estado {string}")]
fn consultation_finalized_with_state(_world: &mut AppWorld, _expected_state: String) {
    pending_step("a consulta é finalizada com o estado {string}");
}

/// Pendente: a urna de votação é imediatamente fechada para novas participações.
#[then("a urna de votação é imediatamente fechada para novas participações")]
fn ballot_box_closed_for_new_participations(_world: &mut AppWorld) {
    pending_step("a urna de votação é imediatamente fechada para novas participações");
}

/// Pendente: que uma consulta estudantil foi encerrada e teve sua apuração finalizada.
#[given("que uma consulta estudantil foi encerrada e teve sua apuração finalizada")]
fn consultation_closed_and_tallied(_world: &mut AppWorld) {
    pending_step("que uma consulta estudantil foi encerrada e teve sua apuração finalizada");
}

/// Pendente: qualquer estudante vinculado ao Centro Acadêmico acessa os resultados.
#[when("qualquer estudante vinculado ao Centro Acadêmico acessa os resultados")]
fn student_accesses_results(_world: &mut AppWorld) {
    pending_step("qualquer estudante vinculado ao Centro Acadêmico acessa os resultados");
}

/// Pendente: ele visualiza o total de participantes da consulta.
#[then("ele visualiza o total de participantes da consulta")]
fn user_views_total_participants(_world: &mut AppWorld) {
    pending_step("ele visualiza o total de participantes da consulta");
}

/// Pendente: visualiza a contagem consolidada e o percentual de cada opção disponível.
#[then("visualiza a contagem consolidada e o percentual de cada opção disponível")]
fn user_views_consolidated_counts_and_percentages(_world: &mut AppWorld) {
    pending_step("visualiza a contagem consolidada e o percentual de cada opção disponível");
}

/// Pendente: nenhum dado identificável que viole o sigilo do voto individual é exibido.
#[then("nenhum dado identificável que viole o sigilo do voto individual é exibido")]
fn no_identifiable_data_violating_secrecy_displayed(_world: &mut AppWorld) {
    pending_step("nenhum dado identificável que viole o sigilo do voto individual é exibido");
}

/// Pendente: que uma consulta estudantil está com a votação em andamento.
#[given("que uma consulta estudantil está com a votação em andamento")]
fn consultation_voting_in_progress(_world: &mut AppWorld) {
    pending_step("que uma consulta estudantil está com a votação em andamento");
}

/// Pendente: um estudante ou gestor tenta acessar a apuração parcial dos votos.
#[when("um estudante ou gestor tenta acessar a apuração parcial dos votos")]
fn student_or_manager_attempts_access_partial_results(_world: &mut AppWorld) {
    pending_step("um estudante ou gestor tenta acessar a apuração parcial dos votos");
}

/// Pendente: os resultados consolidados permanecem ocultos até o término oficial da consulta.
#[then("os resultados consolidados permanecem ocultos até o término oficial da consulta")]
fn consolidated_results_remain_hidden_until_official_close(_world: &mut AppWorld) {
    pending_step("os resultados consolidados permanecem ocultos até o término oficial da consulta");
}

/// Pendente: que o Centro Acadêmico possui simultaneamente:.
#[given("que o Centro Acadêmico possui simultaneamente:")]
fn academic_center_has_simultaneously(_world: &mut AppWorld) {
    pending_step("que o Centro Acadêmico possui simultaneamente:");
}

/// Pendente: um estudante participa da consulta sobre o calendário.
#[when("um estudante participa da consulta sobre o calendário")]
fn student_participates_in_calendar_consultation(_world: &mut AppWorld) {
    pending_step("um estudante participa da consulta sobre o calendário");
}

/// Pendente: essa participação não altera o caderno eleitoral da eleição de chapas.
#[then("essa participação não altera o caderno eleitoral da eleição de chapas")]
fn participation_does_not_affect_electoral_roll(_world: &mut AppWorld) {
    pending_step("essa participação não altera o caderno eleitoral da eleição de chapas");
}

/// Pendente: as regras de quórum, elegibilidade e sigilo da consulta operam de modo isolado da eleição.
#[then("as regras de quórum, elegibilidade e sigilo da consulta operam de modo isolado da eleição")]
fn consultation_rules_operate_isolated_from_election(_world: &mut AppWorld) {
    pending_step(
        "as regras de quórum, elegibilidade e sigilo da consulta operam de modo isolado da eleição",
    );
}
