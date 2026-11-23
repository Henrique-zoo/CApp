//! Contratos SQL de Centro Acadêmico: banco PostgreSQL descartável com migrations reais.
//!
//! Valida a integridade do esquema relacional, chaves estrangeiras institucionais,
//! opcionalidade da sigla (NULL), ausência de unicidade global e check constraints.

use crate::support::TestDatabase;
use backend::modules::ca::domain::entities::{AcademicCenter, AcademicCenterStatus};
use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Cria uma instituição no banco descartável para viabilizar o vínculo de chave estrangeira.
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
    .expect("should insert test institution");

    id
}

#[tokio::test]
async fn should_persist_and_retrieve_academic_center_with_acronym() {
    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "Universidade Teste 1").await;
    let ca_id = Uuid::new_v4();
    let now = chrono::DateTime::from_timestamp_micros(Utc::now().timestamp_micros()).unwrap();

    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, name, acronym, description, status, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(ca_id)
    .bind(institution_id)
    .bind("Centro Acadêmico de Redes")
    .bind("CAER")
    .bind("Descrição do CAER")
    .bind("active")
    .bind(now)
    .bind(now)
    .execute(&db.pool)
    .await
    .expect("should insert academic center with acronym");

    let row = sqlx::query(
        "SELECT id, institution_id, created_by_id, name, acronym, description, status, created_at, updated_at \
         FROM academic_center WHERE id = $1",
    )
    .bind(ca_id)
    .fetch_one(&db.pool)
    .await
    .expect("should retrieve academic center");

    let retrieved_id: Uuid = row.get("id");
    let retrieved_institution_id: Uuid = row.get("institution_id");
    let retrieved_created_by_id: Option<Uuid> = row.get("created_by_id");
    let retrieved_name: String = row.get("name");
    let retrieved_acronym: Option<String> = row.get("acronym");
    let retrieved_description: String = row.get("description");
    let retrieved_status_str: String = row.get("status");
    let retrieved_created_at: chrono::DateTime<Utc> = row.get("created_at");
    let retrieved_updated_at: chrono::DateTime<Utc> = row.get("updated_at");

    assert_eq!(retrieved_id, ca_id);
    assert_eq!(retrieved_institution_id, institution_id);
    assert_eq!(retrieved_created_by_id, None);
    assert_eq!(retrieved_name, "Centro Acadêmico de Redes");
    assert_eq!(retrieved_acronym.as_deref(), Some("CAER"));
    assert_eq!(retrieved_description, "Descrição do CAER");
    assert_eq!(retrieved_status_str, "active");

    let status: AcademicCenterStatus = retrieved_status_str.parse().expect("valid status");
    let entity = AcademicCenter::reconstitute(
        retrieved_id,
        retrieved_institution_id,
        retrieved_created_by_id,
        retrieved_name,
        retrieved_acronym,
        retrieved_description,
        status,
        retrieved_created_at,
        retrieved_updated_at,
    )
    .expect("should reconstitute domain entity");

    assert_eq!(entity.id(), ca_id);
    assert_eq!(entity.institution_id(), institution_id);
    assert_eq!(entity.acronym(), Some("CAER"));
    assert_eq!(entity.status(), AcademicCenterStatus::Active);

    db.close().await;
}

#[tokio::test]
async fn should_persist_and_retrieve_academic_center_without_acronym() {
    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "Universidade Teste 2").await;
    let ca_id = Uuid::new_v4();
    let now = chrono::DateTime::from_timestamp_micros(Utc::now().timestamp_micros()).unwrap();

    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, name, acronym, description, status, created_at, updated_at) \
         VALUES ($1, $2, $3, NULL, $4, 'active', $5, $6)",
    )
    .bind(ca_id)
    .bind(institution_id)
    .bind("Centro Acadêmico Sem Sigla")
    .bind("Descrição geral")
    .bind(now)
    .bind(now)
    .execute(&db.pool)
    .await
    .expect("should insert academic center without acronym");

    let row = sqlx::query(
        "SELECT id, institution_id, created_by_id, name, acronym, description, status, created_at, updated_at \
         FROM academic_center WHERE id = $1",
    )
    .bind(ca_id)
    .fetch_one(&db.pool)
    .await
    .expect("should retrieve academic center");

    let retrieved_acronym: Option<String> = row.get("acronym");
    assert_eq!(retrieved_acronym, None);

    let status: AcademicCenterStatus = row.get::<String, _>("status").parse().unwrap();
    let entity = AcademicCenter::reconstitute(
        row.get("id"),
        row.get("institution_id"),
        row.get("created_by_id"),
        row.get("name"),
        retrieved_acronym,
        row.get("description"),
        status,
        row.get("created_at"),
        row.get("updated_at"),
    )
    .expect("should reconstitute entity without acronym");

    assert_eq!(entity.acronym(), None);
    assert_eq!(entity.name(), "Centro Acadêmico Sem Sigla");

    db.close().await;
}

#[tokio::test]
async fn should_enforce_institutional_foreign_key_relationship() {
    let db = TestDatabase::new().await;
    let non_existent_institution = Uuid::new_v4();

    let result = sqlx::query(
        "INSERT INTO academic_center (institution_id, name, status) VALUES ($1, 'CA Órfão', 'active')",
    )
    .bind(non_existent_institution)
    .execute(&db.pool)
    .await;

    assert!(
        result.is_err(),
        "deve falhar por violação de chave estrangeira ao referenciar instituição inexistente"
    );

    let institution_id = create_test_institution(&db.pool, "Universidade Federal Teste").await;
    let ca_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, name, acronym, status) \
         VALUES ($1, $2, 'Centro Acadêmico de Teste', 'CAT', 'active')",
    )
    .bind(ca_id)
    .bind(institution_id)
    .execute(&db.pool)
    .await
    .expect("should insert valid ca with institution");

    let join_row = sqlx::query(
        "SELECT ca.id AS ca_id, ca.name AS ca_name, inst.id AS inst_id, inst.name AS inst_name \
         FROM academic_center ca \
         JOIN institution inst ON ca.institution_id = inst.id \
         WHERE ca.id = $1",
    )
    .bind(ca_id)
    .fetch_one(&db.pool)
    .await
    .expect("should query join between academic center and institution");

    let retrieved_inst_name: String = join_row.get("inst_name");
    assert_eq!(retrieved_inst_name, "Universidade Federal Teste");

    db.close().await;
}

#[tokio::test]
async fn should_reference_app_user_when_creator_is_informed() {
    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "Universidade do Criador").await;
    let unknown_user_id = Uuid::new_v4();

    let missing_creator = sqlx::query(
        "INSERT INTO academic_center (institution_id, created_by_id, name, status) \
         VALUES ($1, $2, 'CA sem autor conhecido', 'active')",
    )
    .bind(institution_id)
    .bind(unknown_user_id)
    .execute(&db.pool)
    .await;
    assert!(
        missing_creator.is_err(),
        "deve falhar ao referenciar um usuário que não existe em app_user"
    );

    let creator_id = Uuid::new_v4();
    sqlx::query("INSERT INTO app_user (id, firebase_uid) VALUES ($1, $2)")
        .bind(creator_id)
        .bind(format!("creator-{}", creator_id))
        .execute(&db.pool)
        .await
        .expect("should insert creator");

    let ca_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, status) \
         VALUES ($1, $2, $3, 'CA com autor', 'active')",
    )
    .bind(ca_id)
    .bind(institution_id)
    .bind(creator_id)
    .execute(&db.pool)
    .await
    .expect("should insert academic center created by an existing user");

    let stored_creator: Option<Uuid> =
        sqlx::query_scalar("SELECT created_by_id FROM academic_center WHERE id = $1")
            .bind(ca_id)
            .fetch_one(&db.pool)
            .await
            .expect("should read creator");
    assert_eq!(stored_creator, Some(creator_id));

    let delete_creator = sqlx::query("DELETE FROM app_user WHERE id = $1")
        .bind(creator_id)
        .execute(&db.pool)
        .await;
    assert!(
        delete_creator.is_err(),
        "deve impedir apagar o usuário enquanto ele for o autor de um CA"
    );

    db.close().await;
}

#[tokio::test]
async fn should_allow_duplicate_acronyms_without_global_uniqueness() {
    let db = TestDatabase::new().await;
    let inst_1 = create_test_institution(&db.pool, "Instituição Alfa").await;
    let inst_2 = create_test_institution(&db.pool, "Instituição Beta").await;

    let ca1_result = sqlx::query(
        "INSERT INTO academic_center (institution_id, name, acronym, status) \
         VALUES ($1, 'Centro Acadêmico de Computação Alfa', 'CAC', 'active')",
    )
    .bind(inst_1)
    .execute(&db.pool)
    .await;
    assert!(
        ca1_result.is_ok(),
        "primeiro CA com sigla CAC deve ser inserido"
    );

    let ca2_result = sqlx::query(
        "INSERT INTO academic_center (institution_id, name, acronym, status) \
         VALUES ($1, 'Centro Acadêmico de Computação Beta', 'CAC', 'active')",
    )
    .bind(inst_2)
    .execute(&db.pool)
    .await;
    assert!(
        ca2_result.is_ok(),
        "segundo CA com mesma sigla CAC deve ser inserido sem violar unicidade global"
    );

    db.close().await;
}

#[tokio::test]
async fn should_enforce_database_check_constraints() {
    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "Instituição Constraints").await;

    // 1. Nome vazio ou só espaços
    let empty_name_res = sqlx::query(
        "INSERT INTO academic_center (institution_id, name, status) VALUES ($1, '   ', 'active')",
    )
    .bind(institution_id)
    .execute(&db.pool)
    .await;
    assert!(
        empty_name_res.is_err(),
        "deve rejeitar nome em branco via check constraint"
    );

    // 2. Sigla só espaços (quando não NULL)
    let empty_acronym_res = sqlx::query(
        "INSERT INTO academic_center (institution_id, name, acronym, status) VALUES ($1, 'Nome Válido', '   ', 'active')",
    )
    .bind(institution_id)
    .execute(&db.pool)
    .await;
    assert!(
        empty_acronym_res.is_err(),
        "deve rejeitar sigla em branco via check constraint"
    );

    // 3. Status 'draft' ou inválido
    let draft_status_res = sqlx::query(
        "INSERT INTO academic_center (institution_id, name, status) VALUES ($1, 'Nome Válido', 'draft')",
    )
    .bind(institution_id)
    .execute(&db.pool)
    .await;
    assert!(
        draft_status_res.is_err(),
        "deve rejeitar status 'draft' via check constraint"
    );

    // 4. Timestamps inconsistentes
    let now = Utc::now();
    let past = now - chrono::Duration::hours(2);
    let ts_res = sqlx::query(
        "INSERT INTO academic_center (institution_id, name, status, created_at, updated_at) \
         VALUES ($1, 'Nome Válido', 'active', $2, $3)",
    )
    .bind(institution_id)
    .bind(now)
    .bind(past)
    .execute(&db.pool)
    .await;
    assert!(
        ts_res.is_err(),
        "deve rejeitar updated_at anterior a created_at via check constraint"
    );

    db.close().await;
}
