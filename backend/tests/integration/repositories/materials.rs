//! Integridade SQL, isolamento e concorrência das publicações.

use backend::{
    infrastructure::database::PgDatabase,
    modules::materials::application::services::{
        self, MaterialError, MaterialInput, MaterialUpload,
    },
};
use uuid::Uuid;

use crate::{materials_fixtures::Fixture, support::TestDatabase};

#[tokio::test]
async fn materials_preserve_constraints_and_one_concurrent_publication() {
    let database = TestDatabase::new().await;
    let fixture = Fixture::new(
        &database.pool,
        ["Computação", "Matemática"],
        ["Cálculo I", "Álgebra"],
    )
    .await;
    let db = PgDatabase::new(database.pool.clone());
    let ca = fixture.ca[0];
    fixture.grant(&database.pool, ca).await;
    let key = Uuid::new_v4();
    let publish = || {
        services::publish(
            &db,
            None,
            fixture.author,
            ca,
            key,
            MaterialInput {
                title: "Resumo".into(),
                description: String::new(),
                discipline_id: fixture.discipline[0],
            },
            MaterialUpload::Link("https://exemplo.edu.br/resumo".into()),
        )
    };

    let (first, second) = tokio::join!(publish(), publish());
    let first = first.unwrap();
    let second = second.unwrap();
    let id = first.value.material.id();
    assert_eq!(id, second.value.material.id());
    assert_ne!(first.created, second.created);

    for (statement, expected) in [
        (
            "UPDATE study_material SET discipline_id = $2 WHERE id = $1",
            "23503",
        ),
        (
            "UPDATE study_material SET author_id = $2 WHERE id = $1",
            "23503",
        ),
        (
            "UPDATE study_material SET file_name = $2::uuid::text WHERE id = $1",
            "23514",
        ),
    ] {
        let error = sqlx::query(statement)
            .bind(id)
            .bind(fixture.discipline[1])
            .execute(&database.pool)
            .await
            .unwrap_err();
        assert_eq!(error.as_database_error().unwrap().code().unwrap(), expected);
    }
    sqlx::query("UPDATE study_material SET status = 'archived' WHERE id = $1")
        .bind(id)
        .execute(&database.pool)
        .await
        .unwrap();

    let student = fixture.students[0];
    assert!(matches!(
        services::details(&db, student, ca, id).await,
        Err(MaterialError::NotFound)
    ));
    let page = services::list(&db, student, ca, Some(fixture.discipline[0]), 30, 0)
        .await
        .unwrap();
    assert!(page.items.is_empty());
    database.close().await;
}
