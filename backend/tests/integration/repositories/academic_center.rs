//! Contratos SQL de Centro Acadêmico: banco PostgreSQL descartável com migrations reais.
//!
//! Valida a integridade do esquema relacional, chaves estrangeiras institucionais,
//! opcionalidade da sigla (NULL), ausência de unicidade global e check constraints.

use crate::support::TestDatabase;
use backend::modules::ca::domain::entities::AcademicCenter;
use backend::modules::ca::domain::entities::AcademicCenterStatus;
use chrono::Utc;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;
use backend::infrastructure::database::PgDatabase;
use backend::modules::ca::application::ports::PollRepository;
use backend::modules::ca::domain::entities::PollStatus;
use backend::modules::ca::domain::entities::PollVote;
use backend::modules::ca::domain::entities::StudentPoll;
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
#[tokio::test]
async fn should_persist_and_retrieve_mandate_with_pg_database() {
    use backend::infrastructure::database::PgDatabase;
    use backend::modules::ca::application::ports::MandateRepository;
    use backend::modules::ca::domain::entities::{Mandate, MandateStatus};
    use chrono::NaiveDate;

    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "Universidade Mandato").await;
    let user_id = Uuid::new_v4();
    let ca_id = Uuid::new_v4();

    // Cria app_user para FK
    sqlx::query("INSERT INTO app_user (id, firebase_uid) VALUES ($1, $2)")
        .bind(user_id)
        .bind(format!("firebase-{user_id}"))
        .execute(&db.pool)
        .await
        .expect("should insert app_user");

    // Cria academic_center para FK
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, status) \
         VALUES ($1, $2, $3, 'Centro Acadêmico Mandato', 'CAM', 'active')",
    )
    .bind(ca_id)
    .bind(institution_id)
    .bind(user_id)
    .execute(&db.pool)
    .await
    .expect("should insert academic_center");

    let pg_db = PgDatabase::new(db.pool.clone());
    let start_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();

    let mandate = Mandate::new(ca_id, user_id, Some("Gestão 2026"), start_date, end_date)
        .expect("valid mandate");

    // Salva via trait MandateRepository
    pg_db.save(&mandate).await.expect("should save mandate");

    // Recupera por ID
    let found = pg_db
        .find_by_id(mandate.id())
        .await
        .expect("should query by id")
        .expect("mandate should be found");

    assert_eq!(found.id(), mandate.id());
    assert_eq!(found.academic_center_id(), ca_id);
    assert_eq!(found.created_by_id(), user_id);
    assert_eq!(found.title(), Some("Gestão 2026"));
    assert_eq!(found.start_date(), start_date);
    assert_eq!(found.end_date(), end_date);
    assert_eq!(found.status(), MandateStatus::Active);
    assert_eq!(found.duration_in_days(), 364);

    // Recupera ativo por CA ID
    let active = pg_db
        .find_active_by_academic_center_id(ca_id)
        .await
        .expect("should query active")
        .expect("active mandate should be found");
    assert_eq!(active.id(), mandate.id());

    // Lista todos
    let all = pg_db
        .find_all_by_academic_center_id(ca_id)
        .await
        .expect("should list all");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id(), mandate.id());

    // Testa view plural academic_center_mandates
    let view_row = sqlx::query(
        "SELECT COUNT(*) as cnt FROM academic_center_mandates WHERE academic_center_id = $1",
    )
    .bind(ca_id)
    .fetch_one(&db.pool)
    .await
    .expect("should query view");
    let count: i64 = view_row.get("cnt");
    assert_eq!(count, 1);

    db.close().await;
}
#[tokio::test]
async fn should_enforce_mandate_database_constraints() {
    use backend::infrastructure::database::PgDatabase;
    use backend::modules::ca::application::ports::{MandateRepository, MandateRepositoryError};
    use backend::modules::ca::domain::entities::Mandate;
    use chrono::NaiveDate;

    let db = TestDatabase::new().await;
    let institution_id =
        create_test_institution(&db.pool, "Universidade Constraints Mandato").await;
    let user_id = Uuid::new_v4();
    let ca_id = Uuid::new_v4();

    sqlx::query("INSERT INTO app_user (id, firebase_uid) VALUES ($1, $2)")
        .bind(user_id)
        .bind(format!("firebase-{user_id}"))
        .execute(&db.pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, status) \
         VALUES ($1, $2, $3, 'CA Constraints', 'CAC', 'active')",
    )
    .bind(ca_id)
    .bind(institution_id)
    .bind(user_id)
    .execute(&db.pool)
    .await
    .unwrap();

    let pg_db = PgDatabase::new(db.pool.clone());

    // 1. Constraint CHECK: end_date <= start_date no SQL
    let invalid_dates_res = sqlx::query(
        "INSERT INTO academic_center_mandate (academic_center_id, created_by_id, start_date, end_date, status) \
         VALUES ($1, $2, '2026-12-31', '2026-01-01', 'active')",
    )
    .bind(ca_id)
    .bind(user_id)
    .execute(&db.pool)
    .await;
    assert!(
        invalid_dates_res.is_err(),
        "deve rejeitar end_date <= start_date via check constraint"
    );

    // 2. Constraint CHECK: status inválido
    let invalid_status_res = sqlx::query(
        "INSERT INTO academic_center_mandate (academic_center_id, created_by_id, start_date, end_date, status) \
         VALUES ($1, $2, '2026-01-01', '2026-12-31', 'invalid_status')",
    )
    .bind(ca_id)
    .bind(user_id)
    .execute(&db.pool)
    .await;
    assert!(
        invalid_status_res.is_err(),
        "deve rejeitar status inválido via check constraint"
    );

    // 3. Unicidade de mandato ativo por CA (índice parcial)
    let m1 = Mandate::new(
        ca_id,
        user_id,
        Some("Primeiro Ativo"),
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .unwrap();
    pg_db.save(&m1).await.unwrap();

    let m2 = Mandate::new(
        ca_id,
        user_id,
        Some("Segundo Ativo Conflitante"),
        NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2027, 12, 31).unwrap(),
    )
    .unwrap();

    let conflict_err = pg_db.save(&m2).await.unwrap_err();
    assert!(matches!(conflict_err, MandateRepositoryError::Conflict(_)));

    db.close().await;
}
#[tokio::test]
async fn should_preserve_concluded_mandates_and_members_history() {
    use backend::infrastructure::database::PgDatabase;
    use backend::modules::ca::application::ports::MandateRepository;
    use backend::modules::ca::domain::entities::{Mandate, MandateMember};
    use chrono::{Duration, NaiveDate};

    let db = TestDatabase::new().await;
    let institution_id =
        create_test_institution(&db.pool, "Universidade Historico Repositorio").await;
    let user_id = Uuid::new_v4();
    let ca_id = Uuid::new_v4();

    sqlx::query("INSERT INTO app_user (id, firebase_uid) VALUES ($1, $2)")
        .bind(user_id)
        .bind(format!("firebase-{user_id}"))
        .execute(&db.pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, status) \
         VALUES ($1, $2, $3, 'CA Histórico', 'CAH', 'active')",
    )
    .bind(ca_id)
    .bind(institution_id)
    .bind(user_id)
    .execute(&db.pool)
    .await
    .unwrap();

    let repo = PgDatabase::new(db.pool.clone());

    // 1. Cria primeiro mandato com membros
    let mut m1 = Mandate::new(
        ca_id,
        user_id,
        Some("Gestão 2025"),
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
    )
    .unwrap();

    let member_1 = MandateMember::new("Presidente 2025", "Presidente", Some(user_id)).unwrap();
    let member_2 = MandateMember::new("Secretário Geral", "Secretário", None).unwrap();
    m1.add_member(member_1.clone());
    m1.add_member(member_2.clone());

    repo.save(&m1).await.expect("salva m1 com membros");

    // 2. Conclui m1 e desliga um membro antes
    m1.disaffiliate_member(member_2.id(), m1.created_at() + Duration::days(180))
        .unwrap();
    m1.conclude(m1.created_at() + Duration::days(365)).unwrap();
    repo.save(&m1).await.expect("atualiza m1 concluído");

    // 3. Cria segundo mandato com novo membro
    let mut m2 = Mandate::new(
        ca_id,
        user_id,
        Some("Gestão 2026"),
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .unwrap();
    let member_3 = MandateMember::new("Presidente 2026", "Presidente", Some(user_id)).unwrap();
    m2.add_member(member_3);
    repo.save(&m2).await.expect("salva m2");

    // 4. Consulta histórico completo (find_all_by_academic_center_id)
    let all = repo.find_all_by_academic_center_id(ca_id).await.unwrap();
    assert_eq!(all.len(), 2);
    // Ordenação: mais recente primeiro (start_date DESC)
    assert_eq!(all[0].id(), m2.id());
    assert_eq!(all[1].id(), m1.id());

    // Verifica integridade imutável dos dados preservados de m1
    let loaded_m1 = &all[1];
    assert!(loaded_m1.is_concluded());
    assert!(loaded_m1.concluded_at().is_some());
    assert_eq!(loaded_m1.members().len(), 2);

    let loaded_m1_m2 = loaded_m1
        .members()
        .iter()
        .find(|m| m.id() == member_2.id())
        .unwrap();
    assert!(!loaded_m1_m2.is_active());
    assert!(loaded_m1_m2.left_at().is_some());

    let loaded_m1_m1 = loaded_m1
        .members()
        .iter()
        .find(|m| m.id() == member_1.id())
        .unwrap();
    assert!(loaded_m1_m1.is_active());

    // 5. find_active_by_academic_center_id retorna apenas m2
    let active = repo
        .find_active_by_academic_center_id(ca_id)
        .await
        .unwrap()
        .expect("deve ter ativo");
    assert_eq!(active.id(), m2.id());

    // 6. find_by_id recupera mandate com membros
    let by_id = repo
        .find_by_id(m1.id())
        .await
        .unwrap()
        .expect("encontra m1 por id");
    assert_eq!(by_id.members().len(), 2);

    db.close().await;
}
/// Helper para inserir usuário de teste na tabela app_user com perfil ativo válido.
async fn create_test_user(pool: &PgPool, uid: &str, matricula: &str) -> Uuid {
    let id = Uuid::new_v4();
    let email = format!("{matricula}@aluno.unb.br");
    sqlx::query(
        "INSERT INTO app_user (\
            id, firebase_uid, status, nickname, email, \
            current_semester, institution_entry_semester, profile_completed_at, \
            created_at, updated_at\
         ) VALUES ($1, $2, 'active', 'User Test', $3, 1, '2024.1', NOW(), NOW(), NOW())",
    )
    .bind(id)
    .bind(uid)
    .bind(email)
    .execute(pool)
    .await
    .expect("should insert test user");
    id
}
#[tokio::test]
async fn should_persist_and_retrieve_student_poll_in_database() {
    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "Universidade de Brasília").await;
    let manager_id = create_test_user(&db.pool, "firebase-manager-test", "241098765").await;

    let ca_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, description, status) \
         VALUES ($1, $2, $3, 'Centro Acadêmico de Computação', 'CACC', 'Descrição', 'active')",
    )
    .bind(ca_id)
    .bind(institution_id)
    .bind(manager_id)
    .execute(&db.pool)
    .await
    .unwrap();

    let repo = PgDatabase::new(db.pool.clone());
    let now = Utc::now();

    let poll = StudentPoll::new(
        ca_id,
        manager_id,
        "Consulta sobre Horários".to_string(),
        Some("Descrição da proposta".to_string()),
        vec!["Opção 1".to_string(), "Opção 2".to_string()],
        now,
        Some(now + chrono::Duration::days(5)),
        Some("all_students".to_string()),
        Some("identified".to_string()),
    )
    .expect("poll should be valid");

    repo.save(&poll).await.expect("should persist poll");

    let retrieved = repo
        .find_by_id(poll.id())
        .await
        .expect("query should succeed")
        .expect("poll should be found");

    assert_eq!(retrieved.id(), poll.id());
    assert_eq!(retrieved.academic_center_id(), ca_id);
    assert_eq!(retrieved.created_by_id(), manager_id);
    assert_eq!(retrieved.title(), "Consulta sobre Horários");
    assert_eq!(retrieved.description(), Some("Descrição da proposta"));
    assert_eq!(retrieved.status(), PollStatus::Open);
    assert_eq!(retrieved.options().len(), 2);
    assert_eq!(retrieved.options()[0].title(), "Opção 1");
    assert_eq!(retrieved.options()[1].title(), "Opção 2");

    db.close().await;
}
#[tokio::test]
async fn should_record_votes_and_enforce_unique_poll_user_in_database() {
    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "Universidade de Brasília").await;
    let manager_id = create_test_user(&db.pool, "firebase-manager-vote", "241098765").await;
    let student_1 = create_test_user(&db.pool, "firebase-student-1", "241098766").await;
    let student_2 = create_test_user(&db.pool, "firebase-student-2", "241098767").await;

    let ca_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, description, status) \
         VALUES ($1, $2, $3, 'Centro Acadêmico de Redes', 'CAER', 'Descrição', 'active')",
    )
    .bind(ca_id)
    .bind(institution_id)
    .bind(manager_id)
    .execute(&db.pool)
    .await
    .unwrap();

    let repo = PgDatabase::new(db.pool.clone());
    let now = Utc::now();

    let poll = StudentPoll::new(
        ca_id,
        manager_id,
        "Consulta de Votação".to_string(),
        None,
        vec!["Sim".to_string(), "Não".to_string()],
        now,
        None,
        None,
        None,
    )
    .expect("poll should be valid");

    repo.save(&poll).await.expect("should persist poll");

    let opt_1 = poll.options()[0].id();
    let opt_2 = poll.options()[1].id();

    // Voto do estudante 1
    let vote_1 = PollVote::new(poll.id(), opt_1, student_1, now);
    repo.save_vote(&vote_1)
        .await
        .expect("vote 1 should be saved");

    // Voto do estudante 2
    let vote_2 = PollVote::new(poll.id(), opt_2, student_2, now);
    repo.save_vote(&vote_2)
        .await
        .expect("vote 2 should be saved");

    // Tentativa de voto duplicado pelo estudante 1
    let vote_dup = PollVote::new(poll.id(), opt_2, student_1, now);
    let err_dup = repo
        .save_vote(&vote_dup)
        .await
        .expect_err("should reject duplicate");
    assert!(matches!(
        err_dup,
        backend::modules::ca::application::ports::PollRepositoryError::Conflict(_)
    ));

    // Checagem de has_user_voted
    assert!(repo.has_user_voted(poll.id(), student_1).await.unwrap());
    assert!(repo.has_user_voted(poll.id(), student_2).await.unwrap());
    assert!(
        !repo
            .has_user_voted(poll.id(), Uuid::new_v4())
            .await
            .unwrap()
    );

    // Apuração de contagem
    let counts = repo.count_votes_by_option(poll.id()).await.unwrap();
    assert_eq!(counts.get(&opt_1).copied().unwrap_or(0), 1);
    assert_eq!(counts.get(&opt_2).copied().unwrap_or(0), 1);

    // Encerramento da consulta
    let close_time = Utc::now();
    let closed = repo
        .close_poll(poll.id(), manager_id, close_time)
        .await
        .expect("should close poll");
    assert_eq!(closed.status(), PollStatus::Closed);

    db.close().await;
}
