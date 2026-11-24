//! Steps automatizados de visualização da agenda pública, consulta de detalhes,
//! inscrição e cancelamento de inscrição em eventos (#103).
//!
//! Cobre integralmente as especificações:
//! - `features/visualizacao_eventos.feature`
//! - `features/inscricao_eventos.feature`
//!
//! Executa contra o banco PostgreSQL isolado do cenário (`AppWorld::database_pool`)
//! e o roteador HTTP real do backend (`backend::api::router::create_router`),
//! verificando persistência, códigos de status e contratos JSON de resposta.

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use backend::{
    api::{router::create_router, state::AppState},
    modules::events::{
        api::{EventDetailResponse, EventErrorResponse, EventRegistrationResponse, EventResponse},
        application::{EventRepository, EventService, RegisterInEventCommand},
        domain::{
            Event, EventPublicationStatus, EventRegistration, EventRegistrationInfo,
            RegistrationStatus,
        },
        infrastructure::PostgresEventRepository,
    },
};
use chrono::{Duration, Utc};
use cucumber::{given, then, when};
use sqlx::{FromRow, PgPool};
use tower::ServiceExt;
use uuid::Uuid;

use crate::support::{AppWorld, results::TestResponse};

/// Identificador determinístico do estudante autenticado padrão nos cenários de consulta e inscrição.
const DEFAULT_AUTHENTICATED_STUDENT_ID: Uuid =
    Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0103);

/// Identificador determinístico do autor institucional padrão dos eventos semeados nos cenários.
const DEFAULT_EVENT_AUTHOR_ID: Uuid = Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0900);

/// Identificador determinístico padrão do CA inicial caso o cenário não declare outro explicitamente antes.
const DEFAULT_CA_NAME: &str = "CA de Computação";

/// Estado auxiliar do cenário persistido no banco isolado.
#[derive(Debug, Clone, FromRow)]
struct ViewerScenarioContext {
    student_id: Uuid,
    active_ca_id: Uuid,
    active_ca_name: String,
    last_event_id: Option<Uuid>,
    snapshot_registration_count: i64,
}

/// Deriva um UUID determinístico e estável a partir do nome do Centro Acadêmico.
fn ca_uuid_from_name(ca_name: &str) -> Uuid {
    let mut acc: u128 = 0xcbf2_9ce4_8422_2325;
    for byte in ca_name.trim().as_bytes() {
        acc ^= u128::from(*byte);
        acc = acc.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let high = 0x550e_8400_e29b_41d4_u128 << 64;
    let low = (acc & 0x3fff_ffff_ffff_ffff) | 0x8000_0000_0000_0000;
    Uuid::from_u128(high | low)
}

/// Garante a tabela auxiliar de contexto do cenário e retorna o estado atual.
async fn ensure_viewer_context(pool: &PgPool) -> ViewerScenarioContext {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS bdd_viewer_scenario_context (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            student_id UUID NOT NULL,
            active_ca_id UUID NOT NULL,
            active_ca_name TEXT NOT NULL,
            last_event_id UUID NULL,
            last_event_title TEXT NULL,
            snapshot_registration_count BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await
    .expect("deve criar tabela bdd_viewer_scenario_context no banco do cenário");

    let default_ca_id = ca_uuid_from_name(DEFAULT_CA_NAME);
    sqlx::query(
        "INSERT INTO bdd_viewer_scenario_context (
            id,
            student_id,
            active_ca_id,
            active_ca_name,
            last_event_id,
            last_event_title,
            snapshot_registration_count
         )
         VALUES (1, $1, $2, $3, NULL, NULL, 0)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(DEFAULT_AUTHENTICATED_STUDENT_ID)
    .bind(default_ca_id)
    .bind(DEFAULT_CA_NAME)
    .execute(pool)
    .await
    .expect("deve inicializar contexto padrão de visualização e inscrição");

    sqlx::query_as::<_, ViewerScenarioContext>(
        "SELECT
            student_id,
            active_ca_id,
            active_ca_name,
            last_event_id,
            snapshot_registration_count
         FROM bdd_viewer_scenario_context
         WHERE id = 1",
    )
    .fetch_one(pool)
    .await
    .expect("deve carregar o contexto do cenário")
}

/// Atualiza o CA ativo no contexto do cenário.
async fn set_active_ca(pool: &PgPool, ca_name: &str) -> Uuid {
    let _ = ensure_viewer_context(pool).await;
    let ca_id = ca_uuid_from_name(ca_name);
    sqlx::query(
        "UPDATE bdd_viewer_scenario_context
         SET active_ca_id = $1,
             active_ca_name = $2
         WHERE id = 1",
    )
    .bind(ca_id)
    .bind(ca_name.trim())
    .execute(pool)
    .await
    .expect("deve atualizar o CA ativo do cenário");
    ca_id
}

/// Registra o último evento referenciado pelos steps `Dado` para steps subsequentes como `nesse evento`.
async fn set_last_referenced_event(pool: &PgPool, event_id: Uuid, title: &str) {
    let _ = ensure_viewer_context(pool).await;
    sqlx::query(
        "UPDATE bdd_viewer_scenario_context
         SET last_event_id = $1,
             last_event_title = $2
         WHERE id = 1",
    )
    .bind(event_id)
    .bind(title)
    .execute(pool)
    .await
    .expect("deve atualizar o último evento referenciado no cenário");
}

/// Salva um retrato da quantidade de inscrições ativas no evento para verificações de inalterabilidade.
async fn snapshot_event_registrations(pool: &PgPool, event_id: Uuid) {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT
         FROM event_registrations
         WHERE event_id = $1 AND status IN ('confirmed', 'confirmada')",
    )
    .bind(event_id)
    .fetch_one(pool)
    .await
    .expect("deve contar inscrições ativas do evento");

    sqlx::query(
        "UPDATE bdd_viewer_scenario_context
         SET snapshot_registration_count = $1
         WHERE id = 1",
    )
    .bind(count)
    .execute(pool)
    .await
    .expect("deve salvar snapshot de inscrições do evento");
}

/// Cria ou localiza um evento com o título e status informados no Centro Acadêmico especificado.
async fn seed_event_in_ca(
    pool: &PgPool,
    ca_id: Uuid,
    title: &str,
    description: &str,
    status: EventPublicationStatus,
) -> Event {
    let existing_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM events WHERE academic_center_id = $1 AND title = $2 LIMIT 1",
    )
    .bind(ca_id)
    .bind(title)
    .fetch_optional(pool)
    .await
    .expect("deve consultar existência prévia do evento no CA");

    let repo = PostgresEventRepository::new(pool.clone());
    if let Some(id) = existing_id
        && let Some(mut existing) = repo
            .find_by_id(id)
            .await
            .expect("deve carregar evento existente")
    {
        existing.status = status;
        existing.description = description.to_string();
        existing.published_at = if status == EventPublicationStatus::Published {
            Some(Utc::now())
        } else {
            None
        };
        existing.updated_at = Utc::now();
        let saved = repo
            .update(&existing)
            .await
            .expect("deve atualizar evento semeado");
        set_last_referenced_event(pool, saved.id, &saved.title).await;
        return saved;
    }

    let now = Utc::now();
    let starts_at = now + Duration::days(5);
    let ends_at = starts_at + Duration::hours(2);
    let published_at = if status == EventPublicationStatus::Published {
        Some(now)
    } else {
        None
    };

    let event = Event::new(
        Uuid::new_v4(),
        ca_id,
        DEFAULT_EVENT_AUTHOR_ID,
        title,
        description,
        "Auditório Principal",
        starts_at,
        ends_at,
        status,
        EventRegistrationInfo::unrestricted(),
        published_at,
        now,
        now,
    )
    .expect("dados do evento semeado devem ser válidos");

    let created = repo
        .create(&event)
        .await
        .expect("deve persistir evento semeado no PostgreSQL");
    set_last_referenced_event(pool, created.id, &created.title).await;
    created
}

/// Localiza um evento pelo título no CA ativo (ou em qualquer CA caso não esteja no ativo).
async fn find_event_by_title(pool: &PgPool, active_ca_id: Uuid, title: &str) -> Event {
    let id_in_active: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM events WHERE academic_center_id = $1 AND title = $2 LIMIT 1",
    )
    .bind(active_ca_id)
    .bind(title)
    .fetch_optional(pool)
    .await
    .expect("deve buscar evento pelo título no CA ativo");

    let event_id = match id_in_active {
        Some(id) => id,
        None => sqlx::query_scalar::<_, Uuid>("SELECT id FROM events WHERE title = $1 LIMIT 1")
            .bind(title)
            .fetch_one(pool)
            .await
            .unwrap_or_else(|_| panic!("deve existir o evento '{title}' semeado no cenário")),
    };

    let repo = PostgresEventRepository::new(pool.clone());
    repo.find_by_id(event_id)
        .await
        .expect("deve consultar evento por id")
        .expect("o evento localizado deve existir")
}

/// Envia uma requisição HTTP autenticada ao roteador real do backend e armazena a resposta em `world.last_response`.
async fn send_authenticated_request(world: &mut AppWorld, method: &str, uri: &str, user_id: Uuid) {
    let pool = world.database_pool().clone();
    let app = create_router().with_state(AppState::new(pool));
    world.request_count += 1;

    let response = app
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("X-User-Id", user_id.to_string())
                .body(Body::empty())
                .expect("requisição HTTP de teste deve ser válida"),
        )
        .await
        .expect("roteador do backend deve processar a requisição");

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("corpo da resposta deve ser legível");

    world.last_response = Some(TestResponse {
        status,
        body: String::from_utf8(body.to_vec()).expect("corpo da resposta deve ser UTF-8"),
    });
}

// ============================================================================
// GIVEN STEPS
// ============================================================================

/// Inicializa o contexto autenticado do estudante no CApp.
#[given("que estou autenticado no CApp")]
async fn given_authenticated_in_capp(world: &mut AppWorld) {
    let _ = ensure_viewer_context(world.database_pool()).await;
}

/// Prepara o contexto de consulta da agenda pública de eventos.
#[given("estou consultando a agenda pública de eventos")]
async fn given_consulting_public_event_schedule(world: &mut AppWorld) {
    let _ = ensure_viewer_context(world.database_pool()).await;
}

/// Define o Centro Acadêmico ativo na sessão de consulta do estudante.
#[given(expr = "que o CA ativo é {string}")]
async fn given_active_ca_is(world: &mut AppWorld, ca_name: String) {
    let _ = set_active_ca(world.database_pool(), &ca_name).await;
}

/// Semeia dois eventos publicados no CA ativo.
#[given(expr = "existem os eventos publicados {string} e {string} nesse CA")]
async fn given_two_published_events_exist_in_active_ca(
    world: &mut AppWorld,
    first_title: String,
    second_title: String,
) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    seed_event_in_ca(
        pool,
        ctx.active_ca_id,
        &first_title,
        &format!("Descrição oficial de {first_title}"),
        EventPublicationStatus::Published,
    )
    .await;
    seed_event_in_ca(
        pool,
        ctx.active_ca_id,
        &second_title,
        &format!("Descrição oficial de {second_title}"),
        EventPublicationStatus::Published,
    )
    .await;
}

/// Semeia um evento publicado no CA ativo.
#[given(expr = "existe o evento publicado {string} nesse CA")]
async fn given_published_event_exists_in_active_ca(world: &mut AppWorld, title: String) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    seed_event_in_ca(
        pool,
        ctx.active_ca_id,
        &title,
        &format!("Descrição oficial de {title}"),
        EventPublicationStatus::Published,
    )
    .await;
}

/// Semeia um evento publicado em um Centro Acadêmico específico.
#[given(expr = "existe o evento publicado {string} no CA {string}")]
async fn given_published_event_exists_in_named_ca(
    world: &mut AppWorld,
    title: String,
    ca_name: String,
) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let target_ca_id = ca_uuid_from_name(&ca_name);

    // Se o cenário não havia configurado outro CA ativo antes, alinha o CA ativo ao CA do evento principal
    if ctx.last_event_id.is_none() && ctx.active_ca_name == DEFAULT_CA_NAME {
        set_active_ca(pool, &ca_name).await;
    }

    seed_event_in_ca(
        pool,
        target_ca_id,
        &title,
        &format!("Descrição oficial de {title}"),
        EventPublicationStatus::Published,
    )
    .await;
}

/// Semeia uma proposta de evento pendente de homologação no CA ativo.
#[given(expr = "existe o evento {string} proposto e ainda não homologado nesse CA")]
async fn given_pending_event_exists_in_active_ca(world: &mut AppWorld, title: String) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    seed_event_in_ca(
        pool,
        ctx.active_ca_id,
        &title,
        &format!("Proposta em análise para {title}"),
        EventPublicationStatus::PendingApproval,
    )
    .await;
}

/// Semeia uma proposta de evento pendente de homologação em um CA específico.
#[given(expr = "existe o evento {string} proposto e ainda não homologado no CA {string}")]
async fn given_pending_event_exists_in_named_ca(
    world: &mut AppWorld,
    title: String,
    ca_name: String,
) {
    let pool = world.database_pool();
    let target_ca_id = set_active_ca(pool, &ca_name).await;
    seed_event_in_ca(
        pool,
        target_ca_id,
        &title,
        &format!("Proposta em análise para {title}"),
        EventPublicationStatus::PendingApproval,
    )
    .await;
}

/// Semeia um evento em rascunho (não publicado) no CA ativo.
#[given(expr = "existe o evento {string} não publicado nesse CA")]
async fn given_unpublished_draft_event_exists_in_active_ca(world: &mut AppWorld, title: String) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    seed_event_in_ca(
        pool,
        ctx.active_ca_id,
        &title,
        &format!("Rascunho interno de {title}"),
        EventPublicationStatus::Draft,
    )
    .await;
}

/// Garante que o CA ativo não possua nenhum evento publicado.
#[given("não existem eventos publicados nesse CA")]
async fn given_no_published_events_in_active_ca(world: &mut AppWorld) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    sqlx::query(
        "DELETE FROM events
         WHERE academic_center_id = $1 AND status IN ('published', 'publicado')",
    )
    .bind(ctx.active_ca_id)
    .execute(pool)
    .await
    .expect("deve remover eventos publicados do CA ativo");
}

/// Atualiza a descrição detalhada do último evento semeado no cenário.
#[given(expr = "o evento contém a informação {string}")]
async fn given_last_event_contains_information(world: &mut AppWorld, information: String) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve existir um evento referenciado anteriormente no cenário");

    let repo = PostgresEventRepository::new(pool.clone());
    let mut event = repo
        .find_by_id(event_id)
        .await
        .expect("deve consultar evento no repositório")
        .expect("o evento referenciado deve existir");
    event.description = information;
    event.updated_at = Utc::now();
    repo.update(&event)
        .await
        .expect("deve atualizar informação detalhada do evento");
}

/// Simula indisponibilidade real na consulta ao armazenamento de eventos do CA ativo.
#[given("que não foi possível obter os eventos publicados do CA ativo")]
async fn given_failed_to_load_published_events(world: &mut AppWorld) {
    let pool = world.database_pool();
    let _ = ensure_viewer_context(pool).await;
    sqlx::query("ALTER TABLE events RENAME TO events_temporarily_unavailable")
        .execute(pool)
        .await
        .expect("deve tornar tabela events indisponível no banco isolado do cenário");
}

/// Garante que o estudante autenticado não possua inscrição ativa no último evento referenciado.
#[given("ainda não estou inscrito nesse evento")]
async fn given_not_yet_registered_in_that_event(world: &mut AppWorld) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve haver um evento referenciado no cenário");

    sqlx::query("DELETE FROM event_registrations WHERE event_id = $1 AND participant_id = $2")
        .bind(event_id)
        .bind(ctx.student_id)
        .execute(pool)
        .await
        .expect("deve garantir ausência de inscrição prévia do estudante");
    snapshot_event_registrations(pool, event_id).await;
}

/// Garante que o estudante autenticado já possua uma inscrição ativa no último evento referenciado.
#[given("já estou inscrito nesse evento")]
async fn given_already_registered_in_that_event(world: &mut AppWorld) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve haver um evento referenciado no cenário");

    let service = EventService::new(PostgresEventRepository::new(pool.clone()));
    service
        .register_in_event(RegisterInEventCommand::new(event_id, ctx.student_id))
        .await
        .expect("deve registrar inscrição inicial do estudante no evento");
    snapshot_event_registrations(pool, event_id).await;
}

/// Garante que o evento publicado exista e que o estudante autenticado esteja inscrito nele.
#[given(expr = "estou inscrito no evento publicado {string}")]
async fn given_registered_in_published_event(world: &mut AppWorld, title: String) {
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

    let service = EventService::new(PostgresEventRepository::new(pool.clone()));
    service
        .register_in_event(RegisterInEventCommand::for_academic_center(
            event.id,
            ctx.student_id,
            ctx.active_ca_id,
        ))
        .await
        .expect("deve registrar inscrição prévia ativa do estudante");
    snapshot_event_registrations(pool, event.id).await;
}

/// Garante que o evento publicado exista e que o estudante autenticado não possua inscrição ativa nele.
#[given(expr = "não estou inscrito no evento publicado {string}")]
async fn given_not_registered_in_published_event(world: &mut AppWorld, title: String) {
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

    sqlx::query("DELETE FROM event_registrations WHERE event_id = $1 AND participant_id = $2")
        .bind(event.id)
        .bind(ctx.student_id)
        .execute(pool)
        .await
        .expect("deve garantir que o estudante não está inscrito no evento");
    snapshot_event_registrations(pool, event.id).await;
}

// ============================================================================
// WHEN STEPS
// ============================================================================

/// Consulta a listagem de eventos publicados do CA ativo via `GET /api/v1/events?ca_id=...`.
#[when("consulto a lista de eventos")]
#[when("consulto novamente a lista de eventos")]
async fn when_list_events_for_active_ca(world: &mut AppWorld) {
    let ctx = ensure_viewer_context(world.database_pool()).await;
    let uri = format!("/api/v1/events?ca_id={}", ctx.active_ca_id);
    world.get(&uri).await;
}

/// Altera o Centro Acadêmico ativo na sessão do estudante.
#[when(expr = "altero o CA ativo para {string}")]
async fn when_switch_active_ca(world: &mut AppWorld, ca_name: String) {
    let _ = set_active_ca(world.database_pool(), &ca_name).await;
}

/// Consulta os detalhes de um evento específico no CA ativo via `GET /api/v1/academic-centers/{ca_id}/events/{id}`.
#[when(expr = "consulto os detalhes do evento {string}")]
#[when(expr = "tento consultar os detalhes do evento {string}")]
async fn when_get_event_details_in_active_ca(world: &mut AppWorld, title: String) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event = find_event_by_title(pool, ctx.active_ca_id, &title).await;
    set_last_referenced_event(pool, event.id, &event.title).await;

    let uri = format!(
        "/api/v1/academic-centers/{}/events/{}",
        ctx.active_ca_id, event.id
    );
    world.get(&uri).await;
}

/// Solicita a inscrição do estudante autenticado no evento informado via `POST /api/v1/academic-centers/{ca_id}/events/{id}/registrations`.
#[when(expr = "solicito minha inscrição no evento {string}")]
#[when(expr = "solicito novamente minha inscrição no evento {string}")]
async fn when_request_event_registration(world: &mut AppWorld, title: String) {
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

/// Solicita o cancelamento da inscrição do estudante autenticado no evento informado via `DELETE /api/v1/academic-centers/{ca_id}/events/{id}/registrations`.
#[when(expr = "solicito cancelar minha inscrição no evento {string}")]
async fn when_request_event_registration_cancellation(world: &mut AppWorld, title: String) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event = find_event_by_title(pool, ctx.active_ca_id, &title).await;
    set_last_referenced_event(pool, event.id, &event.title).await;

    let uri = format!(
        "/api/v1/academic-centers/{}/events/{}/registrations",
        event.academic_center_id, event.id
    );
    send_authenticated_request(world, "DELETE", &uri, ctx.student_id).await;
}

// ============================================================================
// THEN STEPS
// ============================================================================

/// Extrai a lista de eventos da última resposta HTTP 200.
fn parse_listed_events(world: &AppWorld) -> Vec<EventResponse> {
    let response = world
        .last_response
        .as_ref()
        .expect("uma resposta HTTP de listagem deve ter sido capturada");
    assert_eq!(
        response.status,
        StatusCode::OK,
        "a listagem de eventos deveria retornar HTTP 200 OK, mas retornou {}: {}",
        response.status,
        response.body
    );
    serde_json::from_str(&response.body).expect("o corpo da listagem deve ser um JSON de eventos")
}

/// Verifica que ambos os eventos informados aparecem na listagem retornada.
#[then(expr = "vejo {string} e {string}")]
async fn then_see_two_events_in_list(
    world: &mut AppWorld,
    first_title: String,
    second_title: String,
) {
    let events = parse_listed_events(world);
    assert!(
        events.iter().any(|event| event.title == first_title),
        "esperava encontrar o evento '{first_title}' na listagem: {events:?}"
    );
    assert!(
        events.iter().any(|event| event.title == second_title),
        "esperava encontrar o evento '{second_title}' na listagem: {events:?}"
    );
}

/// Verifica que o evento informado aparece na listagem retornada.
#[then(expr = "vejo {string}")]
async fn then_see_event_in_list(world: &mut AppWorld, expected_title: String) {
    let events = parse_listed_events(world);
    assert!(
        events.iter().any(|event| event.title == expected_title),
        "esperava encontrar o evento '{expected_title}' na listagem: {events:?}"
    );
}

/// Verifica que o evento informado não aparece na listagem retornada.
#[then(expr = "não vejo {string}")]
async fn then_do_not_see_event_in_list(world: &mut AppWorld, forbidden_title: String) {
    let events = parse_listed_events(world);
    assert!(
        events.iter().all(|event| event.title != forbidden_title),
        "o evento '{forbidden_title}' não deveria aparecer na listagem: {events:?}"
    );
}

/// Verifica que a listagem retornou vazia (nenhum evento publicado no CA ativo).
#[then("a listagem informa que não há eventos publicados")]
async fn then_listing_reports_no_published_events(world: &mut AppWorld) {
    let events = parse_listed_events(world);
    assert!(
        events.is_empty(),
        "a listagem deveria estar vazia quando não há eventos publicados, mas retornou: {events:?}"
    );
}

/// Verifica que o título do evento aparece na resposta de detalhes.
#[then(expr = "vejo o título {string}")]
async fn then_see_event_detail_title(world: &mut AppWorld, expected_title: String) {
    let response = world
        .last_response
        .as_ref()
        .expect("uma resposta HTTP de detalhes deve ter sido capturada");
    assert_eq!(response.status, StatusCode::OK);
    let detail: EventDetailResponse = serde_json::from_str(&response.body)
        .expect("o corpo deve corresponder a EventDetailResponse");
    assert_eq!(detail.title, expected_title);
}

/// Verifica que a informação detalhada (descrição) aparece na resposta de detalhes do evento.
#[then(expr = "vejo a informação {string}")]
async fn then_see_event_detail_information(world: &mut AppWorld, expected_info: String) {
    let response = world
        .last_response
        .as_ref()
        .expect("uma resposta HTTP de detalhes deve ter sido capturada");
    assert_eq!(response.status, StatusCode::OK);
    let detail: EventDetailResponse = serde_json::from_str(&response.body)
        .expect("o corpo deve corresponder a EventDetailResponse");
    assert_eq!(detail.description, expected_info);
}

/// Verifica que os detalhes de um evento não publicado não foram disponibilizados (HTTP 404).
#[then("os detalhes do evento não são disponibilizados")]
async fn then_event_details_are_not_available(world: &mut AppWorld) {
    let response = world
        .last_response
        .as_ref()
        .expect("uma resposta HTTP deve ter sido capturada");
    assert_eq!(response.status, StatusCode::NOT_FOUND);
    let error: EventErrorResponse =
        serde_json::from_str(&response.body).expect("o corpo deve ser um EventErrorResponse");
    assert_eq!(error.code, "not_found");
}

/// Verifica que uma falha ao consultar os eventos foi reportada pela API com erro semântico.
#[then("o aplicativo informa que não foi possível carregar os eventos")]
async fn then_app_reports_failure_to_load_events(world: &mut AppWorld) {
    let response = world
        .last_response
        .as_ref()
        .expect("uma resposta HTTP de erro deve ter sido capturada");
    assert_eq!(response.status, StatusCode::INTERNAL_SERVER_ERROR);
    let error: EventErrorResponse =
        serde_json::from_str(&response.body).expect("o corpo deve ser um EventErrorResponse");
    assert_eq!(error.code, "internal_error");
}

/// Verifica no banco PostgreSQL que a inscrição ativa do estudante foi persistida no evento.
#[then("minha inscrição no evento é registrada")]
async fn then_my_registration_is_persisted(world: &mut AppWorld) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve haver um evento referenciado no cenário");

    let repo = PostgresEventRepository::new(pool.clone());
    let registration = repo
        .find_registration(event_id, ctx.student_id)
        .await
        .expect("deve consultar inscrição no repositório")
        .expect("a inscrição do estudante deve estar registrada no banco");

    assert_eq!(registration.status, RegistrationStatus::Confirmed);
    assert!(registration.is_active());
    assert!(registration.cancelled_at.is_none());
}

/// Verifica que a resposta HTTP confirmou a realização da inscrição.
#[then("o aplicativo informa que a inscrição foi realizada")]
async fn then_app_reports_registration_succeeded(world: &mut AppWorld) {
    let response = world
        .last_response
        .as_ref()
        .expect("uma resposta HTTP de inscrição deve ter sido capturada");
    assert_eq!(response.status, StatusCode::CREATED);
    let payload: EventRegistrationResponse = serde_json::from_str(&response.body)
        .expect("o corpo deve ser um EventRegistrationResponse");
    assert_eq!(payload.status, RegistrationStatus::Confirmed);
    assert!(
        payload.message.to_lowercase().contains("inscrição"),
        "a mensagem deveria confirmar a inscrição: {}",
        payload.message
    );
}

/// Verifica no banco PostgreSQL que nenhuma inscrição foi registrada para o estudante no evento.
#[then("minha inscrição não é registrada")]
async fn then_my_registration_is_not_persisted(world: &mut AppWorld) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve haver um evento referenciado no cenário");

    let repo = PostgresEventRepository::new(pool.clone());
    let registration = repo
        .find_registration(event_id, ctx.student_id)
        .await
        .expect("deve consultar inscrição no repositório");
    assert!(
        registration.is_none(),
        "nenhuma inscrição deveria ter sido registrada para evento não publicado"
    );
}

/// Verifica que a API recusou a inscrição informando que o evento não está disponível para inscrição.
#[then("o aplicativo informa que o evento não está disponível para inscrição")]
async fn then_app_reports_event_not_available_for_registration(world: &mut AppWorld) {
    let response = world
        .last_response
        .as_ref()
        .expect("uma resposta HTTP deve ter sido capturada");
    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    let error: EventErrorResponse =
        serde_json::from_str(&response.body).expect("o corpo deve ser um EventErrorResponse");
    assert_eq!(error.code, "event_not_available");
    assert!(
        error.message.contains("não está disponível para inscrição"),
        "mensagem inesperada: {}",
        error.message
    );
}

/// Verifica que o estudante continua com uma única inscrição ativa no evento.
#[then("minha inscrição continua única")]
async fn then_my_registration_remains_unique(world: &mut AppWorld) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve haver um evento referenciado no cenário");

    let repo = PostgresEventRepository::new(pool.clone());
    let registrations: Vec<EventRegistration> = repo
        .list_registrations_by_event(event_id)
        .await
        .expect("deve listar inscrições do evento")
        .into_iter()
        .filter(|reg| reg.participant_id == ctx.student_id && reg.is_active())
        .collect();

    assert_eq!(
        registrations.len(),
        1,
        "a inscrição ativa do estudante deve permanecer única"
    );

    let event = repo
        .find_by_id(event_id)
        .await
        .expect("deve consultar evento")
        .expect("o evento deve existir");
    assert_eq!(event.registration.confirmed_registrations, 1);
}

/// Verifica que a API informou que o estudante já está inscrito no evento.
#[then("o aplicativo informa que já estou inscrito nesse evento")]
async fn then_app_reports_already_registered(world: &mut AppWorld) {
    let response = world
        .last_response
        .as_ref()
        .expect("uma resposta HTTP deve ter sido capturada");
    assert_eq!(response.status, StatusCode::CONFLICT);
    let error: EventErrorResponse =
        serde_json::from_str(&response.body).expect("o corpo deve ser um EventErrorResponse");
    assert_eq!(error.code, "already_registered");
    assert!(
        error.message.contains("já está inscrito"),
        "mensagem inesperada: {}",
        error.message
    );
}

/// Verifica no PostgreSQL que a inscrição do estudante passou para o estado cancelado (inativa).
#[then("minha inscrição no evento deixa de estar ativa")]
async fn then_my_registration_is_no_longer_active(world: &mut AppWorld) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve haver um evento referenciado no cenário");

    let repo = PostgresEventRepository::new(pool.clone());
    let registration = repo
        .find_registration(event_id, ctx.student_id)
        .await
        .expect("deve consultar inscrição no repositório")
        .expect("o registro da inscrição cancelada deve existir");

    assert_eq!(registration.status, RegistrationStatus::Cancelled);
    assert!(!registration.is_active());
    assert!(registration.cancelled_at.is_some());
}

/// Verifica que a resposta HTTP confirmou a realização do cancelamento da inscrição.
#[then("o aplicativo informa que o cancelamento foi realizado")]
async fn then_app_reports_cancellation_succeeded(world: &mut AppWorld) {
    let response = world
        .last_response
        .as_ref()
        .expect("uma resposta HTTP de cancelamento deve ter sido capturada");
    assert_eq!(response.status, StatusCode::OK);
    let payload: EventRegistrationResponse = serde_json::from_str(&response.body)
        .expect("o corpo deve ser um EventRegistrationResponse");
    assert_eq!(payload.status, RegistrationStatus::Cancelled);
    assert!(
        payload.message.to_lowercase().contains("cancelamento"),
        "a mensagem deveria confirmar o cancelamento: {}",
        payload.message
    );
}

/// Verifica que nenhuma inscrição do evento foi alterada após tentativa inválida de cancelamento.
#[then("nenhuma inscrição é alterada")]
async fn then_no_registration_is_modified(world: &mut AppWorld) {
    let pool = world.database_pool();
    let ctx = ensure_viewer_context(pool).await;
    let event_id = ctx
        .last_event_id
        .expect("deve haver um evento referenciado no cenário");

    let current_active_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT
         FROM event_registrations
         WHERE event_id = $1 AND status IN ('confirmed', 'confirmada')",
    )
    .bind(event_id)
    .fetch_one(pool)
    .await
    .expect("deve contar inscrições ativas atuais do evento");

    assert_eq!(current_active_count, ctx.snapshot_registration_count);

    let repo = PostgresEventRepository::new(pool.clone());
    let student_reg = repo
        .find_registration(event_id, ctx.student_id)
        .await
        .expect("deve consultar inscrição do estudante");
    assert!(student_reg.is_none());
}

/// Verifica que a API informou que não há inscrição ativa para cancelar.
#[then("o aplicativo informa que não há inscrição ativa para cancelar")]
async fn then_app_reports_no_active_registration_to_cancel(world: &mut AppWorld) {
    let response = world
        .last_response
        .as_ref()
        .expect("uma resposta HTTP deve ter sido capturada");
    assert_eq!(response.status, StatusCode::CONFLICT);
    let error: EventErrorResponse =
        serde_json::from_str(&response.body).expect("o corpo deve ser um EventErrorResponse");
    assert_eq!(error.code, "no_active_registration");
    assert!(
        error
            .message
            .contains("não há inscrição ativa para cancelar"),
        "mensagem inesperada: {}",
        error.message
    );
}
