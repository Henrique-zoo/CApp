//! Percursos de persistência que atravessam PRs com modelos originalmente divergentes.

use std::sync::Arc;

use backend::modules::{
    events::{
        application::{EventApplicationError, EventRepository},
        domain::{Event, EventRegistrationPolicy},
        infrastructure::PostgresEventRepository,
    },
    news::{
        application::services::NewsApplicationService,
        infrastructure::{PgNewsRepository, StaticRoleNewsAuthorization},
    },
};
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::support::TestDatabase;

#[tokio::test]
async fn event_registration_consumes_and_releases_the_only_place() {
    let db = TestDatabase::new().await;
    sqlx::raw_sql(include_str!("../../cucumber/fixtures/api_bdd.sql"))
        .execute(&db.pool)
        .await
        .unwrap();
    let ca = Uuid::parse_str("018f0000-0000-7000-8000-000000000001").unwrap();
    let first = Uuid::parse_str("018f0000-0000-7000-8000-000000000011").unwrap();
    let second = Uuid::parse_str("018f0000-0000-7000-8000-000000000013").unwrap();
    let start = Utc::now() + Duration::hours(1);
    let event = Event::create_published(
        ca,
        first,
        "Atividade do CA",
        "Descrição da atividade",
        "Auditório",
        start,
        start + Duration::hours(1),
        Some(1),
        EventRegistrationPolicy::unrestricted(),
    )
    .unwrap();
    let repo = PostgresEventRepository::new(db.pool.clone());
    let event = repo.create(&event).await.unwrap();

    let registration = repo
        .register_participant_atomically(event.id(), first, Some(ca))
        .await
        .unwrap();
    assert!(registration.is_active());
    assert!(matches!(
        repo.register_participant_atomically(event.id(), second, Some(ca))
            .await,
        Err(EventApplicationError::EventFull { .. })
    ));
    let cancelled = repo
        .cancel_registration_atomically(event.id(), first, Some(ca))
        .await
        .unwrap();
    assert!(!cancelled.is_active());
    assert!(
        repo.register_participant_atomically(event.id(), second, Some(ca))
            .await
            .unwrap()
            .is_active()
    );
    assert_eq!(
        repo.list_registrations_by_event(event.id())
            .await
            .unwrap()
            .len(),
        2
    );
    db.close().await;
}

#[tokio::test]
async fn edited_news_leaves_the_feed_until_approved_again() {
    let db = TestDatabase::new().await;
    let ca = Uuid::new_v4();
    let actor = Uuid::new_v4();
    let mut authorization = StaticRoleNewsAuthorization::new();
    authorization.grant_manage(actor, ca);
    let service = NewsApplicationService::new(
        Arc::new(PgNewsRepository::new(db.pool.clone())),
        Arc::new(authorization),
    );
    let article = service
        .create(actor, ca, "Primeira versão".into(), "Conteúdo".into(), true)
        .await
        .unwrap();
    assert_eq!(
        service.list_published_news_by_ca(ca).await.unwrap().len(),
        1
    );

    service
        .edit(
            article.id,
            actor,
            "Nova versão".into(),
            "Conteúdo revisado".into(),
        )
        .await
        .unwrap();
    assert!(
        service
            .list_published_news_by_ca(ca)
            .await
            .unwrap()
            .is_empty()
    );

    service
        .publish_news(
            backend::modules::news::application::commands::PublishNewsArticleCommand {
                news_id: article.id,
                actor_id: actor,
                published_at: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(
        service.list_published_news_by_ca(ca).await.unwrap()[0].title,
        "Nova versão"
    );
    db.close().await;
}
