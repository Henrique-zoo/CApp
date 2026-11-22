//! Contratos SQL de Demandas: banco PostgreSQL descartável com migrations reais.
//!
//! Valida a integridade relacional, chaves estrangeiras com academic_center e app_user,
//! integridade do fórum institucional vs espaço físico, restrições CHECK e histórico de status.

use crate::support::TestDatabase;
use backend::modules::demands::domain::entities::{
    Demand, DemandForum, DemandStatus, DemandStatusHistoryEntry,
};
use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Cria uma instituição no banco de teste.
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

/// Cria um Centro Acadêmico no banco de teste.
async fn create_test_ca(pool: &PgPool, institution_id: Uuid, name: &str, acronym: &str) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, name, acronym, description, status) \
         VALUES ($1, $2, $3, $4, 'CA para testes de demandas', 'active')",
    )
    .bind(id)
    .bind(institution_id)
    .bind(name)
    .bind(acronym)
    .execute(pool)
    .await
    .expect("should insert test academic center");

    id
}

/// Cria um usuário no banco de teste.
async fn create_test_user(pool: &PgPool, firebase_uid: &str) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query("INSERT INTO app_user (id, firebase_uid) VALUES ($1, $2)")
        .bind(id)
        .bind(firebase_uid)
        .execute(pool)
        .await
        .expect("should insert test user");

    id
}

#[tokio::test]
async fn should_persist_and_retrieve_physical_space_demand_and_reconstitute() {
    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "Universidade de Brasília").await;
    let ca_id = create_test_ca(
        &db.pool,
        institution_id,
        "Centro Acadêmico de Computação",
        "CACC",
    )
    .await;
    let author_id = create_test_user(&db.pool, "student-author-1").await;
    let gestor_id = create_test_user(&db.pool, "manager-user-1").await;

    let demand_id = Uuid::new_v4();
    let history_id = Uuid::new_v4();
    let now = chrono::DateTime::from_timestamp_micros(Utc::now().timestamp_micros()).unwrap();
    let updated = now + chrono::Duration::hours(2);

    sqlx::query(
        "INSERT INTO demand (id, academic_center_id, author_id, forum, title, description, category, course_name, status, created_at, updated_at) \
         VALUES ($1, $2, $3, 'physical_space', $4, $5, $6, NULL, 'in_analysis', $7, $8)",
    )
    .bind(demand_id)
    .bind(ca_id)
    .bind(author_id)
    .bind("Reparo da porta")
    .bind("A porta da sala do CA está danificada")
    .bind("Infraestrutura")
    .bind(now)
    .bind(updated)
    .execute(&db.pool)
    .await
    .expect("should insert physical space demand");

    sqlx::query(
        "INSERT INTO demand_status_history (id, demand_id, from_status, to_status, changed_by_id, comment, created_at) \
         VALUES ($1, $2, 'open', 'in_analysis', $3, 'Iniciada análise de reparo', $4)",
    )
    .bind(history_id)
    .bind(demand_id)
    .bind(gestor_id)
    .bind(updated)
    .execute(&db.pool)
    .await
    .expect("should insert status history");

    let row = sqlx::query(
        "SELECT id, academic_center_id, author_id, forum, title, description, category, course_name, status, created_at, updated_at \
         FROM demand WHERE id = $1",
    )
    .bind(demand_id)
    .fetch_one(&db.pool)
    .await
    .expect("should fetch demand");

    let hist_rows = sqlx::query(
        "SELECT id, demand_id, from_status, to_status, changed_by_id, comment, created_at \
         FROM demand_status_history WHERE demand_id = $1 ORDER BY created_at ASC",
    )
    .bind(demand_id)
    .fetch_all(&db.pool)
    .await
    .expect("should fetch history");

    let mut history_entries = Vec::new();
    for h in hist_rows {
        let from_status_str: Option<String> = h.get("from_status");
        let to_status_str: String = h.get("to_status");

        let from_status =
            from_status_str.map(|s| s.parse::<DemandStatus>().expect("valid from_status"));
        let to_status: DemandStatus = to_status_str.parse().expect("valid to_status");

        let entry = DemandStatusHistoryEntry::new(
            h.get("id"),
            h.get("demand_id"),
            from_status,
            to_status,
            h.get("changed_by_id"),
            h.get("comment"),
            h.get("created_at"),
        )
        .expect("valid history entry");

        history_entries.push(entry);
    }

    let forum: DemandForum = row.get::<String, _>("forum").parse().unwrap();
    let status: DemandStatus = row.get::<String, _>("status").parse().unwrap();

    let entity = Demand::reconstitute(
        row.get("id"),
        row.get("academic_center_id"),
        row.get("author_id"),
        forum,
        row.get("title"),
        row.get("description"),
        row.get("category"),
        row.get("course_name"),
        status,
        row.get("created_at"),
        row.get("updated_at"),
        history_entries,
    )
    .expect("should reconstitute domain demand");

    assert_eq!(entity.id(), demand_id);
    assert_eq!(entity.academic_center_id(), ca_id);
    assert_eq!(entity.author_id(), author_id);
    assert_eq!(entity.forum(), DemandForum::PhysicalSpace);
    assert_eq!(entity.title(), Some("Reparo da porta"));
    assert_eq!(
        entity.description(),
        "A porta da sala do CA está danificada"
    );
    assert_eq!(entity.category(), Some("Infraestrutura"));
    assert_eq!(entity.course_name(), None);
    assert_eq!(entity.status(), DemandStatus::InAnalysis);
    assert_eq!(entity.status_history().len(), 1);
    assert_eq!(
        entity.status_history()[0].comment(),
        Some("Iniciada análise de reparo")
    );

    db.close().await;
}

#[tokio::test]
async fn should_persist_and_retrieve_institutional_demand_with_course_and_forwarding() {
    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "Universidade de Brasília").await;
    let ca_id = create_test_ca(
        &db.pool,
        institution_id,
        "Centro Acadêmico de Redes",
        "CAER",
    )
    .await;
    let author_id = create_test_user(&db.pool, "student-author-2").await;
    let gestor_id = create_test_user(&db.pool, "manager-user-2").await;

    let demand_id = Uuid::new_v4();
    let now = chrono::DateTime::from_timestamp_micros(Utc::now().timestamp_micros()).unwrap();
    let forwarded_at = now + chrono::Duration::hours(3);

    sqlx::query(
        "INSERT INTO demand (id, academic_center_id, author_id, forum, title, description, category, course_name, status, created_at, updated_at) \
         VALUES ($1, $2, $3, 'institutional', $4, $5, 'Acadêmico', $6, 'forwarded_to_coordination', $7, $8)",
    )
    .bind(demand_id)
    .bind(ca_id)
    .bind(author_id)
    .bind("Turma adicional de Redes")
    .bind("Fila de espera excessiva para a matéria obrigatória")
    .bind("Engenharia de Redes de Comunicação")
    .bind(now)
    .bind(forwarded_at)
    .execute(&db.pool)
    .await
    .expect("should insert institutional demand");

    sqlx::query(
        "INSERT INTO demand_status_history (demand_id, from_status, to_status, changed_by_id, comment, created_at) \
         VALUES ($1, 'in_analysis', 'forwarded_to_coordination', $2, 'Ofício enviado ao coordenador', $3)",
    )
    .bind(demand_id)
    .bind(gestor_id)
    .bind(forwarded_at)
    .execute(&db.pool)
    .await
    .expect("should insert history");

    let row = sqlx::query(
        "SELECT id, academic_center_id, author_id, forum, title, description, category, course_name, status, created_at, updated_at \
         FROM demand WHERE id = $1",
    )
    .bind(demand_id)
    .fetch_one(&db.pool)
    .await
    .expect("should fetch demand");

    let forum: DemandForum = row.get::<String, _>("forum").parse().unwrap();
    let status: DemandStatus = row.get::<String, _>("status").parse().unwrap();
    let course_name: Option<String> = row.get("course_name");

    assert_eq!(forum, DemandForum::Institutional);
    assert_eq!(status, DemandStatus::ForwardedToCoordination);
    assert_eq!(
        course_name.as_deref(),
        Some("Engenharia de Redes de Comunicação")
    );

    let entity = Demand::reconstitute(
        row.get("id"),
        row.get("academic_center_id"),
        row.get("author_id"),
        forum,
        row.get("title"),
        row.get("description"),
        row.get("category"),
        course_name,
        status,
        row.get("created_at"),
        row.get("updated_at"),
        vec![],
    )
    .expect("should reconstitute institutional demand");

    assert_eq!(entity.forum(), DemandForum::Institutional);
    assert_eq!(entity.status(), DemandStatus::ForwardedToCoordination);
    assert_eq!(
        entity.course_name(),
        Some("Engenharia de Redes de Comunicação")
    );

    db.close().await;
}

#[tokio::test]
async fn should_enforce_foreign_key_constraints_for_academic_center_and_author() {
    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "Universidade FK").await;
    let ca_id = create_test_ca(&db.pool, institution_id, "CA Teste FK", "CATFK").await;
    let author_id = create_test_user(&db.pool, "user-fk-valid").await;

    let non_existent_ca = Uuid::new_v4();
    let orphan_ca_res = sqlx::query(
        "INSERT INTO demand (academic_center_id, author_id, forum, description) \
         VALUES ($1, $2, 'physical_space', 'Descrição válida')",
    )
    .bind(non_existent_ca)
    .bind(author_id)
    .execute(&db.pool)
    .await;
    assert!(
        orphan_ca_res.is_err(),
        "deve falhar por FK ao referenciar CA inexistente"
    );

    let non_existent_author = Uuid::new_v4();
    let orphan_author_res = sqlx::query(
        "INSERT INTO demand (academic_center_id, author_id, forum, description) \
         VALUES ($1, $2, 'physical_space', 'Descrição válida')",
    )
    .bind(ca_id)
    .bind(non_existent_author)
    .execute(&db.pool)
    .await;
    assert!(
        orphan_author_res.is_err(),
        "deve falhar por FK ao referenciar autor inexistente"
    );

    // Inserção válida
    let demand_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO demand (id, academic_center_id, author_id, forum, description) \
         VALUES ($1, $2, $3, 'physical_space', 'Demanda existente')",
    )
    .bind(demand_id)
    .bind(ca_id)
    .bind(author_id)
    .execute(&db.pool)
    .await
    .expect("should insert demand");

    // Impedir exclusão de CA referenciado (ON DELETE RESTRICT)
    let delete_ca_res = sqlx::query("DELETE FROM academic_center WHERE id = $1")
        .bind(ca_id)
        .execute(&db.pool)
        .await;
    assert!(
        delete_ca_res.is_err(),
        "deve impedir exclusão do CA que possui demandas associadas"
    );

    // Impedir exclusão do usuário autor (ON DELETE RESTRICT)
    let delete_author_res = sqlx::query("DELETE FROM app_user WHERE id = $1")
        .bind(author_id)
        .execute(&db.pool)
        .await;
    assert!(
        delete_author_res.is_err(),
        "deve impedir exclusão do autor que possui demandas associadas"
    );

    db.close().await;
}

#[tokio::test]
async fn should_enforce_database_check_constraints_for_demand() {
    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "Universidade Constraints").await;
    let ca_id = create_test_ca(&db.pool, institution_id, "CA Constraints", "CACST").await;
    let author_id = create_test_user(&db.pool, "user-cst").await;

    // 1. Descrição vazia ou só espaços
    let empty_desc_res = sqlx::query(
        "INSERT INTO demand (academic_center_id, author_id, forum, description) \
         VALUES ($1, $2, 'physical_space', '   ')",
    )
    .bind(ca_id)
    .bind(author_id)
    .execute(&db.pool)
    .await;
    assert!(
        empty_desc_res.is_err(),
        "deve rejeitar descrição em branco via check constraint"
    );

    // 2. Fórum inválido
    let invalid_forum_res = sqlx::query(
        "INSERT INTO demand (academic_center_id, author_id, forum, description) \
         VALUES ($1, $2, 'outro_forum', 'Descrição válida')",
    )
    .bind(ca_id)
    .bind(author_id)
    .execute(&db.pool)
    .await;
    assert!(
        invalid_forum_res.is_err(),
        "deve rejeitar fórum desconhecido via check constraint"
    );

    // 3. Status inválido
    let invalid_status_res = sqlx::query(
        "INSERT INTO demand (academic_center_id, author_id, forum, description, status) \
         VALUES ($1, $2, 'physical_space', 'Descrição válida', 'status_invalido')",
    )
    .bind(ca_id)
    .bind(author_id)
    .execute(&db.pool)
    .await;
    assert!(
        invalid_status_res.is_err(),
        "deve rejeitar status inválido via check constraint"
    );

    // 4. Demanda de espaço físico com status forwarded_to_coordination
    let physical_coordination_res = sqlx::query(
        "INSERT INTO demand (academic_center_id, author_id, forum, description, status) \
         VALUES ($1, $2, 'physical_space', 'Descrição válida', 'forwarded_to_coordination')",
    )
    .bind(ca_id)
    .bind(author_id)
    .execute(&db.pool)
    .await;
    assert!(
        physical_coordination_res.is_err(),
        "deve proibir demanda de espaço físico com status forwarded_to_coordination no banco"
    );

    // 5. Demanda de espaço físico com curso informado
    let physical_course_res = sqlx::query(
        "INSERT INTO demand (academic_center_id, author_id, forum, description, course_name) \
         VALUES ($1, $2, 'physical_space', 'Descrição válida', 'Ciência da Computação')",
    )
    .bind(ca_id)
    .bind(author_id)
    .execute(&db.pool)
    .await;
    assert!(
        physical_course_res.is_err(),
        "deve proibir demanda de espaço físico com curso associado no banco"
    );

    // 6. Timestamps inconsistentes
    let now = Utc::now();
    let past = now - chrono::Duration::hours(2);
    let ts_res = sqlx::query(
        "INSERT INTO demand (academic_center_id, author_id, forum, description, created_at, updated_at) \
         VALUES ($1, $2, 'physical_space', 'Descrição válida', $3, $4)",
    )
    .bind(ca_id)
    .bind(author_id)
    .bind(now)
    .bind(past)
    .execute(&db.pool)
    .await;
    assert!(
        ts_res.is_err(),
        "deve rejeitar updated_at anterior a created_at"
    );

    db.close().await;
}

#[tokio::test]
async fn should_enforce_status_history_cascade_and_constraints() {
    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "Universidade Histórico").await;
    let ca_id = create_test_ca(&db.pool, institution_id, "CA Histórico", "CAHIST").await;
    let author_id = create_test_user(&db.pool, "user-hist-author").await;
    let manager_id = create_test_user(&db.pool, "user-hist-manager").await;

    let demand_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO demand (id, academic_center_id, author_id, forum, description) \
         VALUES ($1, $2, $3, 'physical_space', 'Demanda para teste de cascade')",
    )
    .bind(demand_id)
    .bind(ca_id)
    .bind(author_id)
    .execute(&db.pool)
    .await
    .expect("should insert demand");

    // Histórico com to_status inválido
    let invalid_history_res = sqlx::query(
        "INSERT INTO demand_status_history (demand_id, to_status, changed_by_id) \
         VALUES ($1, 'status_fantasma', $2)",
    )
    .bind(demand_id)
    .bind(manager_id)
    .execute(&db.pool)
    .await;
    assert!(
        invalid_history_res.is_err(),
        "deve rejeitar histórico com to_status inválido"
    );

    // Histórico com comentário em branco
    let empty_comment_res = sqlx::query(
        "INSERT INTO demand_status_history (demand_id, to_status, changed_by_id, comment) \
         VALUES ($1, 'in_analysis', $2, '   ')",
    )
    .bind(demand_id)
    .bind(manager_id)
    .execute(&db.pool)
    .await;
    assert!(
        empty_comment_res.is_err(),
        "deve rejeitar histórico com comentário só de espaços"
    );

    // Histórico válido
    sqlx::query(
        "INSERT INTO demand_status_history (demand_id, from_status, to_status, changed_by_id, comment) \
         VALUES ($1, 'open', 'in_analysis', $2, 'Triagem feita')",
    )
    .bind(demand_id)
    .bind(manager_id)
    .execute(&db.pool)
    .await
    .expect("should insert valid history entry");

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM demand_status_history WHERE demand_id = $1")
            .bind(demand_id)
            .fetch_one(&db.pool)
            .await
            .expect("should count history");
    assert_eq!(count, 1);

    // Ao deletar a demanda, o histórico associado é removido via CASCADE
    sqlx::query("DELETE FROM demand WHERE id = $1")
        .bind(demand_id)
        .execute(&db.pool)
        .await
        .expect("should delete demand");

    let count_after: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM demand_status_history WHERE demand_id = $1")
            .bind(demand_id)
            .fetch_one(&db.pool)
            .await
            .expect("should count history after delete");
    assert_eq!(count_after, 0);

    db.close().await;
}

#[tokio::test]
async fn should_create_demand_and_initial_history_atomically_using_postgres_repository() {
    use backend::modules::demands::{
        application::ports::DemandRepository, infrastructure::PostgresDemandRepository,
    };

    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "Universidade Repositório").await;
    let ca_id = create_test_ca(
        &db.pool,
        institution_id,
        "Centro Acadêmico de Redes",
        "CAER",
    )
    .await;
    let author_id = create_test_user(&db.pool, "user-repo-test-1").await;

    let repo = PostgresDemandRepository::new(db.pool.clone());

    let demand_id = Uuid::new_v4();
    let now = Utc::now();
    let demand = Demand::new(
        demand_id,
        ca_id,
        author_id,
        DemandForum::PhysicalSpace,
        Some("Reparo da fechadura".to_string()),
        "A fechadura da porta principal está quebrada".to_string(),
        Some("Manutenção".to_string()),
        None,
        now,
    )
    .expect("demanda válida");

    repo.create(&demand)
        .await
        .expect("deve criar demanda transacionalmente");

    // Verifica persistência na tabela demand
    let retrieved = repo
        .find_by_id(demand_id)
        .await
        .expect("consulta deve ser bem-sucedida")
        .expect("demanda deve existir");

    assert_eq!(retrieved.id(), demand_id);
    assert_eq!(retrieved.academic_center_id(), ca_id);
    assert_eq!(retrieved.author_id(), author_id);
    assert_eq!(retrieved.forum(), DemandForum::PhysicalSpace);
    assert_eq!(retrieved.title(), Some("Reparo da fechadura"));
    assert_eq!(
        retrieved.description(),
        "A fechadura da porta principal está quebrada"
    );
    assert_eq!(retrieved.category(), Some("Manutenção"));
    assert_eq!(retrieved.status(), DemandStatus::Open);

    // Verifica que o histórico inicial de status 'open' foi registrado transacionalmente
    let history_row = sqlx::query(
        "SELECT demand_id, from_status, to_status, changed_by_id, comment
         FROM demand_status_history WHERE demand_id = $1",
    )
    .bind(demand_id)
    .fetch_one(&db.pool)
    .await
    .expect("deve encontrar registro no histórico");

    let h_demand_id: Uuid = history_row.get("demand_id");
    let h_from: Option<String> = history_row.get("from_status");
    let h_to: String = history_row.get("to_status");
    let h_changed_by: Uuid = history_row.get("changed_by_id");
    let h_comment: Option<String> = history_row.get("comment");

    assert_eq!(h_demand_id, demand_id);
    assert_eq!(h_from, None);
    assert_eq!(h_to, "open");
    assert_eq!(h_changed_by, author_id);
    assert_eq!(h_comment, None);

    db.close().await;
}

#[tokio::test]
async fn should_return_none_when_demand_not_found_in_postgres_repository() {
    use backend::modules::demands::{
        application::ports::DemandRepository, infrastructure::PostgresDemandRepository,
    };

    let db = TestDatabase::new().await;
    let repo = PostgresDemandRepository::new(db.pool.clone());

    let non_existent_id = Uuid::new_v4();
    let result = repo
        .find_by_id(non_existent_id)
        .await
        .expect("busca por id inexistente deve retornar Ok");

    assert!(result.is_none());

    db.close().await;
}

#[tokio::test]
async fn should_list_and_filter_demands_with_ca_isolation_in_postgres_repository() {
    use backend::modules::demands::{
        application::ports::{DemandFilter, DemandRepository},
        infrastructure::PostgresDemandRepository,
    };

    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "Universidade de Brasília").await;
    let ca_1 = create_test_ca(
        &db.pool,
        institution_id,
        "Centro Acadêmico de Computação",
        "CACC",
    )
    .await;
    let ca_2 = create_test_ca(
        &db.pool,
        institution_id,
        "Centro Acadêmico de Matemática",
        "CAMAT",
    )
    .await;

    let author_1 = create_test_user(&db.pool, "author-user-1").await;
    let author_2 = create_test_user(&db.pool, "author-user-2").await;

    let repo = PostgresDemandRepository::new(db.pool.clone());
    let now = chrono::DateTime::from_timestamp_micros(Utc::now().timestamp_micros()).unwrap();

    // Demanda 1: CA 1, autor 1, espaço físico, status open
    let d1 = Demand::new(
        Uuid::new_v4(),
        ca_1,
        author_1,
        DemandForum::PhysicalSpace,
        Some("Reparo da porta".to_string()),
        "A porta da sala do CA está quebrada".to_string(),
        Some("Infraestrutura".to_string()),
        None,
        now,
    )
    .unwrap();
    repo.create(&d1).await.expect("d1 deve ser persistida");

    // Demanda 2: CA 1, autor 2, espaço físico, status in_analysis
    let d2 = Demand::new(
        Uuid::new_v4(),
        ca_1,
        author_2,
        DemandForum::PhysicalSpace,
        Some("Troca da lâmpada".to_string()),
        "Lâmpada queimada na sala de estudos".to_string(),
        Some("Infraestrutura".to_string()),
        None,
        now + chrono::Duration::seconds(1),
    )
    .unwrap();
    repo.create(&d2).await.expect("d2 deve ser persistida");
    // Atualiza status de d2 para in_analysis
    sqlx::query("UPDATE demand SET status = 'in_analysis' WHERE id = $1")
        .bind(d2.id())
        .execute(&db.pool)
        .await
        .expect("update status d2");

    // Demanda 3: CA 1, autor 1, institucional, status open
    let d3 = Demand::new(
        Uuid::new_v4(),
        ca_1,
        author_1,
        DemandForum::Institutional,
        Some("Ajuste curricular".to_string()),
        "Pré-requisitos de cálculo".to_string(),
        Some("Pedagógico".to_string()),
        Some("Ciência da Computação".to_string()),
        now + chrono::Duration::seconds(2),
    )
    .unwrap();
    repo.create(&d3).await.expect("d3 deve ser persistida");

    // Demanda 4: CA 2, autor 1, espaço físico, status open (pertence a outro CA)
    let d4 = Demand::new(
        Uuid::new_v4(),
        ca_2,
        author_1,
        DemandForum::PhysicalSpace,
        Some("Pintura da sala".to_string()),
        "Demanda do CA 2".to_string(),
        Some("Infraestrutura".to_string()),
        None,
        now + chrono::Duration::seconds(3),
    )
    .unwrap();
    repo.create(&d4).await.expect("d4 deve ser persistida");

    // 1. Listagem geral do CA 1: retorna d3, d2, d1 em ordem decrescente de criação, e NUNCA d4 (isolamento de CA)
    let list_ca1 = repo
        .list(&DemandFilter {
            academic_center_id: ca_1,
            forum: None,
            status: None,
            author_id: None,
            limit: None,
            offset: None,
        })
        .await
        .expect("listagem ca1 deve ter sucesso");

    assert_eq!(list_ca1.len(), 3);
    assert_eq!(list_ca1[0].id(), d3.id());
    assert_eq!(list_ca1[1].id(), d2.id());
    assert_eq!(list_ca1[2].id(), d1.id());
    assert!(list_ca1.iter().all(|d| d.academic_center_id() == ca_1));

    // 2. Filtro por fórum de espaço físico no CA 1
    let list_ps = repo
        .list(&DemandFilter {
            academic_center_id: ca_1,
            forum: Some(DemandForum::PhysicalSpace),
            status: None,
            author_id: None,
            limit: None,
            offset: None,
        })
        .await
        .expect("filtro physical_space");
    assert_eq!(list_ps.len(), 2);
    assert_eq!(list_ps[0].id(), d2.id());
    assert_eq!(list_ps[1].id(), d1.id());

    // 3. Filtro por fórum institucional no CA 1
    let list_inst = repo
        .list(&DemandFilter {
            academic_center_id: ca_1,
            forum: Some(DemandForum::Institutional),
            status: None,
            author_id: None,
            limit: None,
            offset: None,
        })
        .await
        .expect("filtro institutional");
    assert_eq!(list_inst.len(), 1);
    assert_eq!(list_inst[0].id(), d3.id());

    // 4. Filtro por status in_analysis
    let list_status = repo
        .list(&DemandFilter {
            academic_center_id: ca_1,
            forum: None,
            status: Some(DemandStatus::InAnalysis),
            author_id: None,
            limit: None,
            offset: None,
        })
        .await
        .expect("filtro status");
    assert_eq!(list_status.len(), 1);
    assert_eq!(list_status[0].id(), d2.id());

    // 5. Filtro de autor (demandas próprias de autor_2)
    let list_mine = repo
        .list(&DemandFilter {
            academic_center_id: ca_1,
            forum: None,
            status: None,
            author_id: Some(author_2),
            limit: None,
            offset: None,
        })
        .await
        .expect("filtro autor");
    assert_eq!(list_mine.len(), 1);
    assert_eq!(list_mine[0].id(), d2.id());

    // 6. Paginação (limit e offset)
    let page1 = repo
        .list(&DemandFilter {
            academic_center_id: ca_1,
            forum: None,
            status: None,
            author_id: None,
            limit: Some(2),
            offset: Some(0),
        })
        .await
        .expect("page 1");
    assert_eq!(page1.len(), 2);
    assert_eq!(page1[0].id(), d3.id());
    assert_eq!(page1[1].id(), d2.id());

    let page2 = repo
        .list(&DemandFilter {
            academic_center_id: ca_1,
            forum: None,
            status: None,
            author_id: None,
            limit: Some(2),
            offset: Some(2),
        })
        .await
        .expect("page 2");
    assert_eq!(page2.len(), 1);
    assert_eq!(page2[0].id(), d1.id());

    db.close().await;
}

#[tokio::test]
async fn should_update_demand_status_and_persist_history_entry_atomically() {
    use backend::modules::demands::{
        application::ports::DemandRepository, infrastructure::PostgresDemandRepository,
    };

    let db = TestDatabase::new().await;
    let institution_id = create_test_institution(&db.pool, "Universidade de Brasília").await;
    let ca_id = create_test_ca(
        &db.pool,
        institution_id,
        "Centro Acadêmico de Redes",
        "CAER",
    )
    .await;
    let author_id = create_test_user(&db.pool, "student-author-status-1").await;
    let manager_id = create_test_user(&db.pool, "manager-status-1").await;

    let repo = PostgresDemandRepository::new(db.pool.clone());
    let now = chrono::DateTime::from_timestamp_micros(Utc::now().timestamp_micros()).unwrap();

    let mut demand = Demand::new(
        Uuid::new_v4(),
        ca_id,
        author_id,
        DemandForum::PhysicalSpace,
        Some("Reparo no ar-condicionado".to_string()),
        "Ar-condicionado da sala não refrigera".to_string(),
        Some("Infraestrutura".to_string()),
        None,
        now,
    )
    .unwrap();

    repo.create(&demand)
        .await
        .expect("deve criar demanda inicial");

    // Reconstitui com status inicial
    let retrieved = repo
        .find_by_id(demand.id())
        .await
        .unwrap()
        .expect("demanda criada deve existir");
    assert_eq!(retrieved.status(), DemandStatus::Open);
    assert_eq!(retrieved.status_history().len(), 1);
    assert_eq!(
        retrieved.status_history()[0].to_status(),
        DemandStatus::Open
    );

    // Gestor avança status para in_analysis com parecer
    let opinion_1 = "Análise técnica iniciada";
    let at_1 = now + chrono::Duration::hours(1);
    demand
        .transition_to(
            DemandStatus::InAnalysis,
            manager_id,
            Some(opinion_1.to_string()),
            at_1,
        )
        .unwrap();

    repo.update(&demand)
        .await
        .expect("deve atualizar status no repositório");

    // Verifica persistência e reconstituição do histórico
    let updated_1 = repo
        .find_by_id(demand.id())
        .await
        .unwrap()
        .expect("demanda deve existir");
    assert_eq!(updated_1.status(), DemandStatus::InAnalysis);
    assert_eq!(updated_1.updated_at(), at_1);
    assert_eq!(updated_1.status_history().len(), 2);
    assert_eq!(
        updated_1.status_history()[1].from_status(),
        Some(DemandStatus::Open)
    );
    assert_eq!(
        updated_1.status_history()[1].to_status(),
        DemandStatus::InAnalysis
    );
    assert_eq!(updated_1.status_history()[1].changed_by_id(), manager_id);
    assert_eq!(updated_1.status_history()[1].comment(), Some(opinion_1));

    // Gestor conclui demanda com parecer resolutivo
    let opinion_2 = "Manutenção realizada e equipamento testado com sucesso";
    let at_2 = at_1 + chrono::Duration::hours(2);
    demand
        .transition_to(
            DemandStatus::Completed,
            manager_id,
            Some(opinion_2.to_string()),
            at_2,
        )
        .unwrap();

    repo.update(&demand)
        .await
        .expect("deve concluir demanda no repositório");

    let updated_2 = repo
        .find_by_id(demand.id())
        .await
        .unwrap()
        .expect("demanda deve existir");
    assert_eq!(updated_2.status(), DemandStatus::Completed);
    assert_eq!(updated_2.updated_at(), at_2);
    assert_eq!(updated_2.status_history().len(), 3);
    assert_eq!(
        updated_2.status_history()[2].from_status(),
        Some(DemandStatus::InAnalysis)
    );
    assert_eq!(
        updated_2.status_history()[2].to_status(),
        DemandStatus::Completed
    );
    assert_eq!(updated_2.status_history()[2].changed_by_id(), manager_id);
    assert_eq!(updated_2.status_history()[2].comment(), Some(opinion_2));

    db.close().await;
}
