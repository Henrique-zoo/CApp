//! Contratos SQL de identidade: um único PostgreSQL descartável com migrations reais.
//!
//! Cada caso usa instituição e UID próprios. Não lê DATABASE_URL nem acessa o banco
//! de desenvolvimento. Execute `cargo test --test integration repositories::identity::` com Docker.

use crate::support::TestDatabase;
use backend::{
    infrastructure::database::PgDatabase,
    modules::identity::{
        application::{
            error::InstitutionRepositoryError,
            ports::{InstitutionRepository, InstitutionalVerificationRepository, UserRepository},
        },
        domain::{CatalogStatus, entities::institutional_verification::InstitutionalVerification},
    },
};
use chrono::{Duration, Utc};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

/// Cria cadastro e evidência distintos para um caso, sem credenciais institucionais reais.
async fn fixture(pool: &PgPool, student: bool) -> InstitutionalVerification {
    let id = Uuid::new_v4();
    let tenant = Uuid::new_v4();
    let group = Uuid::new_v4();

    sqlx::query("INSERT INTO institution (id,name,acronym,microsoft_tenant_id,microsoft_student_group_id) VALUES ($1,'Instituição fictícia','IF',$2,$3)")
        .bind(id).bind(tenant).bind(group).execute(pool).await.unwrap();
    // Arredonda para microssegundos, a precisão persistida pelo PostgreSQL.

    let checked_at =
        chrono::DateTime::from_timestamp_micros(Utc::now().timestamp_micros()).unwrap();

    InstitutionalVerification {
        user_id: PgDatabase::new(pool.clone())
            .provision(&Uuid::new_v4().to_string())
            .await
            .unwrap()
            .id(),
        profile_name: Some("Nome institucional".into()),
        profile_email: Some("institucional@example.test".into()),
        course_name: Some("Curso".into()),
        microsoft_object_id: Uuid::new_v4(),
        institution_id: id,
        microsoft_tenant_id: tenant,
        microsoft_student_group_id: group,
        student_membership_confirmed: student,
        checked_at,
        expires_at: checked_at + Duration::hours(24),
    }
}

async fn uid(pool: &PgPool, id: Uuid) -> String {
    sqlx::query_scalar("SELECT firebase_uid FROM app_user WHERE id=$1")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Executa as migrations antigas e a nova no mesmo schema isolado, preservando UIDs e prazos.
async fn legacy_evidence_migration(pool: &PgPool) {
    let mut tx = pool.begin().await.unwrap();
    let schema = format!("legacy_{}", Uuid::new_v4().simple());

    sqlx::raw_sql(&format!(
        "CREATE SCHEMA {schema}; SET LOCAL search_path TO {schema};"
    ))
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::raw_sql(include_str!(
        "../../../migrations/20261019205625_create_institution.sql"
    ))
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::raw_sql(include_str!(
        "../../../migrations/20261019211713_create_institutional_verification.sql"
    ))
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::raw_sql("INSERT INTO institution (name,microsoft_tenant_id,microsoft_student_group_id) VALUES ('Legada',gen_random_uuid(),gen_random_uuid()); INSERT INTO institutional_verification SELECT 'legacy-uid',gen_random_uuid(),id,microsoft_tenant_id,microsoft_student_group_id,true,now(),now()+interval '1 day' FROM institution;").execute(&mut *tx).await.unwrap();

    let expiry: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT expires_at FROM institutional_verification")
            .fetch_one(&mut *tx)
            .await
            .unwrap();

    sqlx::raw_sql(include_str!(
        "../../../migrations/20261111210000_provision_internal_users.sql"
    ))
    .execute(&mut *tx)
    .await
    .unwrap();

    let (uid, status, restored_expiry): (String, String, chrono::DateTime<Utc>) = sqlx::query_as("SELECT u.firebase_uid,u.status::text,v.expires_at FROM institutional_verification v JOIN app_user u ON u.id=v.user_id").fetch_one(&mut *tx).await.unwrap();

    assert_eq!(uid, "legacy-uid");
    assert_eq!(status, "incomplete");
    assert_eq!(expiry, restored_expiry);

    tx.rollback().await.unwrap();
}

/// Provisionamento concorrente preserva UID, ID, status e os dados do perfil.
async fn user_provisioning(pool: &PgPool, repo: &PgDatabase) {
    use backend::modules::identity::domain::{UserStatus, entities::user::ProfileCompletion};

    let uid = Uuid::new_v4().to_string();
    let (a, b) = tokio::join!(repo.provision(&uid), repo.provision(&uid));
    let mut user = a.unwrap();

    assert_eq!(user.id(), b.unwrap().id());
    assert_eq!(user.status(), UserStatus::Incomplete);

    let mut stale_copy = repo.provision(&uid).await.unwrap();

    user.complete_profile(
        ProfileCompletion {
            nickname: " Apelido ".into(),
            current_semester: 3,
            institution_entry_semester: "2024.1".into(),
            name: Some("Nome declarado".into()),
            institutional_email: Some("institucional@example.test".into()),
            course_name: Some("Curso declarado".into()),
        },
        "pessoal@example.test",
        Utc::now(),
    )
    .unwrap();

    assert!(repo.update_profile(&user).await.unwrap());

    stale_copy
        .complete_profile(
            ProfileCompletion {
                nickname: "Apelido".into(),
                current_semester: 3,
                institution_entry_semester: "2024.1".into(),
                name: Some("Nome declarado".into()),
                institutional_email: Some("institucional@example.test".into()),
                course_name: Some("Curso declarado".into()),
            },
            "pessoal@example.test",
            Utc::now() + Duration::seconds(1),
        )
        .unwrap();

    assert!(repo.update_profile(&stale_copy).await.unwrap());

    let completed_at: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT profile_completed_at FROM app_user WHERE id=$1")
            .bind(user.id())
            .fetch_one(pool)
            .await
            .unwrap();

    assert_eq!(
        completed_at.timestamp_micros(),
        user.profile_completed_at().unwrap().timestamp_micros()
    );

    let restored = repo.provision(&uid).await.unwrap();

    assert_eq!(restored.status(), UserStatus::Active);
    assert_eq!(restored.nickname(), Some("Apelido"));
    assert_eq!(restored.name(), Some("Nome declarado"));
    assert_eq!(
        restored.institutional_email(),
        Some("institucional@example.test")
    );
    assert_eq!(restored.course_name(), Some("Curso declarado"));
    assert_eq!(restored.email(), Some("pessoal@example.test"));
    assert_eq!(restored.current_semester(), Some(3));
    assert_eq!(restored.institution_entry_semester(), Some("2024.1"));

    sqlx::query("UPDATE app_user SET status='inactive' WHERE id=$1")
        .bind(user.id())
        .execute(pool)
        .await
        .unwrap();

    assert!(!repo.update_profile(&user).await.unwrap());
    assert_eq!(
        repo.provision(&uid).await.unwrap().status(),
        UserStatus::Inactive
    );

    for invalid in [String::new(), "a".repeat(129)] {
        assert!(repo.provision(&invalid).await.is_err());
    }

    let id = Uuid::new_v4();

    assert!(
        sqlx::query("INSERT INTO app_user (id,firebase_uid,status) VALUES ($1,$2,'active')")
            .bind(id)
            .bind(id.to_string())
            .execute(pool)
            .await
            .is_err()
    );
}

/// Busca por tenant distingue ausência, mapeia enum PostgreSQL e preserva cadastro inativo.
async fn institution_lookup(pool: &PgPool, repo: &PgDatabase) {
    let v = fixture(pool, true).await;

    assert!(
        repo.find_by_microsoft_tenant_id(Uuid::new_v4())
            .await
            .unwrap()
            .is_none()
    );

    let institution = repo
        .find_by_microsoft_tenant_id(v.microsoft_tenant_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(institution.id, v.institution_id);
    assert_eq!(institution.name, "Instituição fictícia");
    assert_eq!(institution.acronym.as_deref(), Some("IF"));
    assert_eq!(
        institution.microsoft_student_group_id,
        v.microsoft_student_group_id
    );
    assert!(matches!(institution.status, CatalogStatus::Active));

    sqlx::query("UPDATE institution SET status='inactive', acronym=NULL WHERE id=$1")
        .bind(v.institution_id)
        .execute(pool)
        .await
        .unwrap();

    let institution = repo
        .find_by_microsoft_tenant_id(v.microsoft_tenant_id)
        .await
        .unwrap()
        .unwrap();

    assert!(matches!(institution.status, CatalogStatus::Inactive));
    assert!(institution.acronym.is_none());
}

/// Evidências vigentes de estudantes e não estudantes preservam todos os identificadores.
async fn evidence_roundtrip(pool: &PgPool, repo: &PgDatabase) {
    for student in [true, false] {
        let v = fixture(pool, student).await;

        assert!(repo.save(&v).await.unwrap());

        let stored = repo
            .find_valid_by_firebase_uid(&uid(pool, v.user_id).await)
            .await
            .unwrap()
            .expect("ambos os vínculos devem ser reutilizáveis");

        assert_eq!(stored.user_id, v.user_id);
        assert_eq!(stored.microsoft_object_id, v.microsoft_object_id);
        assert_eq!(stored.institution_id, v.institution_id);
        assert_eq!(stored.microsoft_tenant_id, v.microsoft_tenant_id);
        assert_eq!(
            stored.microsoft_student_group_id,
            v.microsoft_student_group_id
        );
        assert_eq!(stored.student_membership_confirmed, student);
        assert_eq!(stored.checked_at, v.checked_at);
        assert_eq!(stored.expires_at, v.expires_at);
        assert!(
            repo.find_valid_by_firebase_uid("uid-ausente")
                .await
                .unwrap()
                .is_none()
        );
    }
}

/// A consulta por UID não entrega a evidência de outra pessoa com o mesmo objeto Microsoft.
async fn uid_isolation(pool: &PgPool, repo: &PgDatabase) {
    let v = fixture(pool, true).await;

    repo.save(&v).await.unwrap();

    assert!(
        repo.find_valid_by_firebase_uid(&Uuid::new_v4().to_string())
            .await
            .unwrap()
            .is_none()
    );

    let mut other = v.clone();

    other.user_id = repo
        .provision(&Uuid::new_v4().to_string())
        .await
        .unwrap()
        .id();
    other.student_membership_confirmed = false;

    repo.save(&other).await.unwrap();

    assert!(
        repo.find_valid_by_firebase_uid(&uid(pool, v.user_id).await)
            .await
            .unwrap()
            .unwrap()
            .student_membership_confirmed
    );
    assert!(
        !repo
            .find_valid_by_firebase_uid(&uid(pool, other.user_id).await)
            .await
            .unwrap()
            .unwrap()
            .student_membership_confirmed
    );
}

/// Evidência expirada e alterações de tenant/grupo invalidam a reutilização.
async fn stale_evidence_is_filtered(pool: &PgPool, repo: &PgDatabase) {
    let mut expired = fixture(pool, true).await;

    expired.checked_at = Utc::now() - Duration::hours(2);
    expired.expires_at = Utc::now() - Duration::hours(1);

    repo.save(&expired).await.unwrap();

    assert!(
        repo.find_valid_by_firebase_uid(&uid(pool, expired.user_id).await)
            .await
            .unwrap()
            .is_none()
    );

    for column in ["microsoft_tenant_id", "microsoft_student_group_id"] {
        let v = fixture(pool, true).await;

        repo.save(&v).await.unwrap();
        sqlx::query(&format!("UPDATE institution SET {column}=$1 WHERE id=$2"))
            .bind(Uuid::new_v4())
            .bind(v.institution_id)
            .execute(pool)
            .await
            .unwrap();

        assert!(
            repo.find_valid_by_firebase_uid(&uid(pool, v.user_id).await)
                .await
                .unwrap()
                .is_none()
        );
    }
}

/// Nova resposta substitui todos os dados; escritas anteriores ou empatadas são descartadas.
async fn newer_evidence_wins(pool: &PgPool, repo: &PgDatabase) {
    let old = fixture(pool, true).await;

    repo.save(&old).await.unwrap();

    let mut latest = fixture(pool, false).await;

    latest.user_id = old.user_id;
    latest.checked_at = old.checked_at + Duration::seconds(1);
    latest.expires_at = latest.checked_at + Duration::hours(24);

    assert!(repo.save(&latest).await.unwrap());
    assert!(!repo.save(&old).await.unwrap());

    let mut tied = latest.clone();

    tied.student_membership_confirmed = true;

    assert!(!repo.save(&tied).await.unwrap());

    let stored = repo
        .find_valid_by_firebase_uid(&uid(pool, old.user_id).await)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(stored.institution_id, latest.institution_id);
    assert_eq!(stored.microsoft_object_id, latest.microsoft_object_id);
    assert!(!stored.student_membership_confirmed);
    assert_eq!(stored.checked_at, latest.checked_at);

    latest.checked_at += Duration::seconds(1);

    latest.student_membership_confirmed = true;

    assert!(repo.save(&latest).await.unwrap());
    assert!(
        repo.find_valid_by_firebase_uid(&uid(pool, latest.user_id).await)
            .await
            .unwrap()
            .unwrap()
            .student_membership_confirmed
    );
}

/// UPSERT concorrente preserva o maior checked_at, independentemente da ordem de chegada.
async fn concurrent_writes_preserve_newest(pool: &PgPool, repo: &PgDatabase) {
    let old = fixture(pool, true).await;
    let mut new = old.clone();

    new.checked_at += Duration::seconds(1);
    new.expires_at += Duration::seconds(1);

    new.student_membership_confirmed = false;

    let (a, b) = tokio::join!(repo.save(&old), repo.save(&new));

    a.unwrap();

    assert!(b.unwrap());

    let stored = repo
        .find_valid_by_firebase_uid(&uid(pool, old.user_id).await)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(stored.checked_at, new.checked_at);
    assert!(!stored.student_membership_confirmed);
}

/// FKs, UID e período inválidos são rejeitados, sem transformar falhas em sucesso.
async fn invalid_evidence_is_rejected(pool: &PgPool, repo: &PgDatabase) {
    let v = fixture(pool, true).await;

    for kind in 0..4 {
        let mut invalid = v.clone();

        match kind {
            0 => invalid.institution_id = Uuid::new_v4(),
            1 | 2 => invalid.user_id = Uuid::new_v4(),
            _ => invalid.expires_at = invalid.checked_at,
        }

        assert!(repo.save(&invalid).await.is_err());
    }

    assert!(
        repo.find_valid_by_firebase_uid(&uid(pool, v.user_id).await)
            .await
            .unwrap()
            .is_none()
    );
}

/// Unicidade do tenant, campos não vazios e cascata da instituição são aplicados pelo banco.
async fn institution_constraints_and_cascade(pool: &PgPool, repo: &PgDatabase) {
    let v = fixture(pool, true).await;

    repo.save(&v).await.unwrap();

    let duplicate=sqlx::query("INSERT INTO institution (name,microsoft_tenant_id,microsoft_student_group_id) VALUES ('Outra',$1,$2)")
        .bind(v.microsoft_tenant_id).bind(Uuid::new_v4()).execute(pool).await.unwrap_err();

    assert_eq!(
        duplicate.as_database_error().unwrap().code().as_deref(),
        Some("23505")
    );

    for column in ["name", "acronym"] {
        let error = sqlx::query(&format!(
            "UPDATE institution SET {column}='   ' WHERE id=$1"
        ))
        .bind(v.institution_id)
        .execute(pool)
        .await
        .unwrap_err();

        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("23514")
        );
    }

    sqlx::query("DELETE FROM institution WHERE id=$1")
        .bind(v.institution_id)
        .execute(pool)
        .await
        .unwrap();

    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM institutional_verification WHERE user_id=$1")
            .bind(v.user_id)
            .fetch_one(pool)
            .await
            .unwrap();

    assert_eq!(count, 0);
}

/// Falhas de aquisição de conexão preservam os erros técnicos dos contratos.
async fn closed_pool_returns_repository_errors() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://test:test@127.0.0.1:1/test")
        .unwrap();

    pool.close().await;

    let repo = PgDatabase::new(pool);

    assert!(matches!(
        repo.find_by_microsoft_tenant_id(Uuid::new_v4()).await,
        Err(InstitutionRepositoryError::InternalServer)
    ));
    assert!(repo.find_valid_by_firebase_uid("uid").await.is_err());
    assert!(repo.get_by_firebase_uid("uid").await.is_err());
    assert!(repo.provision("uid").await.is_err());

    let now = Utc::now();
    let v = InstitutionalVerification {
        user_id: Uuid::new_v4(),
        profile_name: None,
        profile_email: None,
        course_name: None,
        microsoft_object_id: Uuid::new_v4(),
        institution_id: Uuid::new_v4(),
        microsoft_tenant_id: Uuid::new_v4(),
        microsoft_student_group_id: Uuid::new_v4(),
        student_membership_confirmed: false,
        checked_at: now,
        expires_at: now + Duration::hours(1),
    };

    assert!(repo.save(&v).await.is_err());
}

/// Executa nove contratos em um único container; o descarte remove o banco mesmo em panic.
#[tokio::test]
async fn postgresql_identity_contracts() {
    let database = TestDatabase::new().await;
    let pool = database.pool.clone();
    let repo = PgDatabase::new(pool.clone());

    legacy_evidence_migration(&pool).await;
    user_provisioning(&pool, &repo).await;
    institution_lookup(&pool, &repo).await;
    evidence_roundtrip(&pool, &repo).await;
    uid_isolation(&pool, &repo).await;
    stale_evidence_is_filtered(&pool, &repo).await;
    newer_evidence_wins(&pool, &repo).await;
    concurrent_writes_preserve_newest(&pool, &repo).await;
    invalid_evidence_is_rejected(&pool, &repo).await;
    institution_constraints_and_cascade(&pool, &repo).await;
    closed_pool_returns_repository_errors().await;
    database.close().await;
}
