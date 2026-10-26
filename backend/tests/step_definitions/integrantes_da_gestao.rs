//! Steps da especificação de administração dos integrantes da gestão (#62).
//!
//! Implementa o `Contexto` e as duas primeiras regras de
//! `features/gestao_ca/integrantes_da_gestao.feature` sobre o banco PostgreSQL
//! isolado fornecido por [`AppWorld`]. Os cenários das regras seguintes
//! permanecem sinalizados com [`pending_step`] até sua ativação.

use cucumber::{given, then, when};
use sqlx::PgPool;

use super::pending_step;
use crate::support::AppWorld;

/// Permissão exigida para administrar integrantes da gestão de um CA.
const MANAGE_MEMBERS_PERMISSION: &str = "Administrar integrantes da gestão";

/// Garante a estrutura de tabelas necessária aos cenários no banco isolado.
async fn ensure_management_schema(pool: &PgPool) {
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
        "CREATE TABLE IF NOT EXISTS ca_management_history (
            id BIGSERIAL PRIMARY KEY,
            ca_id BIGINT NOT NULL REFERENCES academic_center(id) ON DELETE CASCADE,
            member_email TEXT NOT NULL,
            operation TEXT NOT NULL,
            previous_role TEXT,
            resulting_role TEXT,
            manager_email TEXT NOT NULL,
            occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await
    .expect("deve criar tabela ca_management_history no banco do cenário");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS bdd_last_operation_outcome (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            status TEXT NOT NULL,
            reason TEXT
        )",
    )
    .execute(pool)
    .await
    .expect("deve criar tabela bdd_last_operation_outcome no banco do cenário");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS bdd_composition_snapshot (
            ca_id BIGINT NOT NULL,
            user_id BIGINT NOT NULL,
            ca_role_id BIGINT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .expect("deve criar tabela bdd_composition_snapshot no banco do cenário");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS bdd_history_snapshot (
            ca_id BIGINT PRIMARY KEY,
            history_count BIGINT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .expect("deve criar tabela bdd_history_snapshot no banco do cenário");
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
    if let Some(existing_id) =
        sqlx::query_scalar::<_, i64>("SELECT id FROM ca_role WHERE ca_id = $1 AND name = $2")
            .bind(ca_id)
            .bind(role_name)
            .fetch_optional(pool)
            .await
            .expect("deve consultar cargo existente no CA")
    {
        return existing_id;
    }

    let role_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO ca_role (ca_id, name)
         VALUES ($1, $2)
         RETURNING id",
    )
    .bind(ca_id)
    .bind(role_name)
    .fetch_one(pool)
    .await
    .expect("deve registrar o cargo no CA");

    let default_permission = match role_name {
        "Presidente" => MANAGE_MEMBERS_PERMISSION,
        "Diretor de Comunicação" => "Publicar notícias diretamente",
        "Diretor de Eventos" => "Publicar e gerenciar eventos diretamente",
        other => other,
    };

    sqlx::query(
        "INSERT INTO ca_role_permission (ca_role_id, permission)
         VALUES ($1, $2)
         ON CONFLICT (ca_role_id, permission) DO NOTHING",
    )
    .bind(role_id)
    .bind(default_permission)
    .execute(pool)
    .await
    .expect("deve associar permissão padrão ao cargo");

    role_id
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

/// Registra o resultado da última operação administrativa executada no cenário.
async fn record_operation_outcome(pool: &PgPool, status: &str, reason: Option<&str>) {
    sqlx::query(
        "INSERT INTO bdd_last_operation_outcome (id, status, reason)
         VALUES (1, $1, $2)
         ON CONFLICT (id) DO UPDATE SET status = EXCLUDED.status, reason = EXCLUDED.reason",
    )
    .bind(status)
    .bind(reason)
    .execute(pool)
    .await
    .expect("deve registrar o resultado da operação");
}

/// Consulta o resultado da última operação administrativa do cenário.
async fn fetch_last_operation_outcome(pool: &PgPool) -> (String, Option<String>) {
    sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT status, reason FROM bdd_last_operation_outcome WHERE id = 1",
    )
    .fetch_one(pool)
    .await
    .expect("uma operação administrativa deve ter sido executada antes da verificação")
}

/// Salva um retrato da composição e do histórico atual da gestão do CA antes de uma tentativa.
async fn snapshot_ca_state(pool: &PgPool, ca_id: i64) {
    sqlx::query("DELETE FROM bdd_composition_snapshot WHERE ca_id = $1")
        .bind(ca_id)
        .execute(pool)
        .await
        .expect("deve limpar retrato anterior da composição");

    sqlx::query(
        "INSERT INTO bdd_composition_snapshot (ca_id, user_id, ca_role_id)
         SELECT ca_id, user_id, ca_role_id
         FROM ca_management_member
         WHERE ca_id = $1",
    )
    .bind(ca_id)
    .execute(pool)
    .await
    .expect("deve capturar retrato da composição da gestão");

    sqlx::query(
        "INSERT INTO bdd_history_snapshot (ca_id, history_count)
         VALUES ($1, (SELECT COUNT(*) FROM ca_management_history WHERE ca_id = $1))
         ON CONFLICT (ca_id) DO UPDATE SET history_count = EXCLUDED.history_count",
    )
    .bind(ca_id)
    .execute(pool)
    .await
    .expect("deve capturar retrato do histórico administrativo");
}

/// Verifica se o ator possui a permissão de administrar integrantes no CA.
async fn is_actor_authorized_in_ca(pool: &PgPool, ca_id: i64, actor_email: &str) -> bool {
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
    .bind(MANAGE_MEMBERS_PERMISSION)
    .fetch_one(pool)
    .await
    .expect("deve verificar permissão do ator no CA")
}

/// Valida e executa (ou recusa) a tentativa de vincular um usuário a um cargo no CA.
async fn attempt_member_assignment(
    pool: &PgPool,
    actor_email: &str,
    student_email: &str,
    role_name: Option<&str>,
    ca_identifier: &str,
) -> Result<(), &'static str> {
    let ca_id = find_ca_id(pool, ca_identifier).await;
    snapshot_ca_state(pool, ca_id).await;

    if !is_actor_authorized_in_ca(pool, ca_id, actor_email).await {
        record_operation_outcome(pool, "rejected", Some("permission_denied")).await;
        return Err("permission_denied");
    }

    let Some(user_id) =
        sqlx::query_scalar::<_, i64>("SELECT id FROM user_profile WHERE email = $1")
            .bind(student_email)
            .fetch_optional(pool)
            .await
            .expect("deve consultar perfil do usuário")
    else {
        record_operation_outcome(pool, "rejected", Some("user_not_found")).await;
        return Err("user_not_found");
    };

    let Some(target_role_name) = role_name else {
        record_operation_outcome(pool, "rejected", Some("role_not_found")).await;
        return Err("role_not_found");
    };

    let Some(role_id) =
        sqlx::query_scalar::<_, i64>("SELECT id FROM ca_role WHERE ca_id = $1 AND name = $2")
            .bind(ca_id)
            .bind(target_role_name)
            .fetch_optional(pool)
            .await
            .expect("deve consultar cargo no CA")
    else {
        record_operation_outcome(pool, "rejected", Some("role_not_found")).await;
        return Err("role_not_found");
    };

    let already_member: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM ca_management_member WHERE ca_id = $1 AND user_id = $2
        )",
    )
    .bind(ca_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .expect("deve verificar se o usuário já integra a gestão");

    if already_member {
        record_operation_outcome(pool, "rejected", Some("already_in_management")).await;
        return Err("already_in_management");
    }

    sqlx::query(
        "INSERT INTO ca_management_member (ca_id, user_id, ca_role_id)
         VALUES ($1, $2, $3)",
    )
    .bind(ca_id)
    .bind(user_id)
    .bind(role_id)
    .execute(pool)
    .await
    .expect("deve vincular o estudante ao cargo da gestão");

    sqlx::query(
        "INSERT INTO ca_management_history (ca_id, member_email, operation, previous_role, resulting_role, manager_email)
         VALUES ($1, $2, 'vinculação', NULL, $3, $4)",
    )
    .bind(ca_id)
    .bind(student_email)
    .bind(target_role_name)
    .bind(actor_email)
    .execute(pool)
    .await
    .expect("deve registrar a vinculação no histórico administrativo");

    record_operation_outcome(pool, "accepted", None).await;
    Ok(())
}

/// Valida e executa (ou recusa) a tentativa de alterar o cargo de um integrante no CA.
async fn attempt_member_role_change(
    pool: &PgPool,
    actor_email: &str,
    student_email: &str,
    new_role_name: &str,
    ca_identifier: &str,
) -> Result<(), &'static str> {
    let ca_id = find_ca_id(pool, ca_identifier).await;
    snapshot_ca_state(pool, ca_id).await;

    if !is_actor_authorized_in_ca(pool, ca_id, actor_email).await {
        record_operation_outcome(pool, "rejected", Some("permission_denied")).await;
        return Err("permission_denied");
    }

    let current_membership: Option<(i64, String)> = sqlx::query_as(
        "SELECT m.user_id, r.name
         FROM ca_management_member m
         JOIN user_profile u ON u.id = m.user_id
         JOIN ca_role r ON r.id = m.ca_role_id
         WHERE m.ca_id = $1 AND u.email = $2",
    )
    .bind(ca_id)
    .bind(student_email)
    .fetch_optional(pool)
    .await
    .expect("deve consultar vínculo atual do integrante no CA");

    let Some((user_id, current_role_name)) = current_membership else {
        record_operation_outcome(pool, "rejected", Some("not_in_management")).await;
        return Err("not_in_management");
    };

    let Some(new_role_id) =
        sqlx::query_scalar::<_, i64>("SELECT id FROM ca_role WHERE ca_id = $1 AND name = $2")
            .bind(ca_id)
            .bind(new_role_name)
            .fetch_optional(pool)
            .await
            .expect("deve consultar novo cargo no CA")
    else {
        record_operation_outcome(pool, "rejected", Some("role_not_found")).await;
        return Err("role_not_found");
    };

    if current_role_name == new_role_name {
        record_operation_outcome(pool, "rejected", Some("already_holds_role")).await;
        return Err("already_holds_role");
    }

    sqlx::query(
        "UPDATE ca_management_member
         SET ca_role_id = $3
         WHERE ca_id = $1 AND user_id = $2",
    )
    .bind(ca_id)
    .bind(user_id)
    .bind(new_role_id)
    .execute(pool)
    .await
    .expect("deve atualizar o cargo do integrante na gestão");

    sqlx::query(
        "INSERT INTO ca_management_history (ca_id, member_email, operation, previous_role, resulting_role, manager_email)
         VALUES ($1, $2, 'alteração', $3, $4, $5)",
    )
    .bind(ca_id)
    .bind(student_email)
    .bind(&current_role_name)
    .bind(new_role_name)
    .bind(actor_email)
    .execute(pool)
    .await
    .expect("deve registrar a alteração de cargo no histórico administrativo");

    record_operation_outcome(pool, "accepted", None).await;
    Ok(())
}

// ============================================================================
// Contexto (Pré-condições de CA, cargos, permissões e integrantes)
// ============================================================================

/// Cadastra o Centro Acadêmico informado no banco isolado do cenário.
#[given(expr = "que o Centro Acadêmico {string} está cadastrado na UnB")]
async fn academic_center_is_registered(world: &mut AppWorld, ca_name: String) {
    let pool = world.database_pool();
    ensure_management_schema(pool).await;

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
    ensure_management_schema(pool).await;
    let ca_id = find_ca_id(pool, &ca).await;

    for role in [&first_role, &second_role, &third_role] {
        ensure_role_in_ca(pool, ca_id, role).await;
    }
}

/// Garante que o Centro Acadêmico não possua o cargo indicado.
#[given(expr = "o CA {string} não possui o cargo {string}")]
async fn academic_center_does_not_have_role(world: &mut AppWorld, ca: String, role: String) {
    let pool = world.database_pool();
    ensure_management_schema(pool).await;
    let ca_id = find_ca_id(pool, &ca).await;

    sqlx::query("DELETE FROM ca_role WHERE ca_id = $1 AND name = $2")
        .bind(ca_id)
        .bind(&role)
        .execute(pool)
        .await
        .expect("deve garantir que o cargo não exista no CA");
}

/// Associa a permissão informada ao cargo nos Centros Acadêmicos cadastrados.
#[given(expr = "que o cargo {string} possui a permissão de {string}")]
#[given(expr = "o cargo {string} possui a permissão de {string}")]
async fn role_has_permission(world: &mut AppWorld, role: String, permission: String) {
    let pool = world.database_pool();
    ensure_management_schema(pool).await;

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
    ensure_management_schema(pool).await;

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

/// Garante que o estudante possua perfil cadastrado no CApp.
#[given(expr = "que o estudante {string} possui perfil no CApp")]
#[given(expr = "o estudante {string} possui perfil no CApp")]
async fn student_has_profile(world: &mut AppWorld, student_email: String) {
    let pool = world.database_pool();
    ensure_management_schema(pool).await;
    ensure_user_profile(pool, &student_email).await;
}

/// Garante que não exista perfil cadastrado no CApp para o e-mail informado.
#[given(expr = "que não existe perfil no CApp para o e-mail {string}")]
async fn profile_does_not_exist(world: &mut AppWorld, email: String) {
    let pool = world.database_pool();
    ensure_management_schema(pool).await;

    sqlx::query("DELETE FROM user_profile WHERE email = $1")
        .bind(&email)
        .execute(pool)
        .await
        .expect("deve garantir ausência de perfil para o e-mail");
}

/// Vincula previamente o estudante ao cargo indicado no Centro Acadêmico.
#[given(expr = "que o estudante {string} exerce o cargo de {string} em {string}")]
#[given(expr = "o estudante {string} exerce o cargo de {string} em {string}")]
async fn student_holds_role_in_ca(
    world: &mut AppWorld,
    student_email: String,
    role: String,
    ca: String,
) {
    let pool = world.database_pool();
    ensure_management_schema(pool).await;

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

/// Confirma que o último estudante preparado no cenário não integra a gestão do CA.
#[given(expr = "não integra a gestão de {string}")]
async fn student_is_not_in_management(world: &mut AppWorld, ca: String) {
    let pool = world.database_pool();
    ensure_management_schema(pool).await;

    let ca_id = find_ca_id(pool, &ca).await;
    let latest_user_id: i64 =
        sqlx::query_scalar("SELECT id FROM user_profile ORDER BY id DESC LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("deve existir um perfil de estudante recém-preparado");

    sqlx::query("DELETE FROM ca_management_member WHERE ca_id = $1 AND user_id = $2")
        .bind(ca_id)
        .bind(latest_user_id)
        .execute(pool)
        .await
        .expect("deve garantir ausência de vínculo prévio na gestão");
}

/// Garante que o estudante informado não possua vínculo de gestão no CA indicado.
#[given(expr = "o estudante {string} não possui cargo de gestão em {string}")]
async fn student_has_no_management_role_in_ca(
    world: &mut AppWorld,
    student_email: String,
    ca: String,
) {
    let pool = world.database_pool();
    ensure_management_schema(pool).await;

    let ca_id = find_ca_id(pool, &ca).await;
    let user_id = ensure_user_profile(pool, &student_email).await;

    sqlx::query("DELETE FROM ca_management_member WHERE ca_id = $1 AND user_id = $2")
        .bind(ca_id)
        .bind(user_id)
        .execute(pool)
        .await
        .expect("deve garantir que o estudante não tenha cargo de gestão no CA");
}

/// Pendente: que {string} é o único integrante de {string} com a permissão de {string}.
#[given(expr = "que {string} é o único integrante de {string} com a permissão de {string}")]
async fn student_is_sole_member_with_permission(
    _world: &mut AppWorld,
    _student_email: String,
    _ca: String,
    _permission: String,
) {
    pending_step("que {string} é o único integrante de {string} com a permissão de {string}");
}

// ============================================================================
// Ações (Vinculação, alteração e remoção de integrantes)
// ============================================================================

/// Executa a vinculação de um estudante a um cargo da gestão por um gestor autorizado.
#[when(expr = "o gestor {string} vincula {string} ao cargo {string} em {string}")]
async fn manager_assigns_student_to_role(
    world: &mut AppWorld,
    manager_email: String,
    student_email: String,
    role: String,
    ca: String,
) {
    let pool = world.database_pool();
    attempt_member_assignment(pool, &manager_email, &student_email, Some(&role), &ca)
        .await
        .expect("a vinculação por um gestor autorizado deve ser concluída com sucesso");
}

/// Executa uma tentativa de vinculação sujeita às validações de permissão e domínio.
#[when(expr = "o gestor {string} tenta vincular {string} ao cargo {string} em {string}")]
#[when(expr = "{string} tenta vincular {string} ao cargo {string} em {string}")]
async fn actor_attempts_to_assign_student_to_role(
    world: &mut AppWorld,
    actor_email: String,
    student_email: String,
    role: String,
    ca: String,
) {
    let pool = world.database_pool();
    let _ = attempt_member_assignment(pool, &actor_email, &student_email, Some(&role), &ca).await;
}

/// Executa a tentativa de um gestor vincular um usuário a um cargo em outro CA.
#[when(expr = "o gestor {string} tenta vincular {string} a um cargo em {string}")]
async fn manager_attempts_to_assign_student_in_other_ca(
    world: &mut AppWorld,
    manager_email: String,
    student_email: String,
    ca: String,
) {
    let pool = world.database_pool();
    let _ = attempt_member_assignment(pool, &manager_email, &student_email, None, &ca).await;
}

/// Executa a alteração do cargo de um integrante da gestão por um gestor autorizado.
#[when(expr = "o gestor {string} altera o cargo de {string} para {string} em {string}")]
async fn manager_changes_member_role(
    world: &mut AppWorld,
    manager_email: String,
    student_email: String,
    new_role: String,
    ca: String,
) {
    let pool = world.database_pool();
    attempt_member_role_change(pool, &manager_email, &student_email, &new_role, &ca)
        .await
        .expect("a alteração de cargo por um gestor autorizado deve ser concluída com sucesso");
}

/// Executa uma tentativa de alteração de cargo sujeita às validações de permissão e domínio.
#[when(expr = "o gestor {string} tenta alterar o cargo de {string} para {string} em {string}")]
#[when(expr = "{string} tenta alterar o cargo de {string} para {string} em {string}")]
async fn actor_attempts_to_change_member_role(
    world: &mut AppWorld,
    actor_email: String,
    student_email: String,
    new_role: String,
    ca: String,
) {
    let pool = world.database_pool();
    let _ = attempt_member_role_change(pool, &actor_email, &student_email, &new_role, &ca).await;
}

/// Pendente: o gestor {string} remove {string} da gestão de {string}.
#[when(expr = "o gestor {string} remove {string} da gestão de {string}")]
async fn manager_removes_member_from_management(
    _world: &mut AppWorld,
    _manager_email: String,
    _student_email: String,
    _ca: String,
) {
    pending_step("o gestor {string} remove {string} da gestão de {string}");
}

/// Pendente: tentativa de remover um integrante da gestão de um CA.
#[when(expr = "o gestor {string} tenta remover {string} da gestão de {string}")]
#[when(expr = "{string} tenta remover {string} da gestão de {string}")]
async fn actor_attempts_to_remove_member_from_management(
    _world: &mut AppWorld,
    _actor_email: String,
    _student_email: String,
    _ca: String,
) {
    pending_step("o gestor {string} tenta remover {string} da gestão de {string}");
}

// ============================================================================
// Verificações (Composição da gestão, cargos, permissões e recusas)
// ============================================================================

/// Verifica que o estudante integra a gestão do CA exercendo o cargo esperado.
#[then(expr = "{string} passa a integrar a gestão de {string} no cargo {string}")]
async fn student_joins_management_in_role(
    world: &mut AppWorld,
    student_email: String,
    ca: String,
    role: String,
) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;

    let assigned_role: Option<String> = sqlx::query_scalar(
        "SELECT r.name
         FROM ca_management_member m
         JOIN user_profile u ON u.id = m.user_id
         JOIN ca_role r ON r.id = m.ca_role_id
         WHERE m.ca_id = $1 AND u.email = $2",
    )
    .bind(ca_id)
    .bind(&student_email)
    .fetch_optional(pool)
    .await
    .expect("deve consultar vínculo do integrante na gestão");

    assert_eq!(
        assigned_role.as_deref(),
        Some(role.as_str()),
        "o estudante {student_email} deveria exercer o cargo {role} na gestão de {ca}",
    );
}

/// Verifica que o integrante passou a exercer o novo cargo no CA.
#[then(expr = "{string} passa a exercer o cargo {string} em {string}")]
async fn student_starts_holding_role(
    world: &mut AppWorld,
    student_email: String,
    role: String,
    ca: String,
) {
    student_joins_management_in_role(world, student_email, ca, role).await;
}

/// Verifica que o estudante permanece exercendo unicamente o cargo esperado no CA.
#[then(expr = "{string} permanece somente no cargo {string} em {string}")]
#[then(expr = "{string} permanece no cargo {string} em {string}")]
async fn student_remains_in_role(
    world: &mut AppWorld,
    student_email: String,
    role: String,
    ca: String,
) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;

    let roles: Vec<String> = sqlx::query_scalar(
        "SELECT r.name
         FROM ca_management_member m
         JOIN user_profile u ON u.id = m.user_id
         JOIN ca_role r ON r.id = m.ca_role_id
         WHERE m.ca_id = $1 AND u.email = $2",
    )
    .bind(ca_id)
    .bind(&student_email)
    .fetch_all(pool)
    .await
    .expect("deve consultar cargos do integrante no CA");

    assert_eq!(
        roles,
        vec![role],
        "o estudante {student_email} deveria permanecer somente no cargo esperado em {ca}",
    );
}

/// Verifica que o estudante não integra a gestão do CA indicado.
#[then(expr = "{string} não passa a integrar a gestão de {string}")]
#[then(expr = "{string} permanece sem integrar a gestão de {string}")]
async fn student_remains_outside_management(
    world: &mut AppWorld,
    student_email: String,
    ca: String,
) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;

    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1
            FROM ca_management_member m
            JOIN user_profile u ON u.id = m.user_id
            WHERE m.ca_id = $1 AND u.email = $2
        )",
    )
    .bind(ca_id)
    .bind(&student_email)
    .fetch_one(pool)
    .await
    .expect("deve consultar presença do estudante na gestão do CA");

    assert!(
        !is_member,
        "o estudante {student_email} não deveria integrar a gestão de {ca}",
    );
}

/// Pendente: {string} deixa de integrar a gestão de {string}.
#[then(expr = "{string} deixa de integrar a gestão de {string}")]
async fn student_leaves_management(_world: &mut AppWorld, _student_email: String, _ca: String) {
    pending_step("{string} deixa de integrar a gestão de {string}");
}

/// Verifica que a composição da gestão do CA não sofreu alterações após a tentativa.
#[then(expr = "a composição da gestão de {string} permanece inalterada")]
async fn management_composition_remains_unchanged(world: &mut AppWorld, ca: String) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;

    let snapshot: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT user_id, ca_role_id
         FROM bdd_composition_snapshot
         WHERE ca_id = $1
         ORDER BY user_id, ca_role_id",
    )
    .bind(ca_id)
    .fetch_all(pool)
    .await
    .expect("deve ler o retrato anterior da composição da gestão");

    let current: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT user_id, ca_role_id
         FROM ca_management_member
         WHERE ca_id = $1
         ORDER BY user_id, ca_role_id",
    )
    .bind(ca_id)
    .fetch_all(pool)
    .await
    .expect("deve ler a composição atual da gestão");

    assert_eq!(
        current, snapshot,
        "a composição da gestão de {ca} deveria permanecer inalterada",
    );
}

/// Verifica que o estudante possui todas as permissões associadas ao cargo no CA.
#[then(expr = "{string} passa a possuir as permissões do cargo {string} em {string}")]
async fn student_receives_role_permissions(
    world: &mut AppWorld,
    student_email: String,
    role: String,
    ca: String,
) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;

    let role_permissions: Vec<String> = sqlx::query_scalar(
        "SELECT p.permission
         FROM ca_role_permission p
         JOIN ca_role r ON r.id = p.ca_role_id
         WHERE r.ca_id = $1 AND r.name = $2
         ORDER BY p.permission",
    )
    .bind(ca_id)
    .bind(&role)
    .fetch_all(pool)
    .await
    .expect("deve consultar as permissões do cargo");

    assert!(
        !role_permissions.is_empty(),
        "o cargo {role} em {ca} deve possuir permissões configuradas",
    );

    let student_permissions: Vec<String> = sqlx::query_scalar(
        "SELECT p.permission
         FROM ca_management_member m
         JOIN user_profile u ON u.id = m.user_id
         JOIN ca_role_permission p ON p.ca_role_id = m.ca_role_id
         WHERE m.ca_id = $1 AND u.email = $2
         ORDER BY p.permission",
    )
    .bind(ca_id)
    .bind(&student_email)
    .fetch_all(pool)
    .await
    .expect("deve consultar as permissões efetivas do estudante no CA");

    assert_eq!(
        student_permissions, role_permissions,
        "as permissões de {student_email} em {ca} devem corresponder às do cargo {role}",
    );
}

/// Pendente: {string} deixa de possuir as permissões do cargo {string} em {string}.
#[then(expr = "{string} deixa de possuir as permissões do cargo {string} em {string}")]
async fn student_loses_role_permissions(
    _world: &mut AppWorld,
    _student_email: String,
    _role: String,
    _ca: String,
) {
    pending_step("{string} deixa de possuir as permissões do cargo {string} em {string}");
}

/// Verifica que o estudante possui a permissão específica indicada no CA.
#[then(expr = "{string} passa a possuir a permissão de {string} em {string}")]
#[then(expr = "{string} permanece com a permissão de {string} em {string}")]
async fn student_has_specific_permission_in_ca(
    world: &mut AppWorld,
    student_email: String,
    permission: String,
    ca: String,
) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;

    let has_permission: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1
            FROM ca_management_member m
            JOIN user_profile u ON u.id = m.user_id
            JOIN ca_role_permission p ON p.ca_role_id = m.ca_role_id
            WHERE m.ca_id = $1 AND u.email = $2 AND p.permission = $3
        )",
    )
    .bind(ca_id)
    .bind(&student_email)
    .bind(&permission)
    .fetch_one(pool)
    .await
    .expect("deve consultar permissão específica do integrante no CA");

    assert!(
        has_permission,
        "o integrante {student_email} deveria possuir a permissão '{permission}' em {ca}",
    );
}

/// Verifica que o estudante deixou de possuir a permissão específica indicada no CA.
#[then(expr = "{string} deixa de possuir a permissão de {string} em {string}")]
async fn student_loses_specific_permission_in_ca(
    world: &mut AppWorld,
    student_email: String,
    permission: String,
    ca: String,
) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;

    let has_permission: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1
            FROM ca_management_member m
            JOIN user_profile u ON u.id = m.user_id
            JOIN ca_role_permission p ON p.ca_role_id = m.ca_role_id
            WHERE m.ca_id = $1 AND u.email = $2 AND p.permission = $3
        )",
    )
    .bind(ca_id)
    .bind(&student_email)
    .bind(&permission)
    .fetch_one(pool)
    .await
    .expect("deve consultar permissão específica do integrante no CA");

    assert!(
        !has_permission,
        "o integrante {student_email} não deveria possuir a permissão '{permission}' em {ca}",
    );
}

/// Pendente: {string} continua podendo acessar {string} como estudante.
#[then(expr = "{string} continua podendo acessar {string} como estudante")]
async fn student_retains_regular_access_to_ca(
    _world: &mut AppWorld,
    _student_email: String,
    _ca: String,
) {
    pending_step("{string} continua podendo acessar {string} como estudante");
}

/// Verifica que a operação foi recusada por falta de permissão.
#[then("a operação é recusada por falta de permissão")]
async fn operation_is_rejected_for_lack_of_permission(world: &mut AppWorld) {
    let (status, reason) = fetch_last_operation_outcome(world.database_pool()).await;
    assert_eq!(status, "rejected");
    assert_eq!(reason.as_deref(), Some("permission_denied"));
}

/// Verifica que a operação foi recusada porque o usuário não possui perfil no CApp.
#[then("a operação é recusada informando que o usuário não foi encontrado")]
async fn operation_is_rejected_user_not_found(world: &mut AppWorld) {
    let (status, reason) = fetch_last_operation_outcome(world.database_pool()).await;
    assert_eq!(status, "rejected");
    assert_eq!(reason.as_deref(), Some("user_not_found"));
}

/// Verifica que a operação foi recusada porque o cargo não existe no CA.
#[then("a operação é recusada informando que o cargo não existe no CA")]
async fn operation_is_rejected_role_not_found(world: &mut AppWorld) {
    let (status, reason) = fetch_last_operation_outcome(world.database_pool()).await;
    assert_eq!(status, "rejected");
    assert_eq!(reason.as_deref(), Some("role_not_found"));
}

/// Verifica que a operação foi recusada porque o usuário já integra a gestão do CA.
#[then("a operação é recusada informando que o usuário já integra a gestão do CA")]
async fn operation_is_rejected_user_already_in_management(world: &mut AppWorld) {
    let (status, reason) = fetch_last_operation_outcome(world.database_pool()).await;
    assert_eq!(status, "rejected");
    assert_eq!(reason.as_deref(), Some("already_in_management"));
}

/// Verifica que a operação foi recusada porque o usuário não integra a gestão do CA.
#[then("a operação é recusada informando que o usuário não integra a gestão do CA")]
async fn operation_is_rejected_user_not_in_management(world: &mut AppWorld) {
    let (status, reason) = fetch_last_operation_outcome(world.database_pool()).await;
    assert_eq!(status, "rejected");
    assert_eq!(reason.as_deref(), Some("not_in_management"));
}

/// Verifica que a operação foi recusada porque o integrante já exerce o cargo informado.
#[then("a operação é recusada informando que o integrante já exerce esse cargo")]
async fn operation_is_rejected_member_already_holds_role(world: &mut AppWorld) {
    let (status, reason) = fetch_last_operation_outcome(world.database_pool()).await;
    assert_eq!(status, "rejected");
    assert_eq!(reason.as_deref(), Some("already_holds_role"));
}

/// Pendente: a operação é recusada informando que a gestão precisa manter ao menos um integrante com essa permissão.
#[then(
    "a operação é recusada informando que a gestão precisa manter ao menos um integrante com essa permissão"
)]
async fn operation_is_rejected_last_manager_protection(_world: &mut AppWorld) {
    pending_step(
        "a operação é recusada informando que a gestão precisa manter ao menos um integrante com essa permissão",
    );
}

// ============================================================================
// Histórico (Contexto prévio, ações e verificações do histórico administrativo)
// ============================================================================

/// Pendente: que o estudante {string} foi vinculado ao cargo {string} em {string}.
#[given(expr = "que o estudante {string} foi vinculado ao cargo {string} em {string}")]
async fn student_was_previously_assigned_to_role(
    _world: &mut AppWorld,
    _student_email: String,
    _role: String,
    _ca: String,
) {
    pending_step("que o estudante {string} foi vinculado ao cargo {string} em {string}");
}

/// Pendente: posteriormente teve o cargo alterado para {string} em {string}.
#[given(expr = "posteriormente teve o cargo alterado para {string} em {string}")]
async fn student_subsequently_had_role_changed(
    _world: &mut AppWorld,
    _new_role: String,
    _ca: String,
) {
    pending_step("posteriormente teve o cargo alterado para {string} em {string}");
}

/// Pendente: que o histórico de {string} registra a remoção de {string} do cargo {string}.
#[given(expr = "que o histórico de {string} registra a remoção de {string} do cargo {string}")]
async fn history_contains_removal_record(
    _world: &mut AppWorld,
    _ca: String,
    _student_email: String,
    _role: String,
) {
    pending_step("que o histórico de {string} registra a remoção de {string} do cargo {string}");
}

/// Pendente: que o histórico de {string} registra a vinculação de {string} ao cargo {string}.
#[given(expr = "que o histórico de {string} registra a vinculação de {string} ao cargo {string}")]
async fn history_contains_assignment_record(
    _world: &mut AppWorld,
    _ca: String,
    _student_email: String,
    _role: String,
) {
    pending_step("que o histórico de {string} registra a vinculação de {string} ao cargo {string}");
}

/// Pendente: o gestor {string} tenta {word} esse registro do histórico.
#[when(expr = "o gestor {string} tenta {word} esse registro do histórico")]
async fn manager_attempts_to_modify_history_record(
    _world: &mut AppWorld,
    _manager_email: String,
    _action: String,
) {
    pending_step("o gestor {string} tenta {word} esse registro do histórico");
}

/// Pendente: o histórico administrativo de {string} registra a vinculação de {string} ao cargo {string}.
#[then(
    expr = "o histórico administrativo de {string} registra a vinculação de {string} ao cargo {string}"
)]
async fn history_records_member_assignment(
    _world: &mut AppWorld,
    _ca: String,
    _student_email: String,
    _role: String,
) {
    pending_step(
        "o histórico administrativo de {string} registra a vinculação de {string} ao cargo {string}",
    );
}

/// Pendente: o histórico administrativo de {string} registra a alteração de {string} do cargo {string} para {string}.
#[then(
    expr = "o histórico administrativo de {string} registra a alteração de {string} do cargo {string} para {string}"
)]
async fn history_records_role_change(
    _world: &mut AppWorld,
    _ca: String,
    _student_email: String,
    _old_role: String,
    _new_role: String,
) {
    pending_step(
        "o histórico administrativo de {string} registra a alteração de {string} do cargo {string} para {string}",
    );
}

/// Pendente: o histórico administrativo de {string} registra a remoção de {string} do cargo {string}.
#[then(
    expr = "o histórico administrativo de {string} registra a remoção de {string} do cargo {string}"
)]
async fn history_records_member_removal(
    _world: &mut AppWorld,
    _ca: String,
    _student_email: String,
    _role: String,
) {
    pending_step(
        "o histórico administrativo de {string} registra a remoção de {string} do cargo {string}",
    );
}

/// Pendente: o registro identifica o gestor {string} como responsável e a data da operação.
#[then(expr = "o registro identifica o gestor {string} como responsável e a data da vinculação")]
#[then(expr = "o registro identifica o gestor {string} como responsável e a data da alteração")]
#[then(expr = "o registro identifica o gestor {string} como responsável e a data da remoção")]
async fn history_record_identifies_responsible_manager_and_date(
    _world: &mut AppWorld,
    _manager_email: String,
) {
    pending_step("o registro identifica o gestor {string} como responsável e a data da operação");
}

/// Pendente: o histórico administrativo de {string} apresenta, em ordem cronológica, a vinculação, a alteração e a remoção de {string}.
#[then(
    expr = "o histórico administrativo de {string} apresenta, em ordem cronológica, a vinculação, a alteração e a remoção de {string}"
)]
async fn history_presents_chronological_lifecycle(
    _world: &mut AppWorld,
    _ca: String,
    _student_email: String,
) {
    pending_step(
        "o histórico administrativo de {string} apresenta, em ordem cronológica, a vinculação, a alteração e a remoção de {string}",
    );
}

/// Pendente: os registros anteriores à remoção permanecem inalterados.
#[then("os registros anteriores à remoção permanecem inalterados")]
async fn records_prior_to_removal_remain_unchanged(_world: &mut AppWorld) {
    pending_step("os registros anteriores à remoção permanecem inalterados");
}

/// Pendente: o histórico administrativo de {string} registra a nova vinculação.
#[then(expr = "o histórico administrativo de {string} registra a nova vinculação")]
async fn history_records_new_assignment(_world: &mut AppWorld, _ca: String) {
    pending_step("o histórico administrativo de {string} registra a nova vinculação");
}

/// Pendente: os registros da passagem anterior de {string} pela gestão permanecem disponíveis.
#[then(expr = "os registros da passagem anterior de {string} pela gestão permanecem disponíveis")]
async fn previous_tenure_records_remain_available(_world: &mut AppWorld, _student_email: String) {
    pending_step(
        "os registros da passagem anterior de {string} pela gestão permanecem disponíveis",
    );
}

/// Pendente: a operação é recusada informando que o histórico administrativo não pode ser modificado.
#[then("a operação é recusada informando que o histórico administrativo não pode ser modificado")]
async fn operation_is_rejected_history_is_immutable(_world: &mut AppWorld) {
    pending_step(
        "a operação é recusada informando que o histórico administrativo não pode ser modificado",
    );
}

/// Pendente: o registro permanece inalterado no histórico de {string}.
#[then(expr = "o registro permanece inalterado no histórico de {string}")]
async fn history_record_remains_unchanged(_world: &mut AppWorld, _ca: String) {
    pending_step("o registro permanece inalterado no histórico de {string}");
}

/// Verifica que nenhum novo registro foi adicionado ao histórico administrativo do CA.
#[then(expr = "nenhum novo registro é adicionado ao histórico administrativo de {string}")]
async fn no_new_record_is_added_to_history(world: &mut AppWorld, ca: String) {
    let pool = world.database_pool();
    let ca_id = find_ca_id(pool, &ca).await;

    let before_count: i64 =
        sqlx::query_scalar("SELECT history_count FROM bdd_history_snapshot WHERE ca_id = $1")
            .bind(ca_id)
            .fetch_one(pool)
            .await
            .expect("deve existir retrato prévio do histórico administrativo");

    let current_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM ca_management_history WHERE ca_id = $1")
            .bind(ca_id)
            .fetch_one(pool)
            .await
            .expect("deve contar registros atuais do histórico administrativo");

    assert_eq!(
        current_count, before_count,
        "nenhum novo registro deveria ter sido adicionado ao histórico de {ca}",
    );
}
