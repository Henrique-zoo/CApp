//! Steps automatizados de controle opcional de capacidade e limite de vagas em eventos (#104).
//!
//! Cobre os cenários MVP de `features/controle_vagas_eventos.feature`:
//! - definição do limite opcional de vagas ao cadastrar um evento;
//! - inscrição em evento sem limite de vagas;
//! - inscrição quando ainda há vaga disponível;
//! - recusa de inscrição direta quando todas as vagas estão ocupadas (`event_full`);
//! - liberação de vaga após cancelamento;
//! - proteção sob concorrência quando duas pessoas disputam simultaneamente a última vaga.
//!
//! Os cenários pós-MVP da fila de espera (#279) permanecem marcados com `@pending_backend`
//! e possuem seus steps registrados com [`pending_step`] para preservar a entrega incremental.

use std::sync::Arc;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use backend::{
    api::{router::create_router, state::AppState},
    modules::events::{
        api::{EventDetailResponse, EventErrorResponse, EventRegistrationResponse, EventResponse},
        application::{EventRepository, EventService, RegisterInEventCommand},
        domain::{EventPublicationStatus, RegistrationStatus},
        infrastructure::{PostgresEventAuthorizationService, PostgresEventRepository},
    },
};
use chrono::{Duration, Utc};
use cucumber::{given, then, when};
use sqlx::{FromRow, PgPool};
use tower::ServiceExt;
use uuid::Uuid;

use super::{
    consulta_inscricao_eventos::{
        DEFAULT_EVENT_AUTHOR_ID, ensure_viewer_context, find_event_by_title, seed_event_in_ca,
        send_authenticated_request, set_active_ca, set_last_referenced_event,
    },
    pending_step,
};
use crate::support::{AppWorld, results::TestResponse};

/// Estado complementar dos cenários de controle de vagas no banco isolado.
#[derive(Debug, Clone, FromRow)]
struct CapacityScenarioState {
    pending_event_title: Option<String>,
    concurrent_created_count: i32,
    concurrent_conflict_count: i32,
}

/// Garante a tabela auxiliar de estado de capacidade do cenário e retorna a linha atual.
async fn ensure_capacity_state(pool: &PgPool) -> CapacityScenarioState {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS bdd_capacity_scenario_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            pending_event_title TEXT NULL,
            concurrent_created_count INTEGER NOT NULL DEFAULT 0,
            concurrent_conflict_count INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await
    .expect("deve criar tabela bdd_capacity_scenario_state no banco do cenário");

    sqlx::query(
        "INSERT INTO bdd_capacity_scenario_state (
            id,
            pending_event_title,
            concurrent_created_count,
            concurrent_conflict_count
         )
         VALUES (1, NULL, 0, 0)
         ON CONFLICT (id) DO NOTHING",
    )
    .execute(pool)
    .await
    .expect("deve inicializar estado de capacidade do cenário");

    sqlx::query_as::<_, CapacityScenarioState>(
        "SELECT
            pending_event_title,
            concurrent_created_count,
            concurrent_conflict_count
         FROM bdd_capacity_scenario_state
         WHERE id = 1",
    )
    .fetch_one(pool)
    .await
    .expect("deve carregar estado de capacidade do cenário")
}

/// Atualiza o limite máximo de vagas do último evento referenciado no cenário.
async fn configure_event_capacity(pool: &PgPool, event_id: Uuid, max_capacity: Option<i64>) {
    let repo = PostgresEventRepository::new(pool.clone());
    let mut event = repo
        .find_by_id(event_id)
        .await
        .expect("deve consultar evento por id")
        .expect("o evento referenciado deve existir");
    event
        .set_signed_max_capacity(max_capacity)
        .expect("o limite de vagas configurado no step Dado deve ser válido");
    repo.update(&event)
        .await
        .expect("deve persistir o limite de vagas atualizado no PostgreSQL");
}

/// Inscreve `count` participantes distintos no evento informado.
async fn seed_active_participants(pool: &PgPool, event_id: Uuid, ca_id: Uuid, count: usize) {
    let service = EventService::new(PostgresEventRepository::new(pool.clone()));
    for _ in 0..count {
        let participant_id = Uuid::new_v4();
        service
            .register_in_event(RegisterInEventCommand::for_academic_center(
                event_id,
                participant_id,
                ca_id,
            ))
            .await
            .expect("deve inscrever participante prévio no evento");
    }
}

/// Conta o número real de inscrições ativas (`confirmed`) do evento na tabela `event_registrations`.
async fn count_active_registrations_in_db(pool: &PgPool, event_id: Uuid) -> u32 {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT
         FROM event_registrations
         WHERE event_id = $1 AND status IN ('confirmed', 'confirmada')",
    )
    .bind(event_id)
    .fetch_one(pool)
    .await
    .expect("deve contar inscrições ativas na tabela event_registrations");
    u32::try_from(count).unwrap_or(0)
}

// ============================================================================
// GIVEN STEPS (MVP - Controle de Capacidade)
// ============================================================================

/// Define o Centro Acadêmico para o qual o proponente está cadastrando um evento
/// e concede a permissão necessária ao autor do cenário naquele CA.
#[given(expr = "que estou propondo um evento para o CA {string}")]
#[given(expr = "estou propondo um evento para o CA {string}")]
async fn given_proposing_event_for_ca(world: &mut AppWorld, ca_name: String) {
    let pool = world.database_pool();
    let _ = ensure_capacity_state(pool).await;
    let ca_id = set_active_ca(pool, &ca_name).await;
    PostgresEventAuthorizationService::new(pool.clone())
        .grant_direct_event_permission(ca_id, DEFAULT_EVENT_AUTHOR_ID)
        .await
        .expect("deve conceder permissão de cadastro e gerenciamento de eventos ao autor no CA");
}

/// Registra o título do evento que está sendo cadastrado pelo proponente.
#[given(expr = "estou cadastrando o evento {string}")]
async fn given_registering_event_with_title(world: &mut AppWorld, title: String) {
    let pool = world.database_pool();
    let _ = ensure_capacity_state(pool).await;
    sqlx::query("UPDATE bdd_capacity_scenario_state SET pending_event_title = $1 WHERE id = 1")
        .bind(title.trim())
        .execute(pool)
        .await
        .expect("deve salvar o título do evento em cadastro");
}

/// Garante que o evento referenciado não possua limite máximo de vagas (`max_capacity = None`).
#[given("o evento não possui limite de vagas")]
async fn given_event_has_no_capacity_limit(world: &mut AppWorld) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve existir um evento referenciado no cenário");
    configure_event_capacity(pool, event_id, None).await;
}

/// Configura o limite máximo de vagas do último evento referenciado.
#[given(expr = "o evento possui limite de {int} vagas")]
async fn given_event_has_capacity_limit(world: &mut AppWorld, max_capacity: i64) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve existir um evento referenciado no cenário");
    configure_event_capacity(pool, event_id, Some(max_capacity)).await;
}

/// Semeia um evento publicado no CA ativo já configurado com o limite de vagas informado.
#[given(expr = "que o evento publicado {string} possui limite de {int} vagas")]
async fn given_published_event_has_capacity_limit(
    world: &mut AppWorld,
    title: String,
    max_capacity: i64,
) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event = seed_event_in_ca(
        pool,
        ctx.active_ca_id,
        &title,
        &format!("Descrição oficial de {title}"),
        EventPublicationStatus::Published,
    )
    .await;
    configure_event_capacity(pool, event.id, Some(max_capacity)).await;
}

/// Registra participantes prévios para simular um evento que já possui inscritos.
#[given("já existem participantes inscritos nesse evento")]
async fn given_participants_already_registered_in_that_event(world: &mut AppWorld) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve existir um evento referenciado no cenário");
    seed_active_participants(pool, event_id, ctx.active_ca_id, 2).await;
}

/// Registra a quantidade exata de participantes inscritos no evento nomeado.
#[given(expr = "{int} participantes estão inscritos no evento {string}")]
async fn given_n_participants_registered_in_named_event(
    world: &mut AppWorld,
    count: usize,
    title: String,
) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event = find_event_by_title(pool, ctx.active_ca_id, &title).await;
    set_last_referenced_event(pool, event.id, &event.title).await;
    seed_active_participants(pool, event.id, event.academic_center_id, count).await;
}

/// Registra a quantidade exata de participantes inscritos no último evento referenciado.
#[given(expr = "já possui {int} participantes inscritos")]
async fn given_last_event_already_has_n_participants(world: &mut AppWorld, count: usize) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve existir um evento referenciado no cenário");
    seed_active_participants(pool, event_id, ctx.active_ca_id, count).await;
}

/// Garante que o estudante autenticado do cenário não possua inscrição ativa no último evento referenciado.
#[given("não tenho inscrição ativa nesse evento")]
async fn given_i_do_not_have_active_registration_in_that_event(world: &mut AppWorld) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve existir um evento referenciado no cenário");

    sqlx::query("DELETE FROM event_registrations WHERE event_id = $1 AND participant_id = $2")
        .bind(event_id)
        .bind(ctx.student_id)
        .execute(pool)
        .await
        .expect("deve remover inscrição prévia do estudante autenticado");
}

// ============================================================================
// WHEN STEPS (MVP - Controle de Capacidade)
// ============================================================================

/// Envia `POST /api/v1/events` definindo o limite de vagas configurado pelo proponente.
#[when(expr = "defino o limite de {int} vagas e salvo o evento")]
async fn when_define_capacity_limit_and_save_event(world: &mut AppWorld, max_capacity: i64) {
    let pool = world.database_pool().clone();
    let ctx = ensure_viewer_context(&pool).await;
    let state = ensure_capacity_state(&pool).await;
    let title = state
        .pending_event_title
        .unwrap_or_else(|| "Evento com Limite".to_string());

    let starts_at = Utc::now() + Duration::days(5);
    let ends_at = starts_at + Duration::hours(2);
    let payload = serde_json::json!({
        "academic_center_id": ctx.active_ca_id,
        "author_id": DEFAULT_EVENT_AUTHOR_ID,
        "title": title,
        "description": format!("Descrição oficial de {title}"),
        "location": "Auditório Principal",
        "starts_at": starts_at,
        "ends_at": ends_at,
        "publish_directly": true,
        "max_capacity": max_capacity
    });

    let app = create_router().with_state(AppState::new(pool.clone()));
    world.request_count += 1;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/events")
                .header("Content-Type", "application/json")
                .header("X-User-Id", DEFAULT_EVENT_AUTHOR_ID.to_string())
                .body(Body::from(payload.to_string()))
                .expect("requisição POST /api/v1/events deve ser válida"),
        )
        .await
        .expect("roteador deve processar criação do evento");

    let status = response.status();
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("corpo da resposta deve ser legível");
    let body = String::from_utf8(body_bytes.to_vec()).expect("corpo deve ser UTF-8");

    if status == StatusCode::CREATED
        && let Ok(created) = serde_json::from_str::<EventResponse>(&body)
    {
        set_last_referenced_event(&pool, created.id, &created.title).await;
    }

    world.last_response = Some(TestResponse { status, body });
}

/// Solicita a inscrição direta do estudante autenticado no evento informado.
#[when(expr = "solicito minha inscrição direta no evento {string}")]
async fn when_request_direct_registration_in_event(world: &mut AppWorld, title: String) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event = find_event_by_title(pool, ctx.active_ca_id, &title).await;
    set_last_referenced_event(pool, event.id, &event.title).await;

    let uri = format!(
        "/api/v1/academic-centers/{}/events/{}/registrations",
        event.academic_center_id, event.id
    );
    send_authenticated_request(world, "POST", &uri, ctx.student_id).await;
}

/// Cancela a inscrição ativa de um dos participantes já inscritos no evento informado via `DELETE`.
#[when(expr = "um participante cancela sua inscrição no evento {string}")]
async fn when_a_participant_cancels_registration_in_event(world: &mut AppWorld, title: String) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event = find_event_by_title(pool, ctx.active_ca_id, &title).await;
    set_last_referenced_event(pool, event.id, &event.title).await;

    let participant_id: Uuid = sqlx::query_scalar(
        "SELECT participant_id
         FROM event_registrations
         WHERE event_id = $1 AND status IN ('confirmed', 'confirmada')
         ORDER BY registered_at ASC
         LIMIT 1",
    )
    .bind(event.id)
    .fetch_one(pool)
    .await
    .expect("deve existir ao menos um participante inscrito no evento para cancelar");

    let uri = format!(
        "/api/v1/academic-centers/{}/events/{}/registrations",
        event.academic_center_id, event.id
    );
    send_authenticated_request(world, "DELETE", &uri, participant_id).await;
}

/// Executa duas requisições HTTP `POST .../registrations` simultâneas disputando a última vaga disponível.
#[when("duas pessoas sem inscrição solicitam simultaneamente a última vaga")]
async fn when_two_people_simultaneously_request_last_spot(world: &mut AppWorld) {
    let pool = world.database_pool().clone();
    let _ = ensure_capacity_state(&pool).await;
    let ctx = ensure_viewer_context(&pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve existir um evento referenciado no cenário");
    let ca_id = ctx.active_ca_id;

    let app = create_router().with_state(AppState::new(pool.clone()));
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let mut handles = Vec::with_capacity(2);

    for _ in 0..2 {
        let router = app.clone();
        let bar = Arc::clone(&barrier);
        let candidate_id = Uuid::new_v4();
        let uri = format!("/api/v1/academic-centers/{ca_id}/events/{event_id}/registrations");
        handles.push(tokio::spawn(async move {
            bar.wait().await;
            let response = router
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(uri)
                        .header("X-User-Id", candidate_id.to_string())
                        .body(Body::empty())
                        .expect("requisição concorrente deve ser válida"),
                )
                .await
                .expect("roteador deve processar requisição concorrente");
            response.status()
        }));
    }

    let mut created_count = 0i32;
    let mut conflict_count = 0i32;
    for handle in handles {
        match handle.await.expect("tarefa concorrente deve concluir") {
            StatusCode::CREATED => created_count += 1,
            StatusCode::CONFLICT => conflict_count += 1,
            other => panic!("status HTTP inesperado na disputa simultânea pela vaga: {other}"),
        }
    }

    sqlx::query(
        "UPDATE bdd_capacity_scenario_state
         SET concurrent_created_count = $1,
             concurrent_conflict_count = $2
         WHERE id = 1",
    )
    .bind(created_count)
    .bind(conflict_count)
    .execute(&pool)
    .await
    .expect("deve registrar contadores de concorrência no estado do cenário");
}

// ============================================================================
// THEN STEPS (MVP - Controle de Capacidade)
// ============================================================================

/// Verifica que o evento foi persistido e retornado com o limite máximo de vagas esperado.
#[then(expr = "o evento é salvo com limite de {int} vagas")]
async fn then_event_is_saved_with_capacity_limit(world: &mut AppWorld, expected_limit: u32) {
    let response = world
        .last_response
        .as_ref()
        .expect("uma resposta HTTP de criação do evento deve ter sido capturada");
    assert_eq!(response.status, StatusCode::CREATED);
    let created: EventResponse =
        serde_json::from_str(&response.body).expect("o corpo deve ser um EventResponse");
    assert_eq!(created.max_capacity, Some(expected_limit));

    let pool = world.database_pool();
    let repo = PostgresEventRepository::new(pool.clone());
    let persisted = repo
        .find_by_id(created.id)
        .await
        .expect("deve consultar evento persistido")
        .expect("o evento recém-salvo deve existir no PostgreSQL");
    assert_eq!(persisted.registration.max_capacity, Some(expected_limit));
}

/// Verifica que o limite configurado passa a governar a disponibilidade de vagas para novas inscrições.
#[then("o limite passa a ser considerado nas novas inscrições")]
async fn then_capacity_limit_is_considered_for_new_registrations(world: &mut AppWorld) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve existir um evento referenciado no cenário");

    let uri = format!(
        "/api/v1/academic-centers/{}/events/{}",
        ctx.active_ca_id, event_id
    );
    let detail: EventDetailResponse = world.get_json(&uri).await;
    let limit = detail
        .max_capacity
        .expect("o evento deve expor o limite máximo de vagas configurado");
    assert!(detail.requires_registration);
    assert!(detail.has_available_spots);
    assert_eq!(detail.remaining_spots, Some(limit));
}

/// Verifica que a inscrição ativa do estudante autenticado foi registrada com sucesso.
#[then("minha inscrição ativa é registrada")]
async fn then_my_active_registration_is_recorded(world: &mut AppWorld) {
    let response = world
        .last_response
        .as_ref()
        .expect("uma resposta HTTP de inscrição deve ter sido capturada");
    assert_eq!(response.status, StatusCode::CREATED);
    let payload: EventRegistrationResponse = serde_json::from_str(&response.body)
        .expect("o corpo deve ser um EventRegistrationResponse");
    assert_eq!(payload.status, RegistrationStatus::Confirmed);

    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve existir um evento referenciado no cenário");

    let repo = PostgresEventRepository::new(pool.clone());
    let registration = repo
        .find_registration(event_id, ctx.student_id)
        .await
        .expect("deve consultar inscrição no repositório")
        .expect("a inscrição ativa do estudante deve existir no PostgreSQL");
    assert!(registration.is_active());
}

/// Verifica que a inscrição em evento sem limite de vagas não foi recusada por capacidade.
#[then("o aplicativo não recusa a inscrição por limite de capacidade")]
async fn then_app_does_not_reject_registration_for_capacity(world: &mut AppWorld) {
    let response = world
        .last_response
        .as_ref()
        .expect("uma resposta HTTP de inscrição deve ter sido capturada");
    assert_eq!(
        response.status,
        StatusCode::CREATED,
        "a inscrição não deveria ter sido recusada por capacidade: {}",
        response.body
    );
}

/// Verifica que nenhuma inscrição ativa foi registrada para o estudante autenticado.
#[then("minha inscrição ativa não é registrada")]
async fn then_my_active_registration_is_not_recorded(world: &mut AppWorld) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve existir um evento referenciado no cenário");

    let repo = PostgresEventRepository::new(pool.clone());
    let registration = repo
        .find_registration(event_id, ctx.student_id)
        .await
        .expect("deve consultar inscrição no repositório");
    assert!(
        registration.is_none(),
        "o estudante não deveria ter inscrição ativa registrada quando o evento está lotado"
    );
}

/// Verifica o total atual de participantes com inscrição ativa no evento.
#[then(expr = "o evento passa a ter {int} participantes inscritos")]
#[then(expr = "o evento continua com {int} participantes inscritos")]
#[then(expr = "o evento fica com {int} participantes inscritos")]
async fn then_event_has_n_active_participants(world: &mut AppWorld, expected_count: u32) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve existir um evento referenciado no cenário");

    let repo = PostgresEventRepository::new(pool.clone());
    let event = repo
        .find_by_id(event_id)
        .await
        .expect("deve consultar evento por id")
        .expect("o evento deve existir");
    assert_eq!(event.registration.confirmed_registrations, expected_count);

    let active_in_table = count_active_registrations_in_db(pool, event_id).await;
    assert_eq!(active_in_table, expected_count);
}

/// Verifica que a API recusou a inscrição direta informando que o evento está lotado (`event_full`).
#[then("o aplicativo informa que o evento está lotado")]
async fn then_app_reports_event_is_full(world: &mut AppWorld) {
    let response = world
        .last_response
        .as_ref()
        .expect("uma resposta HTTP de lotação deve ter sido capturada");
    assert_eq!(response.status, StatusCode::CONFLICT);
    let error: EventErrorResponse =
        serde_json::from_str(&response.body).expect("o corpo deve ser um EventErrorResponse");
    assert_eq!(error.code, "event_full");
    assert!(
        error.message.contains("lotado"),
        "a mensagem deveria informar que o evento está lotado: {}",
        error.message
    );
}

/// Verifica que o evento passou a ter exatamente 1 vaga disponível após o cancelamento.
#[then("o evento passa a ter uma vaga disponível")]
async fn then_event_has_one_available_spot(world: &mut AppWorld) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve existir um evento referenciado no cenário");

    let repo = PostgresEventRepository::new(pool.clone());
    let event = repo
        .find_by_id(event_id)
        .await
        .expect("deve consultar evento por id")
        .expect("o evento deve existir");
    assert!(event.registration.has_available_spots());
    assert_eq!(event.registration.remaining_spots(), Some(1));
}

/// Verifica que o total de participantes inscritos não ultrapassa o limite máximo configurado.
#[then("o número de participantes inscritos não supera o limite")]
#[then("a capacidade não é ultrapassada")]
async fn then_registered_participants_do_not_exceed_limit(world: &mut AppWorld) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve existir um evento referenciado no cenário");

    let repo = PostgresEventRepository::new(pool.clone());
    let event = repo
        .find_by_id(event_id)
        .await
        .expect("deve consultar evento por id")
        .expect("o evento deve existir");
    let limit = event
        .registration
        .max_capacity
        .expect("o evento deve possuir limite de vagas configurado");

    let active_in_table = count_active_registrations_in_db(pool, event_id).await;
    assert!(
        event.registration.confirmed_registrations <= limit && active_in_table <= limit,
        "o total de inscritos ({active_in_table}) não pode superar o limite ({limit})"
    );
}

/// Verifica que, na disputa simultânea pela última vaga, apenas uma nova inscrição ativa foi registrada.
#[then("somente uma nova inscrição ativa é registrada")]
async fn then_only_one_new_active_registration_is_recorded(world: &mut AppWorld) {
    let pool = world.database_pool();
    let state = ensure_capacity_state(pool).await;
    assert_eq!(
        state.concurrent_created_count, 1,
        "somente 1 das requisições simultâneas deveria obter 201 Created"
    );
    assert_eq!(
        state.concurrent_conflict_count, 1,
        "a outra requisição simultânea deveria receber 409 Conflict (event_full)"
    );
}

// ============================================================================
// PENDING STEPS (Pós-MVP - Fila de Espera #279)
// ============================================================================

/// Pendente (#279): não tenho inscrição ativa nem participação na fila desse evento.
#[given("não tenho inscrição ativa nem participação na fila desse evento")]
fn given_no_active_registration_or_waitlist(_world: &mut AppWorld) {
    pending_step("não tenho inscrição ativa nem participação na fila desse evento");
}

/// Pendente (#279): que estou aguardando na fila de espera do evento {string}.
#[given(expr = "que estou aguardando na fila de espera do evento {string}")]
fn given_waiting_in_event_waitlist(_world: &mut AppWorld, _title: String) {
    pending_step("que estou aguardando na fila de espera do evento {string}");
}

/// Pendente (#279): que estou inscrito no evento publicado {string}.
#[given(expr = "que estou inscrito no evento publicado {string}")]
fn given_que_estou_inscrito_no_evento_publicado(_world: &mut AppWorld, _title: String) {
    pending_step("que estou inscrito no evento publicado {string}");
}

/// Pendente (#279): existe um participante elegível aguardando na fila.
#[given("existe um participante elegível aguardando na fila")]
fn given_eligible_participant_in_waitlist(_world: &mut AppWorld) {
    pending_step("existe um participante elegível aguardando na fila");
}

/// Pendente (#279): a política de convocação da fila está definida para o evento.
#[given("a política de convocação da fila está definida para o evento")]
fn given_waitlist_policy_defined(_world: &mut AppWorld) {
    pending_step("a política de convocação da fila está definida para o evento");
}

/// Pendente (#279): 1 participante permanece inscrito após o cancelamento que liberou uma vaga.
#[given("1 participante permanece inscrito após o cancelamento que liberou uma vaga")]
fn given_one_participant_remains_after_cancellation(_world: &mut AppWorld) {
    pending_step("1 participante permanece inscrito após o cancelamento que liberou uma vaga");
}

/// Pendente (#279): 2 participantes elegíveis aguardam na fila de espera.
#[given("2 participantes elegíveis aguardam na fila de espera")]
fn given_two_eligible_participants_in_waitlist(_world: &mut AppWorld) {
    pending_step("2 participantes elegíveis aguardam na fila de espera");
}

/// Pendente (#279): solicito entrar na fila de espera do evento {string}.
#[when(expr = "solicito entrar na fila de espera do evento {string}")]
fn when_request_join_event_waitlist(_world: &mut AppWorld, _title: String) {
    pending_step("solicito entrar na fila de espera do evento {string}");
}

/// Pendente (#279): solicito sair da fila de espera desse evento.
#[when("solicito sair da fila de espera desse evento")]
fn when_request_leave_waitlist(_world: &mut AppWorld) {
    pending_step("solicito sair da fila de espera desse evento");
}

/// Pendente (#279): solicito entrar na fila de espera desse evento.
#[when("solicito entrar na fila de espera desse evento")]
fn when_request_join_that_event_waitlist(_world: &mut AppWorld) {
    pending_step("solicito entrar na fila de espera desse evento");
}

/// Pendente (#279): as tentativas de ocupar a única vaga disponível são processadas simultaneamente.
#[when("as tentativas de ocupar a única vaga disponível são processadas simultaneamente")]
fn when_simultaneous_attempts_to_occupy_released_spot(_world: &mut AppWorld) {
    pending_step("as tentativas de ocupar a única vaga disponível são processadas simultaneamente");
}

/// Pendente (#279): minha participação na fila de espera é registrada.
#[then("minha participação na fila de espera é registrada")]
fn then_my_waitlist_entry_is_recorded(_world: &mut AppWorld) {
    pending_step("minha participação na fila de espera é registrada");
}

/// Pendente (#279): o aplicativo informa que estou aguardando uma vaga.
#[then("o aplicativo informa que estou aguardando uma vaga")]
fn then_app_reports_waiting_for_spot(_world: &mut AppWorld) {
    pending_step("o aplicativo informa que estou aguardando uma vaga");
}

/// Pendente (#279): o número de inscrições ativas permanece igual ao limite.
#[then("o número de inscrições ativas permanece igual ao limite")]
fn then_active_registrations_remain_equal_to_limit(_world: &mut AppWorld) {
    pending_step("o número de inscrições ativas permanece igual ao limite");
}

/// Pendente (#279): minha participação na fila deixa de estar ativa.
#[then("minha participação na fila deixa de estar ativa")]
fn then_my_waitlist_entry_is_no_longer_active(_world: &mut AppWorld) {
    pending_step("minha participação na fila deixa de estar ativa");
}

/// Pendente (#279): não sou considerado para futuras vagas liberadas nessa fila.
#[then("não sou considerado para futuras vagas liberadas nessa fila")]
fn then_not_considered_for_future_spots(_world: &mut AppWorld) {
    pending_step("não sou considerado para futuras vagas liberadas nessa fila");
}

/// Pendente (#279): minha inscrição ativa é preservada.
#[then("minha inscrição ativa é preservada")]
fn then_my_active_registration_is_preserved(_world: &mut AppWorld) {
    pending_step("minha inscrição ativa é preservada");
}

/// Pendente (#279): não sou registrado na fila de espera desse evento.
#[then("não sou registrado na fila de espera desse evento")]
fn then_not_registered_in_waitlist(_world: &mut AppWorld) {
    pending_step("não sou registrado na fila de espera desse evento");
}

/// Pendente (#279): a vaga liberada é direcionada a um único participante elegível conforme a política definida.
#[then(
    "a vaga liberada é direcionada a um único participante elegível conforme a política definida"
)]
fn then_released_spot_directed_to_single_eligible_participant(_world: &mut AppWorld) {
    pending_step(
        "a vaga liberada é direcionada a um único participante elegível conforme a política definida",
    );
}

/// Pendente (#279): a mesma vaga não é atribuída simultaneamente a outro participante.
#[then("a mesma vaga não é atribuída simultaneamente a outro participante")]
fn then_same_spot_not_assigned_to_another_participant(_world: &mut AppWorld) {
    pending_step("a mesma vaga não é atribuída simultaneamente a outro participante");
}

/// Pendente (#279): o número de inscrições ativas não supera o limite.
#[then("o número de inscrições ativas não supera o limite")]
fn then_active_registrations_do_not_exceed_limit(_world: &mut AppWorld) {
    pending_step("o número de inscrições ativas não supera o limite");
}

/// Pendente (#279): no máximo um dos participantes recebe a vaga.
#[then("no máximo um dos participantes recebe a vaga")]
fn then_at_most_one_participant_receives_spot(_world: &mut AppWorld) {
    pending_step("no máximo um dos participantes recebe a vaga");
}

/// Pendente (#279): a mesma vaga não é confirmada para os dois participantes.
#[then("a mesma vaga não é confirmada para os dois participantes")]
fn then_same_spot_not_confirmed_for_both_participants(_world: &mut AppWorld) {
    pending_step("a mesma vaga não é confirmada para os dois participantes");
}
