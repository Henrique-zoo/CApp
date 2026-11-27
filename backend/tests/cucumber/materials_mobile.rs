//! Serve o router real e as fixtures dos cenários mobile somente em loopback.

#[allow(dead_code)]
#[path = "support/database.rs"]
mod database;
#[allow(dead_code)]
#[path = "support/materials.rs"]
mod materials;

use std::{net::Ipv4Addr, sync::Arc};

use axum::{
    Json, Router,
    body::{Body, Bytes},
    extract::State,
    http::{Request, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use database::{DatabaseServer, SuiteDatabase, TestDatabase};
use materials::{MaterialReadScenario, fixtures};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tower::ServiceExt;

struct MobileFixtures {
    suite: Arc<SuiteDatabase>,
    current: Mutex<Option<(TestDatabase, MaterialReadScenario)>>,
}

type SharedFixtures = Arc<MobileFixtures>;

#[derive(Deserialize)]
struct Reset {
    center: String,
    publication: bool,
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum FixtureChange {
    Seed {
        center: String,
        title: String,
        discipline: Option<String>,
        pdf: bool,
        archived: bool,
    },
    Describe {
        description: String,
    },
    ConvertLastToPdf,
    AddDiscipline {
        center: String,
        name: String,
    },
    EmptyCatalog,
    SetPublisher {
        center: Option<String>,
    },
    FailList,
    FailUpload,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = std::env::var("MATERIALS_BDD_PORT").map_or(Ok(38475), |port| port.parse())?;
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await?;
    let database = DatabaseServer::start().await;
    let state = Arc::new(MobileFixtures {
        suite: database.suite.clone(),
        current: Mutex::new(None),
    });
    let app = Router::new()
        .route("/__fixture/ready", get(|| async { StatusCode::NO_CONTENT }))
        .route("/__fixture/reset", post(reset))
        .route("/__fixture/change", post(change))
        .route("/__fixture/pdf", get(pdf))
        .fallback(serve_materials)
        .with_state(state.clone());

    println!("Fixtures de materiais em http://{}", listener.local_addr()?);
    let result = axum::serve(listener, app)
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
        .await;
    if let Some((db, scenario)) = state.current.lock().await.take() {
        drop(scenario);
        state.suite.release_scenario(db).await;
    }
    database.shutdown().await;
    result?;
    Ok(())
}

async fn reset(State(state): State<SharedFixtures>, Json(reset): Json<Reset>) -> Json<Value> {
    let mut current = state.current.lock().await;
    if let Some((db, scenario)) = current.take() {
        drop(scenario);
        state.suite.release_scenario(db).await;
    }
    let (db, unused_router) = state.suite.prepare_scenario(true).await;
    drop(unused_router);
    let scene = MaterialReadScenario::new(db.pool.clone(), reset.center).await;
    if reset.publication {
        scene.set_publisher(None).await;
        scene.ensure_discipline(1, "Direito Constitucional").await;
    }
    let tokens = json!({
        "reader": scene.http.token(scene.fixture.students[0]),
        "author": scene.http.token(scene.fixture.author),
        "appCheck": scene.http.attestation,
    });
    *current = Some((db, scene));
    Json(tokens)
}

async fn change(
    State(state): State<SharedFixtures>,
    Json(change): Json<FixtureChange>,
) -> Response {
    let mut current = state.current.lock().await;
    let Some((_, scene)) = current.as_mut() else {
        return StatusCode::CONFLICT.into_response();
    };
    match change {
        FixtureChange::Seed {
            center,
            title,
            discipline,
            pdf,
            archived,
        } => {
            let center = scene.ca(&center);
            let id = match discipline {
                Some(discipline) => {
                    let discipline = scene.ensure_discipline(center, &discipline).await;
                    scene.seed_in(center, discipline, title, pdf).await
                }
                None => scene.seed(center, title, pdf).await,
            };
            if archived {
                scene.archive(id).await;
            }
            Json(json!({ "id": id })).into_response()
        }
        FixtureChange::Describe { description } => {
            scene.describe_last(description).await;
            StatusCode::NO_CONTENT.into_response()
        }
        FixtureChange::ConvertLastToPdf => {
            scene.use_pdf_for_last().await;
            StatusCode::NO_CONTENT.into_response()
        }
        FixtureChange::AddDiscipline { center, name } => {
            scene.ensure_discipline(scene.ca(&center), &name).await;
            StatusCode::NO_CONTENT.into_response()
        }
        FixtureChange::EmptyCatalog => {
            scene.clear_catalog().await;
            StatusCode::NO_CONTENT.into_response()
        }
        FixtureChange::SetPublisher { center } => {
            scene
                .set_publisher(center.as_deref().map(|name| scene.ca(name)))
                .await;
            StatusCode::NO_CONTENT.into_response()
        }
        FixtureChange::FailList => {
            scene.fail_list_dependency().await;
            StatusCode::NO_CONTENT.into_response()
        }
        FixtureChange::FailUpload => {
            scene
                .storage
                .fail_next
                .store(true, std::sync::atomic::Ordering::SeqCst);
            StatusCode::NO_CONTENT.into_response()
        }
    }
}

#[derive(Deserialize)]
struct PdfSize {
    size: Option<usize>,
}

async fn pdf(axum::extract::Query(input): axum::extract::Query<PdfSize>) -> Response {
    let base = fixtures::pdf(None);
    if input.size.is_some_and(|size| {
        !(base.len()..=backend::modules::materials::application::services::MAX_PDF_BYTES + 1)
            .contains(&size)
    }) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    (
        [(header::CONTENT_TYPE, "application/pdf")],
        Bytes::from(fixtures::pdf(input.size)),
    )
        .into_response()
}

async fn serve_materials(State(state): State<SharedFixtures>, request: Request<Body>) -> Response {
    let router = state
        .current
        .lock()
        .await
        .as_ref()
        .map(|(_, scene)| scene.http.app.clone());
    match router {
        Some(router) => router.oneshot(request).await.unwrap(),
        None => StatusCode::CONFLICT.into_response(),
    }
}
