//! Testes de integração de persistência para o repositório de FAQ no PostgreSQL.

use crate::support::TestDatabase;
use backend::modules::faq::{
    application::ports::FaqRepository, domain::entities::faq_entry::FaqEntry,
    infrastructure::repository::PostgresFaqRepository,
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

/// Helper para criar uma instituição de teste.
async fn create_test_institution(pool: &PgPool, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    let tenant = Uuid::new_v4();
    let group = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO institution (id, name, acronym, microsoft_tenant_id, microsoft_student_group_id, status) \
         VALUES ($1, $2, 'TEST', $3, $4, 'active')",
    )
    .bind(id)
    .bind(name)
    .bind(tenant)
    .bind(group)
    .execute(pool)
    .await
    .expect("falha ao criar instituição de teste");

    id
}

/// Helper para criar um usuário interno no banco.
async fn create_test_user(pool: &PgPool, uid: &str) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO app_user (id, firebase_uid, status) \
         VALUES ($1, $2, 'incomplete')",
    )
    .bind(id)
    .bind(uid)
    .execute(pool)
    .await
    .expect("falha ao criar usuário de teste");

    id
}

/// Helper para criar um Centro Acadêmico de teste.
async fn create_test_academic_center(
    pool: &PgPool,
    institution_id: Uuid,
    creator_id: Option<Uuid>,
    name: &str,
    status: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    let now = chrono::DateTime::from_timestamp_micros(Utc::now().timestamp_micros()).unwrap();

    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, description, status, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, 'TESTCA', 'Descrição do CA de teste', $5, $6, $7)",
    )
    .bind(id)
    .bind(institution_id)
    .bind(creator_id)
    .bind(name)
    .bind(status)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .expect("falha ao criar CA de teste");

    id
}

#[tokio::test]
async fn should_persist_and_retrieve_faq_entry_with_postgres_repository() {
    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "UnB - FAQ").await;
    let user_id = create_test_user(&db.pool, "firebase-user-faq-1").await;
    let ca_id = create_test_academic_center(
        &db.pool,
        institution_id,
        Some(user_id),
        "CA de Computação",
        "active",
    )
    .await;

    let repo = PostgresFaqRepository::new(db.pool.clone());
    let now = chrono::DateTime::from_timestamp_micros(Utc::now().timestamp_micros()).unwrap();
    let entry_id = Uuid::new_v4();

    let entry = FaqEntry::new(
        entry_id,
        ca_id,
        "Como solicitar apoio acadêmico?".to_string(),
        "Entre em contato com a gestão pelo e-mail oficial.".to_string(),
        Some("Apoio Acadêmico".to_string()),
        user_id,
        now,
    )
    .expect("entidade de domínio válida");

    repo.save(&entry).await.expect("deve salvar no PostgreSQL");

    let loaded = repo
        .find_by_id(entry_id)
        .await
        .expect("consulta bem-sucedida")
        .expect("deve encontrar a entrada salva");

    assert_eq!(loaded.id(), entry_id);
    assert_eq!(loaded.academic_center_id(), ca_id);
    assert_eq!(loaded.question(), "Como solicitar apoio acadêmico?");
    assert_eq!(
        loaded.answer(),
        "Entre em contato com a gestão pelo e-mail oficial."
    );
    assert_eq!(loaded.category(), Some("Apoio Acadêmico"));
    assert_eq!(loaded.created_by(), user_id);
    assert_eq!(loaded.updated_by(), user_id);

    // Listagem por CA
    let entries = repo
        .find_by_academic_center_id(ca_id)
        .await
        .expect("deve listar por CA");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id(), entry_id);
}

#[tokio::test]
async fn should_verify_ca_management_access_correctly() {
    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "UnB - Access").await;
    let manager_id = create_test_user(&db.pool, "firebase-manager").await;
    let non_manager_id = create_test_user(&db.pool, "firebase-student").await;

    let active_ca_id = create_test_academic_center(
        &db.pool,
        institution_id,
        Some(manager_id),
        "CA Ativo",
        "active",
    )
    .await;

    let inactive_ca_id = create_test_academic_center(
        &db.pool,
        institution_id,
        Some(manager_id),
        "CA Arquivado",
        "archived",
    )
    .await;

    let repo = PostgresFaqRepository::new(db.pool.clone());

    // 1. Gestor no CA ativo
    let access = repo
        .check_ca_management_access(active_ca_id, manager_id)
        .await
        .expect("consulta sem erro");
    assert!(access.ca_exists);
    assert!(access.ca_active);
    assert!(access.is_ca_manager);

    // 2. Não gestor no CA ativo
    let access = repo
        .check_ca_management_access(active_ca_id, non_manager_id)
        .await
        .expect("consulta sem erro");
    assert!(access.ca_exists);
    assert!(access.ca_active);
    assert!(!access.is_ca_manager);

    // 3. Gestor no CA inativo
    let access = repo
        .check_ca_management_access(inactive_ca_id, manager_id)
        .await
        .expect("consulta sem erro");
    assert!(access.ca_exists);
    assert!(!access.ca_active);

    // 4. CA inexistente
    let access = repo
        .check_ca_management_access(Uuid::new_v4(), manager_id)
        .await
        .expect("consulta sem erro");
    assert!(!access.ca_exists);
    assert!(!access.ca_active);
    assert!(!access.is_ca_manager);
}

#[tokio::test]
async fn should_enforce_database_check_constraints() {
    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "UnB - Constraints").await;
    let user_id = create_test_user(&db.pool, "firebase-user-constraints").await;
    let ca_id = create_test_academic_center(
        &db.pool,
        institution_id,
        Some(user_id),
        "CA Constraints",
        "active",
    )
    .await;

    let now = chrono::DateTime::from_timestamp_micros(Utc::now().timestamp_micros()).unwrap();

    // 1. Pergunta vazia rejeitada pelo banco
    let empty_q_result = sqlx::query(
        "INSERT INTO faq_entry (id, academic_center_id, question, answer, category, created_by, updated_by, created_at, updated_at) \
         VALUES ($1, $2, '   ', 'Resposta válida', NULL, $3, $3, $4, $4)"
    )
    .bind(Uuid::new_v4())
    .bind(ca_id)
    .bind(user_id)
    .bind(now)
    .execute(&db.pool)
    .await;
    assert!(
        empty_q_result.is_err(),
        "banco deve rejeitar pergunta vazia"
    );

    // 2. Resposta vazia rejeitada pelo banco
    let empty_a_result = sqlx::query(
        "INSERT INTO faq_entry (id, academic_center_id, question, answer, category, created_by, updated_by, created_at, updated_at) \
         VALUES ($1, $2, 'Pergunta válida', '   ', NULL, $3, $3, $4, $4)"
    )
    .bind(Uuid::new_v4())
    .bind(ca_id)
    .bind(user_id)
    .bind(now)
    .execute(&db.pool)
    .await;
    assert!(
        empty_a_result.is_err(),
        "banco deve rejeitar resposta vazia"
    );

    // 3. Timestamps inconsistentes rejeitados pelo banco
    let later = now;
    let earlier = now - chrono::Duration::seconds(10);
    let invalid_ts_result = sqlx::query(
        "INSERT INTO faq_entry (id, academic_center_id, question, answer, category, created_by, updated_by, created_at, updated_at) \
         VALUES ($1, $2, 'Pergunta válida', 'Resposta válida', NULL, $3, $3, $4, $5)"
    )
    .bind(Uuid::new_v4())
    .bind(ca_id)
    .bind(user_id)
    .bind(later)
    .bind(earlier)
    .execute(&db.pool)
    .await;
    assert!(
        invalid_ts_result.is_err(),
        "banco deve rejeitar updated_at < created_at"
    );
}
