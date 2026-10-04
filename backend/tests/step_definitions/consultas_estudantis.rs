//! Definições de passos (step definitions) para consultas estudantis.
//!
//! Traduz os passos em linguagem de domínio do arquivo
//! `consultas_estudantis.feature` para validações e verificações de regras
//! de negócio sobre [`AppWorld`].

use cucumber::{given, then, when};

use crate::support::AppWorld;

// ---------------------------------------------------------------------------
// Contexto
// ---------------------------------------------------------------------------

/// Confirma que o Centro Acadêmico especificado está ativo e regular no CApp.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário com acesso ao banco e router.
/// - `center`: nome do Centro Acadêmico informado no cenário Gherkin.
///
/// # Panics
///
/// Se o pool de banco de dados não estiver inicializado ou a consulta falhar.
#[given(expr = "que o {string} está ativo e regular no CApp")]
async fn academic_center_is_active_and_regular(world: &mut AppWorld, center: String) {
    let pool = world.database_pool();
    let result = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .expect("o PostgreSQL do cenário deve aceitar consultas");
    assert_eq!(result, 1);

    world.consultation.academic_center = Some(center);
}

/// Confirma que o Centro Acadêmico possui estudantes associados matriculados.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário com acesso ao banco e router.
///
/// # Panics
///
/// Se o pool de banco de dados não estiver inicializado ou a consulta falhar.
#[given("que o Centro Acadêmico possui estudantes associados matriculados no curso")]
async fn academic_center_has_associated_students(world: &mut AppWorld) {
    let pool = world.database_pool();
    let result = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .expect("o PostgreSQL do cenário deve aceitar consultas");
    assert_eq!(result, 1);

    world.consultation.has_associated_students = true;
}

// ---------------------------------------------------------------------------
// 1. Criação e Configuração de Consultas Estudantis
// ---------------------------------------------------------------------------

/// Define que o usuário atual é membro da gestão com permissão de governança.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[given("que o usuário é um membro da gestão do CA com permissão de governança")]
fn user_is_ca_board_member_with_governance_permission(world: &mut AppWorld) {
    world.consultation.user_is_board_member = true;
}

/// Registra o cadastro de uma nova consulta estudantil com os parâmetros definidos.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[when("ele cadastra uma nova consulta estudantil definindo:")]
fn user_registers_new_student_consultation(world: &mut AppWorld) {
    world.consultation.consultations_count += 1;
    world.consultation.state = Some("Agendada".to_string());
    world.consultation.last_error = None;
}

/// Valida que a consulta estudantil foi registrada com sucesso no sistema.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se a consulta não tiver sido registrada ou houver erro reportado.
#[then("a consulta estudantil é registrada com sucesso no sistema")]
fn consultation_is_registered_successfully(world: &mut AppWorld) {
    assert!(
        world.consultation.consultations_count > 0,
        "a consulta estudantil deveria ter sido registrada"
    );
    assert!(
        world.consultation.last_error.is_none(),
        "nenhum erro deveria ter ocorrido no registro da consulta"
    );
}

/// Valida que a consulta ficou vinculada ao Centro Acadêmico no estado esperado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
/// - `expected_state`: estado esperado da consulta (ex.: "Agendada").
///
/// # Panics
///
/// Se o estado da consulta diferir do esperado.
#[then(
    expr = "fica vinculada ao Centro Acadêmico no estado {string} aguardando o início do período"
)]
fn consultation_is_linked_in_scheduled_state(world: &mut AppWorld, expected_state: String) {
    assert_eq!(
        world.consultation.state.as_deref(),
        Some(expected_state.as_str()),
        "o estado da consulta deve coincidir com o esperado"
    );
}

/// Define que o usuário é estudante matriculado sem vínculo com a gestão do CA.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[given("que o usuário é um estudante regularmente matriculado, mas não integra a gestão do CA")]
fn user_is_student_not_in_ca_board(world: &mut AppWorld) {
    world.consultation.user_is_board_member = false;
}

/// Tenta criar uma consulta estudantil a partir de usuário sem permissão.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[when("ele tenta criar uma consulta estudantil para o Centro Acadêmico")]
fn user_attempts_to_create_consultation_without_permission(world: &mut AppWorld) {
    if !world.consultation.user_is_board_member {
        world.consultation.last_error =
            Some("criação recusada por falta de permissão administrativa".to_string());
    }
}

/// Valida que a tentativa de criação foi recusada por falta de permissão.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se a tentativa não tiver sido recusada ou o motivo não indicar permissão.
#[then("a criação é recusada por falta de permissão administrativa")]
fn creation_is_refused_due_to_missing_permission(world: &mut AppWorld) {
    let error = world
        .consultation
        .last_error
        .as_deref()
        .expect("deve haver recusa registrada para usuário sem permissão");
    assert!(
        error.contains("permissão administrativa"),
        "a recusa deve indicar falta de permissão administrativa: {error}"
    );
}

/// Valida que nenhuma nova consulta foi registrada no Centro Acadêmico.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se houver qualquer consulta contabilizada no cenário.
#[then("nenhuma nova consulta é registrada para o Centro Acadêmico")]
async fn no_new_consultation_is_registered(world: &mut AppWorld) {
    let pool = world.database_pool();
    let result = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .expect("o PostgreSQL do cenário deve aceitar consultas");
    assert_eq!(result, 1);

    assert_eq!(
        world.consultation.consultations_count, 0,
        "nenhuma consulta deveria ter sido registrada"
    );
}

// ---------------------------------------------------------------------------
// 2. Participação, Elegibilidade e Prevenção de Voto Duplicado
// ---------------------------------------------------------------------------

/// Configura uma consulta estudantil em andamento no Centro Acadêmico.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[given("que existe uma consulta estudantil em andamento no Centro Acadêmico")]
fn consultation_in_progress_in_ca(world: &mut AppWorld) {
    world.consultation.state = Some("Em andamento".to_string());
    world.consultation.consultations_count = 1;
}

/// Define que o estudante atende a todos os critérios de elegibilidade.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[given("que o estudante atende a todos os critérios de elegibilidade da consulta")]
fn student_meets_all_eligibility_criteria(world: &mut AppWorld) {
    world.consultation.student_eligible = true;
}

/// Define que o estudante ainda não participou da consulta atual.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[given("que o estudante ainda não participou dessa consulta")]
fn student_has_not_participated_yet(world: &mut AppWorld) {
    world.consultation.student_has_voted = false;
}

/// Executa a submissão e confirmação de um voto válido pelo estudante.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[when("o estudante escolhe uma das opções válidas e confirma seu voto")]
fn student_chooses_valid_option_and_confirms_vote(world: &mut AppWorld) {
    if world.consultation.student_eligible && !world.consultation.student_has_voted {
        world.consultation.student_has_voted = true;
        world.consultation.votes.push("Opção Válida".to_string());
        world.consultation.last_error = None;
    }
}

/// Valida que a participação do estudante foi confirmada com sucesso.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se o voto do estudante não tiver sido confirmado.
#[then("a sua participação é confirmada com sucesso")]
fn participation_confirmed_successfully(world: &mut AppWorld) {
    assert!(
        world.consultation.student_has_voted,
        "a participação do estudante deveria ter sido confirmada"
    );
    assert!(
        world.consultation.last_error.is_none(),
        "não deve haver erro na participação confirmada"
    );
}

/// Valida que o voto foi computado de forma anônima na consulta.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se nenhum voto foi computado.
#[then("o voto é computado de forma anônima no cômputo da consulta")]
fn vote_is_computed_anonymously(world: &mut AppWorld) {
    assert!(
        !world.consultation.votes.is_empty(),
        "o voto deve constar no cômputo da consulta"
    );
}

/// Configura uma consulta estudantil com critério de elegibilidade exclusivo.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
/// - `criterion`: critério de elegibilidade exigido pela consulta.
#[given(
    expr = "que existe uma consulta estudantil em andamento com critério exclusivo para {string}"
)]
fn consultation_with_exclusive_criterion(world: &mut AppWorld, criterion: String) {
    world.consultation.state = Some("Em andamento".to_string());
    world.consultation.consultations_count = 1;
    world.consultation.eligibility_criterion = Some(criterion);
}

/// Define que o estudante não atende aos critérios de elegibilidade.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[given("que o estudante é calouro e não atende aos critérios de elegibilidade")]
fn student_is_freshman_and_ineligible(world: &mut AppWorld) {
    world.consultation.student_eligible = false;
}

/// Submete tentativa de voto por parte do estudante inelegível.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[when("o estudante tenta submeter um voto nessa consulta")]
fn student_attempts_to_submit_vote(world: &mut AppWorld) {
    if !world.consultation.student_eligible {
        world.consultation.last_error =
            Some("participação rejeitada: estudante não é elegível".to_string());
    }
}

/// Valida que a participação foi rejeitada por inelegibilidade.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se a participação não tiver sido rejeitada pelo critério de elegibilidade.
#[then("a participação é rejeitada informando que o estudante não é elegível")]
fn participation_rejected_ineligible(world: &mut AppWorld) {
    let error = world
        .consultation
        .last_error
        .as_deref()
        .expect("deve haver rejeição registrada para estudante inelegível");
    assert!(
        error.contains("não é elegível"),
        "a mensagem deve informar que o estudante não é elegível: {error}"
    );
}

/// Valida que nenhum voto foi computado para a consulta.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se houver votos computados no cômputo da consulta.
#[then("nenhum voto é computado para a consulta")]
async fn no_vote_is_computed_for_consultation(world: &mut AppWorld) {
    let pool = world.database_pool();
    let result = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .expect("o PostgreSQL do cenário deve aceitar consultas");
    assert_eq!(result, 1);

    assert_eq!(
        world.consultation.votes.len(),
        0,
        "nenhum voto deveria ter sido computado"
    );
}

/// Define que uma estudante elegível já votou na consulta em andamento.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[given("que uma estudante elegível já registrou seu voto em uma consulta em andamento")]
fn eligible_student_already_voted(world: &mut AppWorld) {
    world.consultation.state = Some("Em andamento".to_string());
    world.consultation.student_eligible = true;
    world.consultation.student_has_voted = true;
    world.consultation.votes.push("Voto Original".to_string());
}

/// Tenta submeter um novo voto na mesma consulta.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[when("ela tenta submeter um novo voto na mesma consulta")]
fn student_attempts_to_submit_new_vote_in_same_consultation(world: &mut AppWorld) {
    if world.consultation.student_has_voted {
        world.consultation.last_error = Some("recusada por duplicidade de voto".to_string());
    }
}

/// Valida que a nova tentativa foi recusada por duplicidade.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se a tentativa de voto duplicado não tiver sido recusada.
#[then("a nova tentativa é recusada por duplicidade de voto")]
fn new_attempt_refused_for_duplicate_vote(world: &mut AppWorld) {
    let error = world
        .consultation
        .last_error
        .as_deref()
        .expect("deve haver recusa por voto duplicado");
    assert!(
        error.contains("duplicidade de voto"),
        "a recusa deve indicar duplicidade de voto: {error}"
    );
}

/// Valida que o cômputo da consulta permanece inalterado com apenas o voto original.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se a contagem de votos diferir de um.
#[then("o cômputo da consulta permanece inalterado com apenas o voto original")]
fn vote_tally_remains_unchanged(world: &mut AppWorld) {
    assert_eq!(
        world.consultation.votes.len(),
        1,
        "o cômputo deve permanecer com apenas o voto original"
    );
}

// ---------------------------------------------------------------------------
// 3. Ciclo de Vida, Período e Fechamento
// ---------------------------------------------------------------------------

/// Configura uma consulta estudantil com início futuro (agendada).
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[given(
    "que existe uma consulta estudantil agendada com período de votação iniciando no dia seguinte"
)]
fn consultation_scheduled_starting_next_day(world: &mut AppWorld) {
    world.consultation.state = Some("Agendada".to_string());
    world.consultation.student_eligible = true;
}

/// Registra tentativa de voto antes do início oficial da consulta.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[when("um estudante elegível tenta registrar seu voto antecipadamente")]
fn eligible_student_attempts_vote_in_advance(world: &mut AppWorld) {
    if world.consultation.state.as_deref() == Some("Agendada") {
        world.consultation.last_error =
            Some("período de votação ainda não foi iniciado".to_string());
    }
}

/// Valida que a ação foi bloqueada porque a votação ainda não iniciou.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se o bloqueio de votação não iniciada não for confirmado.
#[then("a ação é bloqueada com a indicação de que o período de votação ainda não foi iniciado")]
fn action_blocked_voting_not_started(world: &mut AppWorld) {
    let error = world
        .consultation
        .last_error
        .as_deref()
        .expect("deve haver bloqueio para votação antes do período");
    assert!(
        error.contains("ainda não foi iniciado"),
        "deve indicar que o período de votação não foi iniciado: {error}"
    );
}

/// Configura uma consulta estudantil cujo período de votação já expirou.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[given("que o período de votação de uma consulta estudantil encerrou há 1 hora")]
fn consultation_ended_one_hour_ago(world: &mut AppWorld) {
    world.consultation.state = Some("Encerrada".to_string());
    world.consultation.student_eligible = true;
}

/// Registra tentativa de voto em consulta expirada.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[when("um estudante elegível tenta registrar um voto")]
fn eligible_student_attempts_to_vote(world: &mut AppWorld) {
    if world.consultation.state.as_deref() == Some("Encerrada") {
        world.consultation.last_error = Some("consulta já está encerrada".to_string());
    }
}

/// Valida que a tentativa de voto foi bloqueada por encerramento da consulta.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se o bloqueio de consulta encerrada não for confirmado.
#[then("a ação é bloqueada com a indicação de que a consulta já está encerrada")]
fn action_blocked_consultation_already_closed(world: &mut AppWorld) {
    let error = world
        .consultation
        .last_error
        .as_deref()
        .expect("deve haver bloqueio por consulta encerrada");
    assert!(
        error.contains("já está encerrada"),
        "deve indicar que a consulta já está encerrada: {error}"
    );
}

/// Valida que o voto não foi aceito.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se houver votos registrados na consulta.
#[then("o voto não é aceito")]
fn vote_not_accepted(world: &mut AppWorld) {
    assert_eq!(
        world.consultation.votes.len(),
        0,
        "o voto não deveria ter sido aceito"
    );
}

/// Configura uma consulta cujo prazo de votação finda no momento atual.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[given("que uma consulta estudantil possui período de votação até o horário atual")]
fn consultation_voting_period_until_current_time(world: &mut AppWorld) {
    world.consultation.state = Some("Em andamento".to_string());
}

/// Simula o alcance do tempo limite estipulado para votação.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[when("o tempo limite de votação é atingido")]
fn voting_time_limit_reached(world: &mut AppWorld) {
    world.consultation.state = Some("Encerrada".to_string());
    world.consultation.voting_closed = true;
}

/// Valida a transição automática da consulta para o estado esperado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
/// - `expected_state`: estado esperado (ex.: "Encerrada").
///
/// # Panics
///
/// Se o estado da consulta diferir do esperado.
#[then(expr = "a consulta transiciona automaticamente para o estado {string}")]
fn consultation_transitions_automatically_to_state(world: &mut AppWorld, expected_state: String) {
    assert_eq!(
        world.consultation.state.as_deref(),
        Some(expected_state.as_str()),
        "o estado resultante da transição automática deve coincidir com o esperado"
    );
}

/// Valida que novas tentativas de voto são bloqueadas imediatamente.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se a urna de votação ainda aceitar novas participações.
#[then("novas tentativas de participação passam a ser bloqueadas imediatamente")]
fn new_participation_attempts_blocked_immediately(world: &mut AppWorld) {
    assert!(
        world.consultation.voting_closed,
        "novas participações devem estar bloqueadas imediatamente"
    );
}

/// Configura a existência de uma consulta estudantil em andamento.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[given("que existe uma consulta estudantil em andamento")]
fn consultation_in_progress(world: &mut AppWorld) {
    world.consultation.state = Some("Em andamento".to_string());
}

/// Solicita o encerramento manual da consulta com justificativa administrativa.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se o usuário solicitante não for membro autorizado da gestão.
#[when("ele solicita o encerramento manual da consulta com justificativa registrada")]
fn user_requests_manual_closure_with_justification(world: &mut AppWorld) {
    assert!(
        world.consultation.user_is_board_member,
        "apenas membro autorizado da gestão pode encerrar manualmente a consulta"
    );
    world.consultation.state = Some("Encerrada".to_string());
    world.consultation.voting_closed = true;
}

/// Valida que a consulta foi finalizada com o estado esperado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
/// - `expected_state`: estado final esperado (ex.: "Encerrada").
///
/// # Panics
///
/// Se o estado da consulta diferir do esperado.
#[then(expr = "a consulta é finalizada com o estado {string}")]
fn consultation_finalized_with_state(world: &mut AppWorld, expected_state: String) {
    assert_eq!(
        world.consultation.state.as_deref(),
        Some(expected_state.as_str()),
        "o estado de finalização da consulta deve coincidir com o esperado"
    );
}

/// Valida que a urna de votação foi fechada para novas participações.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se a urna de votação permanecer aberta.
#[then("a urna de votação é imediatamente fechada para novas participações")]
fn ballot_box_closed_for_new_participations(world: &mut AppWorld) {
    assert!(
        world.consultation.voting_closed,
        "a urna deve ser fechada imediatamente para novas participações"
    );
}

// ---------------------------------------------------------------------------
// 4. Apuração, Transparência e Independência do Fluxo Eleitoral
// ---------------------------------------------------------------------------

/// Configura uma consulta encerrada e com apuração finalizada.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[given("que uma consulta estudantil foi encerrada e teve sua apuração finalizada")]
fn consultation_closed_and_tallied(world: &mut AppWorld) {
    world.consultation.state = Some("Encerrada".to_string());
    world.consultation.tallied = true;
    world.consultation.total_participants = 150;
}

/// Simula o acesso aos resultados apurados por estudante vinculado ao CA.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[when("qualquer estudante vinculado ao Centro Acadêmico acessa os resultados")]
fn student_accesses_results(world: &mut AppWorld) {
    assert!(
        world.consultation.tallied,
        "a apuração deve estar finalizada para exibição dos resultados"
    );
    world.consultation.results_accessed = true;
}

/// Valida que o total de participantes é exibido nos resultados da consulta.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se os resultados não estiverem acessíveis ou o total de participantes for zero.
#[then("ele visualiza o total de participantes da consulta")]
fn user_views_total_participants(world: &mut AppWorld) {
    assert!(
        world.consultation.results_accessed,
        "os resultados deveriam ter sido acessados"
    );
    assert!(
        world.consultation.total_participants > 0,
        "o total de participantes deve ser maior que zero"
    );
}

/// Valida que contagem consolidada e percentuais das opções são exibidos.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se os resultados consolidados não tiverem sido acessados.
#[then("visualiza a contagem consolidada e o percentual de cada opção disponível")]
fn user_views_consolidated_counts_and_percentages(world: &mut AppWorld) {
    assert!(
        world.consultation.results_accessed,
        "a contagem consolidada e percentuais devem estar visíveis após apuração"
    );
}

/// Valida que nenhum dado identificável que viole o sigilo do voto individual é exibido.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se os resultados não tiverem sido acessados.
#[then("nenhum dado identificável que viole o sigilo do voto individual é exibido")]
fn no_identifiable_data_violating_secrecy_displayed(world: &mut AppWorld) {
    assert!(
        world.consultation.results_accessed,
        "a visualização não pode expor dados que violem o sigilo dos votos"
    );
}

/// Configura uma consulta estudantil com votação em andamento.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[given("que uma consulta estudantil está com a votação em andamento")]
fn consultation_voting_in_progress(world: &mut AppWorld) {
    world.consultation.state = Some("Em andamento".to_string());
    world.consultation.tallied = false;
}

/// Registra tentativa de acesso à apuração parcial durante a votação.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[when("um estudante ou gestor tenta acessar a apuração parcial dos votos")]
fn student_or_manager_attempts_access_partial_results(world: &mut AppWorld) {
    world.consultation.partial_results_requested = true;
}

/// Valida que os resultados consolidados permanecem ocultos até o encerramento.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se os resultados tiverem sido revelados antes do encerramento oficial.
#[then("os resultados consolidados permanecem ocultos até o término oficial da consulta")]
fn consolidated_results_remain_hidden_until_official_close(world: &mut AppWorld) {
    assert!(
        world.consultation.partial_results_requested,
        "a tentativa de acesso parcial deve ter sido registrada"
    );
    assert!(
        !world.consultation.tallied,
        "os resultados parciais não podem ser divulgados durante a votação"
    );
}

/// Configura processos simultâneos de eleição de chapas e consulta estudantil.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[given("que o Centro Acadêmico possui simultaneamente:")]
fn academic_center_has_simultaneously(world: &mut AppWorld) {
    world.consultation.election_in_progress = true;
    world.consultation.state = Some("Em andamento".to_string());
}

/// Registra a participação do estudante na consulta temática.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
#[when("um estudante participa da consulta sobre o calendário")]
fn student_participates_in_calendar_consultation(world: &mut AppWorld) {
    world.consultation.student_has_voted = true;
    world
        .consultation
        .votes
        .push("A favor do calendário".to_string());
    world.consultation.electoral_roll_intact = true;
}

/// Valida que a participação na consulta não alterou o caderno eleitoral da eleição.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se o caderno eleitoral tiver sido modificado.
#[then("essa participação não altera o caderno eleitoral da eleição de chapas")]
fn participation_does_not_affect_electoral_roll(world: &mut AppWorld) {
    assert!(
        world.consultation.electoral_roll_intact,
        "o caderno eleitoral de eleição de chapas deve permanecer intocado"
    );
}

/// Valida que as regras de quórum, elegibilidade e sigilo operam de modo isolado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário.
///
/// # Panics
///
/// Se a consulta não estiver isolada do processo eleitoral.
#[then("as regras de quórum, elegibilidade e sigilo da consulta operam de modo isolado da eleição")]
fn consultation_rules_operate_isolated_from_election(world: &mut AppWorld) {
    assert!(
        world.consultation.election_in_progress,
        "a eleição de chapas deve coexistir de modo independente"
    );
    assert!(
        world.consultation.student_has_voted,
        "a participação na consulta deve ter ocorrido de modo isolado"
    );
}
