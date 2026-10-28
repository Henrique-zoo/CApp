//! Steps da especificação de criação e gerenciamento de eventos pela gestão (#82 / #101).
//!
//! Implementa o `Contexto` e todas as regras de
//! `features/eventos_ca/criacao_edicao_eventos.feature` sobre o banco PostgreSQL
//! isolado fornecido por [`AppWorld`].

use backend::modules::events::{
    application::{
        CreateEventCommand, EventApplicationError, EventService, PublishEventCommand,
        UpdateEventCommand,
    },
    domain::{EventDomainError, EventRegistrationInfo},
    infrastructure::PostgresEventRepository,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use cucumber::{gherkin::Step, given, then, when};
use sqlx::PgPool;
use uuid::Uuid;

use crate::support::AppWorld;

/// Permissão exigida para publicar e gerenciar eventos diretamente no CA.
const MANAGE_EVENTS_PERMISSION: &str = "Publicar e gerenciar eventos diretamente";

/// Dados de entrada extraídos das tabelas Gherkin de criação/edição de eventos.
#[derive(Debug, Default, Clone)]
struct EventFormInput {
    title: String,
    description: String,
    starts_at: String,
    ends_at: String,
    location: String,
}

impl EventFormInput {
    /// Extrai os campos de uma tabela Gherkin de duas colunas (`campo | valor`).
    fn from_step(step: &Step) -> Self {
        let mut input = Self::default();
        let table = step
            .table
            .as_ref()
            .expect("o step deve conter uma tabela de dados do evento");

        for row in &table.rows {
            if row.len() >= 2 {
                let key = row[0].trim();
                let value = row[1].trim().to_owned();
                match key {
                    "título" => input.title = value,
                    "descrição" => input.description = value,
                    "início" => input.starts_at = value,
                    "término" => input.ends_at = value,
                    "local" => input.location = value,
                    other => panic!("campo desconhecido na tabela de evento: {other}"),
                }
            }
        }
        input
    }
}

/// Garante a estrutura de tabelas necessária aos cenários de eventos no banco isolado.
async fn ensure_events_schema(pool: &PgPool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS academic_center (
            id BIGSERIAL PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            acronym TEXT NOT NULL UNIQUE
        )",
    )
    .execute(pool)
    .await
    .expect("deve criar tabela academic_center no banco do cenário");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS user_profile (
            id BIGSERIAL PRIMARY KEY,
            email TEXT NOT NULL UNIQUE
        )",
    )
    .execute(pool)
    .await
    .expect("deve criar tabela user_profile no banco do cenário");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ca_role (
            id BIGSERIAL PRIMARY KEY,
            ca_id BIGINT NOT NULL REFERENCES academic_center(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            UNIQUE (ca_id, name)
        )",
    )
    .execute(pool)
    .await
    .expect("deve criar tabela ca_role no banco do cenário");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ca_role_permission (
            ca_role_id BIGINT NOT NULL REFERENCES ca_role(id) ON DELETE CASCADE,
            permission TEXT NOT NULL,
            PRIMARY KEY (ca_role_id, permission)
        )",
    )
    .execute(pool)
    .await
    .expect("deve criar tabela ca_role_permission no banco do cenário");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ca_management_member (
            ca_id BIGINT NOT NULL REFERENCES academic_center(id) ON DELETE CASCADE,
            user_id BIGINT NOT NULL REFERENCES user_profile(id) ON DELETE CASCADE,
            ca_role_id BIGINT NOT NULL REFERENCES ca_role(id) ON DELETE RESTRICT,
            PRIMARY KEY (ca_id, user_id)
        )",
    )
    .execute(pool)
    .await
    .expect("deve criar tabela ca_management_member no banco do cenário");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ca_event (
            id BIGSERIAL PRIMARY KEY,
            ca_id BIGINT NOT NULL REFERENCES academic_center(id) ON DELETE CASCADE,
            domain_event_id UUID NULL,
            author_email TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            starts_at TEXT NOT NULL,
            ends_at TEXT NOT NULL,
            location TEXT NOT NULL,
            status TEXT NOT NULL,
            requires_approval BOOLEAN NOT NULL DEFAULT FALSE,
            published_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE (ca_id, title)
        )",
    )
    .execute(pool)
    .await
    .expect("deve criar tabela ca_event no banco do cenário");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS bdd_event_operation_outcome (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            status TEXT NOT NULL,
            reason TEXT
        )",
    )
    .execute(pool)
    .await
    .expect("deve criar tabela bdd_event_operation_outcome no banco do cenário");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS bdd_last_affected_event (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            event_id BIGINT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .expect("deve criar tabela bdd_last_affected_event no banco do cenário");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS bdd_event_snapshot (
            event_id BIGINT PRIMARY KEY,
            ca_id BIGINT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            starts_at TEXT NOT NULL,
            ends_at TEXT NOT NULL,
            location TEXT NOT NULL,
            status TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .expect("deve criar tabela bdd_event_snapshot no banco do cenário");
}

/// Extrai a sigla entre parênteses do nome do CA, ou usa o próprio identificador.
fn extract_ca_acronym(ca_label: &str) -> String {
    if let Some((_, tail)) = ca_label.rsplit_once('(')
        && let Some((acronym, _)) = tail.split_once(')')
    {
        let trimmed = acronym.trim();
        if !trimmed.is_empty() {
            return trimmed.to_owned();
        }
    }
    ca_label.trim().to_owned()
}

/// Obtém o identificador interno de um CA pelo nome completo ou pela sigla.
async fn find_ca_id(pool: &PgPool, ca_identifier: &str) -> i64 {
    let acronym = extract_ca_acronym(ca_identifier);
    sqlx::query_scalar::<_, i64>("SELECT id FROM academic_center WHERE acronym = $1 OR name = $2")
        .bind(&acronym)
        .bind(ca_identifier)
        .fetch_one(pool)
        .await
        .expect("o Centro Acadêmico referenciado deve estar cadastrado")
}

/// Garante que o cargo exista no CA e retorna seu identificador.
async fn ensure_role_in_ca(pool: &PgPool, ca_id: i64, role_name: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO ca_role (ca_id, name)
         VALUES ($1, $2)
         ON CONFLICT (ca_id, name) DO UPDATE SET name = EXCLUDED.name
         RETURNING id",
    )
    .bind(ca_id)
    .bind(role_name)
    .fetch_one(pool)
    .await
    .expect("deve registrar ou localizar o cargo no CA")
}

/// Garante que o perfil do estudante exista e retorna seu identificador.
async fn ensure_user_profile(pool: &PgPool, email: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO user_profile (email)
         VALUES ($1)
         ON CONFLICT (email) DO UPDATE SET email = EXCLUDED.email
         RETURNING id",
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .expect("deve registrar ou localizar o perfil do estudante")
}

/// Verifica se o ator possui a permissão de gerenciar e publicar eventos diretamente no CA.
async fn is_actor_authorized_for_events(pool: &PgPool, ca_id: i64, actor_email: &str) -> bool {
    sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1
            FROM ca_management_member m
            JOIN user_profile u ON u.id = m.user_id
            JOIN ca_role_permission p ON p.ca_role_id = m.ca_role_id
            WHERE m.ca_id = $1 AND u.email = $2 AND p.permission = $3
        )",
    )
    .bind(ca_id)
    .bind(actor_email)
    .bind(MANAGE_EVENTS_PERMISSION)
    .fetch_one(pool)
    .await
    .expect("deve verificar permissão de eventos do ator no CA")
}

/// Registra o resultado da última operação sobre eventos no cenário.
async fn record_event_operation_outcome(pool: &PgPool, status: &str, reason: Option<&str>) {
    sqlx::query(
        "INSERT INTO bdd_event_operation_outcome (id, status, reason)
         VALUES (1, $1, $2)
         ON CONFLICT (id) DO UPDATE SET status = EXCLUDED.status, reason = EXCLUDED.reason",
    )
    .bind(status)
    .bind(reason)
    .execute(pool)
    .await
    .expect("deve registrar resultado da operação de evento");
}

/// Consulta o resultado da última operação sobre eventos no cenário.
async fn fetch_last_event_operation_outcome(pool: &PgPool) -> (String, Option<String>) {
    sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT status, reason FROM bdd_event_operation_outcome WHERE id = 1",
    )
    .fetch_one(pool)
    .await
    .expect("uma operação de evento deve ter sido executada antes da verificação")
}

/// Registra o identificador do último evento manipulado no cenário.
async fn record_last_affected_event(pool: &PgPool, event_id: i64) {
    sqlx::query(
        "INSERT INTO bdd_last_affected_event (id, event_id)
         VALUES (1, $1)
         ON CONFLICT (id) DO UPDATE SET event_id = EXCLUDED.event_id",
    )
    .bind(event_id)
    .execute(pool)
    .await
    .expect("deve registrar o último evento manipulado");
}

/// Obtém o identificador do último evento manipulado no cenário.
async fn fetch_last_affected_event_id(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT event_id FROM bdd_last_affected_event WHERE id = 1")
        .fetch_one(pool)
        .await
        .expect("um evento deve ter sido manipulado antes da verificação")
}

/// Converte o identificador numérico do CA no cenário para um UUID determinístico de domínio.
fn ca_uuid_from_id(ca_id: i64) -> Uuid {
    Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0000 + u128::try_from(ca_id).unwrap_or(1))
}

/// Converte o perfil do usuário no cenário para um UUID determinístico de domínio.
async fn author_uuid_from_email(pool: &PgPool, email: &str) -> Uuid {
    let user_id = ensure_user_profile(pool, email).await;
    Uuid::from_u128(
        0x018f_0000_0000_7000_9000_0000_0000_0000 + u128::try_from(user_id).unwrap_or(1),
    )
}

/// Converte uma string de data/hora Gherkin (`YYYY-MM-DD HH:MM`) para `DateTime<Utc>`.
fn parse_scenario_datetime(raw: &str) -> DateTime<Utc> {
    NaiveDateTime::parse_from_str(raw.trim(), "%Y-%m-%d %H:%M")
        .expect("data/hora do cenário deve seguir o formato YYYY-MM-DD HH:MM")
        .and_utc()
}

/// Traduz erros da camada de aplicação/domínio para os códigos esperados nos cenários BDD.
fn map_application_error_reason(error: &EventApplicationError) -> &'static str {
    match error {
        EventApplicationError::Domain(EventDomainError::EmptyTitle) => "missing:título",
        EventApplicationError::Domain(EventDomainError::EmptyDescription) => "missing:descrição",
        EventApplicationError::Domain(EventDomainError::EmptyLocation) => "missing:local",
        EventApplicationError::Domain(EventDomainError::InvalidEventPeriod { .. }) => {
            "invalid_period"
        }
        EventApplicationError::PermissionDenied => "permission_denied",
        _ => "validation_error",
    }
}

/// Valida e executa a criação de um evento (como publicado ou rascunho) no CA via [`EventService`].
async fn attempt_event_creation(
    pool: &PgPool,
    actor_email: &str,
    ca_identifier: &str,
    input: &EventFormInput,
    target_status: &str,
) -> Result<i64, &'static str> {
    let ca_id = find_ca_id(pool, ca_identifier).await;

    if !is_actor_authorized_for_events(pool, ca_id, actor_email).await {
        record_event_operation_outcome(pool, "rejected", Some("permission_denied")).await;
        return Err("permission_denied");
    }

    if input.title.trim().is_empty() {
        record_event_operation_outcome(pool, "rejected", Some("missing:título")).await;
        return Err("missing:título");
    }
    if input.description.trim().is_empty() {
        record_event_operation_outcome(pool, "rejected", Some("missing:descrição")).await;
        return Err("missing:descrição");
    }
    if input.starts_at.trim().is_empty() {
        record_event_operation_outcome(pool, "rejected", Some("missing:início")).await;
        return Err("missing:início");
    }
    if input.location.trim().is_empty() {
        record_event_operation_outcome(pool, "rejected", Some("missing:local")).await;
        return Err("missing:local");
    }

    let starts_at = parse_scenario_datetime(&input.starts_at);
    let ends_at = if input.ends_at.trim().is_empty() {
        starts_at + chrono::Duration::hours(2)
    } else {
        parse_scenario_datetime(&input.ends_at)
    };

    let service = EventService::new(PostgresEventRepository::new(pool.clone()));
    let command = CreateEventCommand {
        academic_center_id: ca_uuid_from_id(ca_id),
        author_id: author_uuid_from_email(pool, actor_email).await,
        title: input.title.clone(),
        description: input.description.clone(),
        location: input.location.clone(),
        starts_at,
        ends_at,
        publish_directly: target_status == "publicado",
        registration: EventRegistrationInfo::unrestricted(),
    };

    let created_event = match service.create_event(command).await {
        Ok(event) => event,
        Err(error) => {
            let reason = map_application_error_reason(&error);
            record_event_operation_outcome(pool, "rejected", Some(reason)).await;
            return Err(reason);
        }
    };

    let event_id: i64 = sqlx::query_scalar(
        "INSERT INTO ca_event (
            ca_id, domain_event_id, author_email, title, description, starts_at, ends_at, location,
            status, requires_approval, published_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, FALSE, $10)
         RETURNING id",
    )
    .bind(ca_id)
    .bind(created_event.id)
    .bind(actor_email)
    .bind(&created_event.title)
    .bind(&created_event.description)
    .bind(&input.starts_at)
    .bind(&input.ends_at)
    .bind(&created_event.location)
    .bind(target_status)
    .bind(created_event.published_at)
    .fetch_one(pool)
    .await
    .expect("deve registrar espelho do evento no cenário");

    record_last_affected_event(pool, event_id).await;
    record_event_operation_outcome(pool, "accepted", None).await;
    Ok(event_id)
}

/// Valida e executa a publicação direta de um evento salvo em rascunho no CA via [`EventService`].
async fn attempt_direct_publication(
    pool: &PgPool,
    actor_email: &str,
    event_title: &str,
    ca_identifier: &str,
) -> Result<i64, &'static str> {
    let ca_id = find_ca_id(pool, ca_identifier).await;

    if !is_actor_authorized_for_events(pool, ca_id, actor_email).await {
        record_event_operation_outcome(pool, "rejected", Some("permission_denied")).await;
        return Err("permission_denied");
    }

    let domain_event_id: Option<Uuid> =
        sqlx::query_scalar("SELECT domain_event_id FROM ca_event WHERE ca_id = $1 AND title = $2")
            .bind(ca_id)
            .bind(event_title)
            .fetch_one(pool)
            .await
            .expect("deve localizar o evento em rascunho no CA");

    let published_at = if let Some(domain_id) = domain_event_id {
        let service = EventService::new(PostgresEventRepository::new(pool.clone()));
        let published = service
            .publish_event_directly(PublishEventCommand {
                event_id: domain_id,
                academic_center_id: Some(ca_uuid_from_id(ca_id)),
                actor_id: Some(author_uuid_from_email(pool, actor_email).await),
            })
            .await
            .expect("deve publicar diretamente no EventService");
        published.published_at
    } else {
        Some(Utc::now())
    };

    let event_id: i64 = sqlx::query_scalar(
        "UPDATE ca_event
         SET status = 'publicado',
             requires_approval = FALSE,
             published_at = COALESCE($3, NOW())
         WHERE ca_id = $1 AND title = $2
         RETURNING id",
    )
    .bind(ca_id)
    .bind(event_title)
    .bind(published_at)
    .fetch_one(pool)
    .await
    .expect("deve publicar diretamente o evento em rascunho no CA");

    record_last_affected_event(pool, event_id).await;
    record_event_operation_outcome(pool, "accepted", None).await;
    Ok(event_id)
}

/// Captura um retrato do evento no CA antes de uma tentativa de edição.
async fn snapshot_event_before_edit(pool: &PgPool, ca_id: i64, event_title: &str) {
    sqlx::query(
        "INSERT INTO bdd_event_snapshot (
            event_id, ca_id, title, description, starts_at, ends_at, location, status
         )
         SELECT id, ca_id, title, description, starts_at, ends_at, location, status
         FROM ca_event
         WHERE ca_id = $1 AND title = $2
         ON CONFLICT (event_id) DO UPDATE SET
            ca_id = EXCLUDED.ca_id,
            title = EXCLUDED.title,
            description = EXCLUDED.description,
            starts_at = EXCLUDED.starts_at,
            ends_at = EXCLUDED.ends_at,
            location = EXCLUDED.location,
            status = EXCLUDED.status",
    )
    .bind(ca_id)
    .bind(event_title)
    .execute(pool)
    .await
    .expect("deve capturar retrato do evento antes da edição");
}

/// Valida e executa a edição de um evento existente no CA via [`EventService`].
async fn attempt_event_edit(
    pool: &PgPool,
    actor_email: &str,
    ca_identifier: &str,
    original_title: &str,
    input: &EventFormInput,
) -> Result<i64, &'static str> {
    let ca_id = find_ca_id(pool, ca_identifier).await;
    snapshot_event_before_edit(pool, ca_id, original_title).await;

    if !is_actor_authorized_for_events(pool, ca_id, actor_email).await {
        record_event_operation_outcome(pool, "rejected", Some("permission_denied")).await;
        return Err("permission_denied");
    }

    if input.title.trim().is_empty() {
        record_event_operation_outcome(pool, "rejected", Some("missing:título")).await;
        return Err("missing:título");
    }
    if input.description.trim().is_empty() {
        record_event_operation_outcome(pool, "rejected", Some("missing:descrição")).await;
        return Err("missing:descrição");
    }
    if input.starts_at.trim().is_empty() {
        record_event_operation_outcome(pool, "rejected", Some("missing:início")).await;
        return Err("missing:início");
    }
    if input.location.trim().is_empty() {
        record_event_operation_outcome(pool, "rejected", Some("missing:local")).await;
        return Err("missing:local");
    }

    let starts_at = parse_scenario_datetime(&input.starts_at);
    let ends_at = if input.ends_at.trim().is_empty() {
        starts_at + chrono::Duration::hours(2)
    } else {
        parse_scenario_datetime(&input.ends_at)
    };

    let domain_event_id: Option<Uuid> =
        sqlx::query_scalar("SELECT domain_event_id FROM ca_event WHERE ca_id = $1 AND title = $2")
            .bind(ca_id)
            .bind(original_title)
            .fetch_one(pool)
            .await
            .expect("deve localizar o evento a ser editado no CA");

    if let Some(domain_id) = domain_event_id {
        let service = EventService::new(PostgresEventRepository::new(pool.clone()));
        let command = UpdateEventCommand {
            event_id: domain_id,
            academic_center_id: Some(ca_uuid_from_id(ca_id)),
            actor_id: Some(author_uuid_from_email(pool, actor_email).await),
            title: input.title.clone(),
            description: input.description.clone(),
            location: input.location.clone(),
            starts_at,
            ends_at,
            registration: None,
        };

        if let Err(error) = service.update_event(command).await {
            let reason = map_application_error_reason(&error);
            record_event_operation_outcome(pool, "rejected", Some(reason)).await;
            return Err(reason);
        }
    } else if ends_at <= starts_at {
        record_event_operation_outcome(pool, "rejected", Some("invalid_period")).await;
        return Err("invalid_period");
    }

    let event_id: i64 = sqlx::query_scalar(
        "UPDATE ca_event
         SET title = $3,
             description = $4,
             starts_at = $5,
             ends_at = $6,
             location = $7
         WHERE ca_id = $1 AND title = $2
         RETURNING id",
    )
    .bind(ca_id)
    .bind(original_title)
    .bind(&input.title)
    .bind(&input.description)
    .bind(&input.starts_at)
    .bind(&input.ends_at)
    .bind(&input.location)
    .fetch_one(pool)
    .await
    .expect("deve atualizar os dados do evento no CA");

    record_last_affected_event(pool, event_id).await;
    record_event_operation_outcome(pool, "accepted", None).await;
    Ok(event_id)
}

// ============================================================================
// Contexto (Pré-condições de CA, cargos, permissões, estudantes e eventos)
// ============================================================================

/// Cadastra o Centro Acadêmico informado no banco isolado do cenário.
#[given(expr = "que o Centro Acadêmico {string} está cadastrado na UnB")]
async fn academic_center_is_registered(world: &mut AppWorld, ca_name: String) {
    let pool = world.database_pool();
    ensure_events_schema(pool).await;

    let acronym = extract_ca_acronym(&ca_name);
    sqlx::query(
        "INSERT INTO academic_center (name, acronym)
         VALUES ($1, $2)
         ON CONFLICT (acronym) DO UPDATE SET name = EXCLUDED.name",
    )
    .bind(&ca_name)
    .bind(&acronym)
    .execute(pool)
    .await
    .expect("deve cadastrar o Centro Acadêmico no banco do cenário");
}

/// Registra os três cargos iniciais no Centro Acadêmico informado.
#[given(expr = "o CA {string} possui os cargos {string}, {string} e {string}")]
async fn academic_center_has_roles(
    world: &mut AppWorld,
    ca: String,
    first_role: String,
    second_role: String,
    third_role: String,
) {
    let pool = world.database_pool();
    ensure_events_schema(pool).await;
    let ca_id = find_ca_id(pool, &ca).await;

    for role in [&first_role, &second_role, &third_role] {
        ensure_role_in_ca(pool, ca_id, role).await;
    }
}

/// Associa a permissão informada ao cargo nos Centros Acadêmicos cadastrados.
#[given(expr = "o cargo {string} possui a permissão de {string}")]
async fn role_has_permission(world: &mut AppWorld, role: String, permission: String) {
    let pool = world.database_pool();
    ensure_events_schema(pool).await;

    sqlx::query(
        "INSERT INTO ca_role_permission (ca_role_id, permission)
         SELECT id, $2 FROM ca_role WHERE name = $1
         ON CONFLICT (ca_role_id, permission) DO NOTHING",
    )
    .bind(&role)
    .bind(&permission)
    .execute(pool)
    .await
    .expect("deve associar a permissão ao cargo");
}

/// Remove a permissão informada do cargo nos Centros Acadêmicos cadastrados.
#[given(expr = "o cargo {string} não possui a permissão de {string}")]
async fn role_does_not_have_permission(world: &mut AppWorld, role: String, permission: String) {
    let pool = world.database_pool();
    ensure_events_schema(pool).await;

    sqlx::query(
        "DELETE FROM ca_role_permission
         WHERE permission = $2
           AND ca_role_id IN (SELECT id FROM ca_role WHERE name = $1)",
    )
    .bind(&role)
    .bind(&permission)
    .execute(pool)
    .await
    .expect("deve remover a permissão do cargo");
}

/// Vincula o estudante ao cargo indicado no Centro Acadêmico.
#[given(expr = "o estudante {string} exerce o cargo de {string} em {string}")]
async fn student_holds_role_in_ca(
    world: &mut AppWorld,
    student_email: String,
    role: String,
    ca: String,
) {
    let pool = world.database_pool();
    ensure_events_schema(pool).await;

    let ca_id = find_ca_id(pool, &ca).await;
    let role_id = ensure_role_in_ca(pool, ca_id, &role).await;
    let user_id = ensure_user_profile(pool, &student_email).await;

    sqlx::query(
        "INSERT INTO ca_management_member (ca_id, user_id, ca_role_id)
         VALUES ($1, $2, $3)
         ON CONFLICT (ca_id, user_id) DO UPDATE SET ca_role_id = EXCLUDED.ca_role_id",
    )
    .bind(ca_id)
    .bind(user_id)
    .bind(role_id)
    .execute(pool)
    .await
    .expect("deve vincular o estudante ao cargo no CA");
}

/// Garante que o estudante possua perfil no CApp e não integre a gestão do CA informado.
#[given(expr = "o estudante {string} possui perfil no CApp e não integra a gestão de {string}")]
async fn student_has_profile_and_not_in_management(
    world: &mut AppWorld,
    student_email: String,
    ca: String,
) {
    let pool = world.database_pool();
    ensure_events_schema(pool).await;

    let ca_id = find_ca_id(pool, &ca).await;
    let user_id = ensure_user_profile(pool, &student_email).await;

    sqlx::query("DELETE FROM ca_management_member WHERE ca_id = $1 AND user_id = $2")
        .bind(ca_id)
        .bind(user_id)
        .execute(pool)
        .await
        .expect("deve garantir que o estudante não integre a gestão do CA");
}

/// Garante que o estudante não possua cargo de gestão no CA indicado.
#[given(expr = "o estudante {string} não possui cargo de gestão em {string}")]
async fn student_has_no_management_role_in_ca(
    world: &mut AppWorld,
    student_email: String,
    ca: String,
) {
    student_has_profile_and_not_in_management(world, student_email, ca).await;
}

/// Prepara um evento em rascunho no CA criado pelo autor informado.
#[given(expr = "que existe em {string} o evento em rascunho {string} criado por {string}")]
async fn draft_event_exists_in_ca(
    world: &mut AppWorld,
    ca: String,
    event_title: String,
    author_email: String,
) {
    let pool = world.database_pool();
    ensure_events_schema(pool).await;
    let ca_id = find_ca_id(pool, &ca).await;
    let service = EventService::new(PostgresEventRepository::new(pool.clone()));

    let created = service
        .create_event(CreateEventCommand {
            academic_center_id: ca_uuid_from_id(ca_id),
            author_id: author_uuid_from_email(pool, &author_email).await,
            title: event_title.clone(),
            description: "Descrição em rascunho".to_string(),
            location: "Laboratório de Redes - Bloco B".to_string(),
            starts_at: parse_scenario_datetime("2026-11-18 10:00"),
            ends_at: parse_scenario_datetime("2026-11-18 12:00"),
            publish_directly: false,
            registration: EventRegistrationInfo::unrestricted(),
        })
        .await
        .expect("deve criar evento em rascunho no EventService");

    let event_id: i64 = sqlx::query_scalar(
        "INSERT INTO ca_event (
            ca_id, domain_event_id, author_email, title, description, starts_at, ends_at, location,
            status, requires_approval, published_at
         )
         VALUES ($1, $2, $3, $4, 'Descrição em rascunho', '2026-11-18 10:00', '2026-11-18 12:00', 'Laboratório de Redes - Bloco B', 'rascunho', FALSE, NULL)
         RETURNING id",
    )
    .bind(ca_id)
    .bind(created.id)
    .bind(&author_email)
    .bind(&event_title)
    .fetch_one(pool)
    .await
    .expect("deve preparar evento em rascunho no CA");

    record_last_affected_event(pool, event_id).await;
}

/// Prepara um evento publicado no CA com os dados fornecidos na tabela Gherkin.
#[given(expr = "que existe em {string} o evento publicado {string} com os seguintes dados:")]
async fn published_event_exists_with_data(
    world: &mut AppWorld,
    ca: String,
    event_title: String,
    step: &Step,
) {
    let pool = world.database_pool();
    ensure_events_schema(pool).await;
    let ca_id = find_ca_id(pool, &ca).await;
    let input = EventFormInput::from_step(step);
    let service = EventService::new(PostgresEventRepository::new(pool.clone()));

    let created = service
        .create_event(CreateEventCommand {
            academic_center_id: ca_uuid_from_id(ca_id),
            author_id: author_uuid_from_email(pool, "231012345@aluno.unb.br").await,
            title: event_title.clone(),
            description: input.description.clone(),
            location: input.location.clone(),
            starts_at: parse_scenario_datetime(&input.starts_at),
            ends_at: parse_scenario_datetime(&input.ends_at),
            publish_directly: true,
            registration: EventRegistrationInfo::unrestricted(),
        })
        .await
        .expect("deve criar evento publicado inicial no EventService");

    let event_id: i64 = sqlx::query_scalar(
        "INSERT INTO ca_event (
            ca_id, domain_event_id, author_email, title, description, starts_at, ends_at, location,
            status, requires_approval, published_at
         )
         VALUES ($1, $2, '231012345@aluno.unb.br', $3, $4, $5, $6, $7, 'publicado', FALSE, NOW())
         RETURNING id",
    )
    .bind(ca_id)
    .bind(created.id)
    .bind(&event_title)
    .bind(&input.description)
    .bind(&input.starts_at)
    .bind(&input.ends_at)
    .bind(&input.location)
    .fetch_one(pool)
    .await
    .expect("deve preparar evento publicado com dados iniciais no CA");

    record_last_affected_event(pool, event_id).await;
}

/// Prepara um evento em rascunho no CA com os dados fornecidos na tabela Gherkin.
#[given(expr = "que existe em {string} o evento em rascunho {string} com os seguintes dados:")]
async fn draft_event_exists_with_data(
    world: &mut AppWorld,
    ca: String,
    event_title: String,
    step: &Step,
) {
    let pool = world.database_pool();
    ensure_events_schema(pool).await;
    let ca_id = find_ca_id(pool, &ca).await;
    let input = EventFormInput::from_step(step);
    let service = EventService::new(PostgresEventRepository::new(pool.clone()));

    let created = service
        .create_event(CreateEventCommand {
            academic_center_id: ca_uuid_from_id(ca_id),
            author_id: author_uuid_from_email(pool, "231012345@aluno.unb.br").await,
            title: event_title.clone(),
            description: input.description.clone(),
            location: input.location.clone(),
            starts_at: parse_scenario_datetime(&input.starts_at),
            ends_at: parse_scenario_datetime(&input.ends_at),
            publish_directly: false,
            registration: EventRegistrationInfo::unrestricted(),
        })
        .await
        .expect("deve criar evento em rascunho inicial no EventService");

    let event_id: i64 = sqlx::query_scalar(
        "INSERT INTO ca_event (
            ca_id, domain_event_id, author_email, title, description, starts_at, ends_at, location,
            status, requires_approval, published_at
         )
         VALUES ($1, $2, '231012345@aluno.unb.br', $3, $4, $5, $6, $7, 'rascunho', FALSE, NULL)
         RETURNING id",
    )
    .bind(ca_id)
    .bind(created.id)
    .bind(&event_title)
    .bind(&input.description)
    .bind(&input.starts_at)
    .bind(&input.ends_at)
    .bind(&input.location)
    .fetch_one(pool)
    .await
    .expect("deve preparar evento em rascunho com dados iniciais no CA");

    record_last_affected_event(pool, event_id).await;
}

/// Prepara um evento publicado preexistente no CA informado.
#[given(expr = "existe em {string} o evento publicado {string}")]
async fn published_event_exists_in_ca(world: &mut AppWorld, ca: String, event_title: String) {
    let pool = world.database_pool();
    ensure_events_schema(pool).await;
    let ca_id = find_ca_id(pool, &ca).await;
    let service = EventService::new(PostgresEventRepository::new(pool.clone()));

    let created = service
        .create_event(CreateEventCommand {
            academic_center_id: ca_uuid_from_id(ca_id),
            author_id: author_uuid_from_email(pool, "gestao@cacc.unb.br").await,
            title: event_title.clone(),
            description: "Competição oficial de programação da UnB".to_string(),
            location: "Laboratório de Informática - CIC".to_string(),
            starts_at: parse_scenario_datetime("2026-12-01 09:00"),
            ends_at: parse_scenario_datetime("2026-12-01 18:00"),
            publish_directly: true,
            registration: EventRegistrationInfo::unrestricted(),
        })
        .await
        .expect("deve criar evento publicado no outro CA via EventService");

    let event_id: i64 = sqlx::query_scalar(
        "INSERT INTO ca_event (
            ca_id, domain_event_id, author_email, title, description, starts_at, ends_at, location,
            status, requires_approval, published_at
         )
         VALUES (
            $1,
            $2,
            'gestao@cacc.unb.br',
            $3,
            'Competição oficial de programação da UnB',
            '2026-12-01 09:00',
            '2026-12-01 18:00',
            'Laboratório de Informática - CIC',
            'publicado',
            FALSE,
            NOW()
         )
         RETURNING id",
    )
    .bind(ca_id)
    .bind(created.id)
    .bind(&event_title)
    .fetch_one(pool)
    .await
    .expect("deve preparar evento publicado no outro CA");

    record_last_affected_event(pool, event_id).await;
}

// ============================================================================
// Ações (Criação, publicação direta e edição de eventos)
// ============================================================================

/// Executa a criação e publicação direta de um evento por um gestor autorizado.
#[when(
    expr = "o gestor {string} cria e publica diretamente em {string} um evento com os seguintes dados:"
)]
async fn manager_creates_and_publishes_event_directly(
    world: &mut AppWorld,
    manager_email: String,
    ca: String,
    step: &Step,
) {
    let pool = world.database_pool();
    let input = EventFormInput::from_step(step);
    attempt_event_creation(pool, &manager_email, &ca, &input, "publicado")
        .await
        .expect("o gestor autorizado deve criar e publicar diretamente o evento com sucesso");
}

/// Executa a criação de um evento como rascunho por um gestor autorizado.
#[when(expr = "o gestor {string} cria como rascunho em {string} um evento com os seguintes dados:")]
async fn manager_creates_draft_event(
    world: &mut AppWorld,
    manager_email: String,
    ca: String,
    step: &Step,
) {
    let pool = world.database_pool();
    let input = EventFormInput::from_step(step);
    attempt_event_creation(pool, &manager_email, &ca, &input, "rascunho")
        .await
        .expect("o gestor autorizado deve salvar o evento como rascunho com sucesso");
}

/// Executa a publicação direta de um evento salvo em rascunho por um gestor autorizado.
#[when(expr = "o gestor {string} publica diretamente o evento {string} em {string}")]
async fn manager_publishes_draft_event_directly(
    world: &mut AppWorld,
    manager_email: String,
    event_title: String,
    ca: String,
) {
    let pool = world.database_pool();
    attempt_direct_publication(pool, &manager_email, &event_title, &ca)
        .await
        .expect("o gestor autorizado deve publicar diretamente o rascunho com sucesso");
}

/// Executa uma tentativa de criar um evento sujeita às validações de campos obrigatórios e período.
#[when(expr = "o gestor {string} tenta criar em {string} um evento com os seguintes dados:")]
async fn manager_attempts_to_create_event(
    world: &mut AppWorld,
    manager_email: String,
    ca: String,
    step: &Step,
) {
    let pool = world.database_pool();
    let input = EventFormInput::from_step(step);
    let _ = attempt_event_creation(pool, &manager_email, &ca, &input, "publicado").await;
}

/// Executa uma tentativa de criar e publicar diretamente um evento por um usuário sem permissão.
#[when(
    expr = "o estudante {string} tenta criar e publicar diretamente em {string} um evento com os seguintes dados:"
)]
#[when(
    expr = "{string} tenta criar e publicar diretamente em {string} um evento com os seguintes dados:"
)]
async fn actor_attempts_to_create_and_publish_event_directly(
    world: &mut AppWorld,
    actor_email: String,
    ca: String,
    step: &Step,
) {
    let pool = world.database_pool();
    let input = EventFormInput::from_step(step);
    let _ = attempt_event_creation(pool, &actor_email, &ca, &input, "publicado").await;
}

/// Executa uma tentativa de publicar diretamente um rascunho por um usuário sem permissão.
#[when(expr = "{string} tenta publicar diretamente o evento {string} em {string}")]
async fn actor_attempts_to_publish_draft_directly(
    world: &mut AppWorld,
    actor_email: String,
    event_title: String,
    ca: String,
) {
    let pool = world.database_pool();
    let _ = attempt_direct_publication(pool, &actor_email, &event_title, &ca).await;
}

/// Executa a edição de informações de um evento existente por um gestor autorizado.
#[when(expr = "o gestor {string} edita o evento {string} em {string} para:")]
async fn manager_edits_event(
    world: &mut AppWorld,
    manager_email: String,
    event_title: String,
    ca: String,
    step: &Step,
) {
    let pool = world.database_pool();
    let input = EventFormInput::from_step(step);
    attempt_event_edit(pool, &manager_email, &ca, &event_title, &input)
        .await
        .expect("o gestor autorizado deve editar o evento com sucesso");
}

/// Executa uma tentativa de editar um evento sujeita às validações de permissão, campos e período.
#[when(expr = "o gestor {string} tenta editar o evento {string} em {string} para:")]
#[when(expr = "{string} tenta editar o evento {string} em {string} para:")]
async fn actor_attempts_to_edit_event_with_table(
    world: &mut AppWorld,
    actor_email: String,
    event_title: String,
    ca: String,
    step: &Step,
) {
    let pool = world.database_pool();
    let input = EventFormInput::from_step(step);
    let _ = attempt_event_edit(pool, &actor_email, &ca, &event_title, &input).await;
}

/// Executa uma tentativa de editar um evento pertencente a outro CA sem permissão naquela gestão.
#[when(expr = "o gestor {string} tenta editar o evento {string} em {string}")]
async fn manager_attempts_to_edit_event_in_other_ca(
    world: &mut AppWorld,
    manager_email: String,
    event_title: String,
    ca: String,
) {
    let pool = world.database_pool();
    let attempted_input = EventFormInput {
        title: format!("{event_title} (Alterado Indevidamente)"),
        description: "Tentativa de edição sem vínculo no outro CA".to_string(),
        starts_at: "2026-12-01 10:00".to_string(),
        ends_at: "2026-12-01 19:00".to_string(),
        location: "Outro Local".to_string(),
    };
    let _ = attempt_event_edit(pool, &manager_email, &ca, &event_title, &attempted_input).await;
}

// ============================================================================
// Verificações (Situação de publicação, agenda do CA, autoria e recusas)
// ============================================================================

/// Verifica que o evento indicado está com situação publicada no CA.
#[then(expr = "o evento {string} passa a estar publicado em {string}")]
async fn event_becomes_published_in_ca(world: &mut AppWorld, event_title: String, ca: String) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;

    let status: Option<String> =
        sqlx::query_scalar("SELECT status FROM ca_event WHERE ca_id = $1 AND title = $2")
            .bind(ca_id)
            .bind(&event_title)
            .fetch_optional(pool)
            .await
            .expect("deve consultar situação do evento no CA");

    assert_eq!(
        status.as_deref(),
        Some("publicado"),
        "o evento '{event_title}' deveria estar publicado em {ca}",
    );
}

/// Verifica que o último evento manipulado registra o gestor esperado como autor responsável no CA.
#[then(expr = "o evento registra {string} como autor responsável em {string}")]
async fn event_records_responsible_author_in_ca(
    world: &mut AppWorld,
    author_email: String,
    ca: String,
) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;
    let event_id = fetch_last_affected_event_id(pool).await;

    let recorded_author: Option<String> =
        sqlx::query_scalar("SELECT author_email FROM ca_event WHERE id = $1 AND ca_id = $2")
            .bind(event_id)
            .bind(ca_id)
            .fetch_optional(pool)
            .await
            .expect("deve consultar o autor responsável do evento no CA");

    assert_eq!(
        recorded_author.as_deref(),
        Some(author_email.as_str()),
        "o evento deveria registrar {author_email} como autor responsável em {ca}",
    );
}

/// Verifica que o evento está publicado na agenda do CA e não requer homologação.
#[then(
    expr = "o evento fica visível na agenda de eventos publicados de {string} sem exigir homologação"
)]
async fn event_is_visible_in_published_schedule_without_approval(world: &mut AppWorld, ca: String) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;
    let event_id = fetch_last_affected_event_id(pool).await;

    let (status, requires_approval): (String, bool) = sqlx::query_as(
        "SELECT status, requires_approval FROM ca_event WHERE id = $1 AND ca_id = $2",
    )
    .bind(event_id)
    .bind(ca_id)
    .fetch_one(pool)
    .await
    .expect("deve consultar visibilidade e exigência de homologação do evento");

    assert_eq!(status, "publicado");
    assert!(
        !requires_approval,
        "a publicação direta não deve exigir homologação",
    );
}

/// Verifica que o evento está registrado com situação de rascunho no CA.
#[then(expr = "o evento {string} fica registrado como rascunho em {string}")]
#[then(expr = "o evento {string} mantém a situação de rascunho em {string}")]
#[then(expr = "o evento {string} permanece como rascunho em {string}")]
async fn event_is_registered_as_draft_in_ca(world: &mut AppWorld, event_title: String, ca: String) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;

    let status: Option<String> =
        sqlx::query_scalar("SELECT status FROM ca_event WHERE ca_id = $1 AND title = $2")
            .bind(ca_id)
            .bind(&event_title)
            .fetch_optional(pool)
            .await
            .expect("deve consultar situação de rascunho do evento no CA");

    assert_eq!(
        status.as_deref(),
        Some("rascunho"),
        "o evento '{event_title}' deveria estar registrado como rascunho em {ca}",
    );
}

/// Verifica que o último evento manipulado não figura na agenda de eventos publicados do CA.
#[then(expr = "o evento não aparece na agenda de eventos publicados de {string}")]
#[then(expr = "permanece fora da agenda de eventos publicados de {string}")]
#[then(expr = "não aparece na agenda de eventos publicados de {string}")]
async fn event_does_not_appear_in_published_schedule(world: &mut AppWorld, ca: String) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;
    let event_id = fetch_last_affected_event_id(pool).await;

    let in_published_schedule: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM ca_event
            WHERE id = $1 AND ca_id = $2 AND status = 'publicado'
        )",
    )
    .bind(event_id)
    .bind(ca_id)
    .fetch_one(pool)
    .await
    .expect("deve verificar ausência do evento na agenda publicada do CA");

    assert!(
        !in_published_schedule,
        "o evento em rascunho não deveria aparecer na agenda publicada de {ca}",
    );
}

/// Verifica que o último evento manipulado passou a figurar na agenda de eventos publicados do CA.
#[then(expr = "o evento passa a figurar na agenda de eventos publicados de {string}")]
#[then(expr = "o evento permanece publicado na agenda de {string}")]
async fn event_appears_in_published_schedule(world: &mut AppWorld, ca: String) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;
    let event_id = fetch_last_affected_event_id(pool).await;

    let in_published_schedule: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM ca_event
            WHERE id = $1 AND ca_id = $2 AND status = 'publicado'
        )",
    )
    .bind(event_id)
    .bind(ca_id)
    .fetch_one(pool)
    .await
    .expect("deve verificar presença do evento na agenda publicada do CA");

    assert!(
        in_published_schedule,
        "o evento deveria figurar na agenda de eventos publicados de {ca}",
    );
}

/// Verifica que o evento possui data/hora de publicação registrada.
#[then("a data de publicação do evento fica registrada")]
async fn event_publication_timestamp_is_recorded(world: &mut AppWorld) {
    let pool = world.database_pool();
    let event_id = fetch_last_affected_event_id(pool).await;

    let has_published_at: bool = sqlx::query_scalar(
        "SELECT published_at IS NOT NULL AND published_at <= NOW()
         FROM ca_event
         WHERE id = $1",
    )
    .bind(event_id)
    .fetch_one(pool)
    .await
    .expect("deve verificar data de publicação do evento");

    assert!(
        has_published_at,
        "o evento publicado deve registrar a data/hora de publicação",
    );
}

/// Verifica que o evento indicado pelo título está visível na agenda publicada do CA.
#[then(expr = "o evento {string} fica visível na agenda de eventos publicados de {string}")]
async fn named_event_is_visible_in_ca_schedule(
    world: &mut AppWorld,
    event_title: String,
    ca: String,
) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;

    let is_visible: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM ca_event
            WHERE ca_id = $1 AND title = $2 AND status = 'publicado'
        )",
    )
    .bind(ca_id)
    .bind(&event_title)
    .fetch_one(pool)
    .await
    .expect("deve verificar visibilidade do evento na agenda do CA");

    assert!(
        is_visible,
        "o evento '{event_title}' deveria estar visível na agenda publicada de {ca}",
    );
}

/// Verifica que a agenda publicada do CA não contém o evento indicado.
#[then(expr = "a agenda de eventos publicados de {string} permanece sem o evento {string}")]
async fn ca_published_schedule_does_not_contain_event(
    world: &mut AppWorld,
    ca: String,
    event_title: String,
) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;

    let exists_in_ca: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM ca_event
            WHERE ca_id = $1 AND title = $2 AND status = 'publicado'
        )",
    )
    .bind(ca_id)
    .bind(&event_title)
    .fetch_one(pool)
    .await
    .expect("deve verificar agenda de eventos publicados do outro CA");

    assert!(
        !exists_in_ca,
        "a agenda publicada de {ca} não deveria conter o evento '{event_title}'",
    );
}

/// Verifica que a operação foi recusada devido à ausência de um campo obrigatório específico.
#[then(expr = "a operação é recusada informando que {string} do evento é obrigatório")]
async fn operation_is_rejected_missing_required_field(
    world: &mut AppWorld,
    required_field: String,
) {
    let (status, reason) = fetch_last_event_operation_outcome(world.database_pool()).await;
    let expected_reason = format!("missing:{required_field}");
    assert_eq!(status, "rejected");
    assert_eq!(reason.as_deref(), Some(expected_reason.as_str()));
}

/// Verifica que a operação foi recusada porque o término informado não é posterior ao início.
#[then("a operação é recusada informando que o horário de término deve ser posterior ao início")]
async fn operation_is_rejected_invalid_period(world: &mut AppWorld) {
    let (status, reason) = fetch_last_event_operation_outcome(world.database_pool()).await;
    assert_eq!(status, "rejected");
    assert_eq!(reason.as_deref(), Some("invalid_period"));
}

/// Verifica que nenhum evento foi criado no CA após a tentativa recusada.
#[then(expr = "nenhum novo evento é registrado em {string}")]
async fn no_new_event_is_registered_in_ca(world: &mut AppWorld, ca: String) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ca_event WHERE ca_id = $1")
        .bind(ca_id)
        .fetch_one(pool)
        .await
        .expect("deve contar eventos registrados no CA");

    assert_eq!(
        count, 0,
        "nenhum novo evento deveria ter sido registrado em {ca}",
    );
}

/// Representa os campos persistidos de um evento para verificação de atualização ou inalterabilidade.
#[derive(Debug, PartialEq, Eq, sqlx::FromRow)]
struct EventDetailsRow {
    title: String,
    description: String,
    starts_at: String,
    ends_at: String,
    location: String,
    status: String,
}

/// Verifica que o evento editado apresenta no CA todos os novos dados informados na tabela.
#[then(expr = "o evento passa a apresentar em {string} os dados atualizados:")]
async fn event_presents_updated_data_in_ca(world: &mut AppWorld, ca: String, step: &Step) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;
    let event_id = fetch_last_affected_event_id(pool).await;
    let expected = EventFormInput::from_step(step);

    let current: EventDetailsRow = sqlx::query_as(
        "SELECT title, description, starts_at, ends_at, location, status
         FROM ca_event
         WHERE id = $1 AND ca_id = $2",
    )
    .bind(event_id)
    .bind(ca_id)
    .fetch_one(pool)
    .await
    .expect("deve consultar os dados atualizados do evento no CA");

    assert_eq!(current.title, expected.title);
    assert_eq!(current.description, expected.description);
    assert_eq!(current.starts_at, expected.starts_at);
    assert_eq!(current.ends_at, expected.ends_at);
    assert_eq!(current.location, expected.location);
}

/// Verifica que o evento permaneceu exatamente com os mesmos dados anteriores após uma recusa.
#[then(expr = "o evento {string} permanece inalterado em {string}")]
async fn event_remains_unchanged_in_ca(world: &mut AppWorld, event_title: String, ca: String) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;

    let snapshot: EventDetailsRow = sqlx::query_as(
        "SELECT title, description, starts_at, ends_at, location, status
         FROM bdd_event_snapshot
         WHERE ca_id = $1 AND title = $2",
    )
    .bind(ca_id)
    .bind(&event_title)
    .fetch_one(pool)
    .await
    .expect("deve existir retrato prévio do evento");

    let current: EventDetailsRow = sqlx::query_as(
        "SELECT title, description, starts_at, ends_at, location, status
         FROM ca_event
         WHERE ca_id = $1 AND title = $2",
    )
    .bind(ca_id)
    .bind(&event_title)
    .fetch_one(pool)
    .await
    .expect("deve consultar o evento atual no CA");

    assert_eq!(
        current, snapshot,
        "o evento '{event_title}' em {ca} deveria permanecer inalterado",
    );
}

/// Verifica que a operação foi recusada devido à ausência da permissão exigida no CA.
#[then("a operação é recusada por falta de permissão")]
async fn operation_is_rejected_for_lack_of_permission(world: &mut AppWorld) {
    let (status, reason) = fetch_last_event_operation_outcome(world.database_pool()).await;
    assert_eq!(status, "rejected");
    assert_eq!(reason.as_deref(), Some("permission_denied"));
}

/// Verifica que o evento indicado pelo título não foi publicado na agenda do CA.
#[then(expr = "o evento {string} não é publicado na agenda de {string}")]
async fn event_is_not_published_in_ca_schedule(
    world: &mut AppWorld,
    event_title: String,
    ca: String,
) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;

    let is_published: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM ca_event
            WHERE ca_id = $1 AND title = $2 AND status = 'publicado'
        )",
    )
    .bind(ca_id)
    .bind(&event_title)
    .fetch_one(pool)
    .await
    .expect("deve verificar ausência de publicação do evento na agenda do CA");

    assert!(
        !is_published,
        "o evento '{event_title}' não deveria estar publicado na agenda de {ca}",
    );
}
