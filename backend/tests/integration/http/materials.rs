//! Contratos de materiais pelo router real, PostgreSQL e armazenamento em disco.

use std::sync::Arc;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use backend::{
    infrastructure::storage::DirectoryMaterialStorage,
    modules::materials::application::{ports::MaterialAccount, services::MAX_PDF_BYTES},
};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

use crate::{materials_fixtures as fixtures, support::TestDatabase};

#[path = "../../fixtures/materials/auth.rs"]
mod auth;
use auth::LocalMaterialAuth;

struct HttpFixture {
    auth: LocalMaterialAuth,
    root: std::path::PathBuf,
}

impl HttpFixture {
    async fn new(db: &TestDatabase) -> Self {
        let root = std::env::temp_dir().join(format!("capp-materials-{}", Uuid::new_v4()));
        let storage = DirectoryMaterialStorage::new(root.clone()).await.unwrap();
        let auth =
            LocalMaterialAuth::new(&db.pool, "materials-test".into(), Some(Arc::new(storage)))
                .await;
        Self { auth, root }
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        account: MaterialAccount,
    ) -> axum::http::request::Builder {
        self.auth.request(method, path, account)
    }

    async fn json(&self, request: Request<Body>, status: StatusCode) -> Value {
        let response = self.auth.app.clone().oneshot(request).await.unwrap();
        let actual = response.status();
        let body: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
        assert_eq!(actual, status, "{body}");
        body
    }

    async fn link(
        &self,
        path: &str,
        author: MaterialAccount,
        key: Uuid,
        input: Value,
        status: StatusCode,
    ) -> Value {
        self.json(
            self.request("POST", path, author)
                .header("idempotency-key", key.to_string())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(input.to_string()))
                .unwrap(),
            status,
        )
        .await
    }

    fn pdf(
        &self,
        path: &str,
        author: MaterialAccount,
        key: Uuid,
        discipline: Uuid,
        pdf: &[u8],
    ) -> Request<Body> {
        let metadata = json!({"title":"Resumo em PDF","description":"Para a avaliação","disciplineId":discipline});
        let boundary = "capp-test-material-boundary";
        let mut body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"metadata\"\r\n\r\n{metadata}\r\n\
             --{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"resumo.pdf\"\r\n\
             Content-Type: application/pdf\r\n\r\n"
        ).into_bytes();
        body.extend_from_slice(pdf);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        self.request("POST", path, author)
            .header("idempotency-key", key.to_string())
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap()
    }
}

impl Drop for HttpFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[tokio::test]
async fn materials_authorize_publish_and_read_by_ca() {
    let db = TestDatabase::new().await;
    let data = fixtures::Fixture::new(
        &db.pool,
        ["Computação", "Matemática"],
        ["Cálculo I", "Álgebra"],
    )
    .await;
    data.grant(&db.pool, data.ca[0]).await;
    let api = HttpFixture::new(&db).await;
    let path = format!("/academic-centers/{}/materials", data.ca[0]);
    let other = format!("/academic-centers/{}/materials", data.ca[1]);
    let input = json!({"title": "Referências", "description": "", "disciplineId": data.discipline[0],
        "content":{"type":"external_link","url":"https://exemplo.edu.br/calculo"}});

    let request = Request::builder().uri(&path).body(Body::empty()).unwrap();
    assert_eq!(
        api.json(request, StatusCode::FORBIDDEN).await["error"]["code"],
        "app_check_required"
    );
    let request = Request::builder()
        .uri(&path)
        .header("x-firebase-appcheck", &api.auth.attestation)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        api.json(request, StatusCode::UNAUTHORIZED).await["error"]["code"],
        "unauthorized"
    );

    let contexts = api
        .json(
            api.request("GET", "/materials/contexts", data.author)
                .body(Body::empty())
                .unwrap(),
            StatusCode::OK,
        )
        .await;
    assert_eq!(contexts[0]["canPublish"], true);
    assert_eq!(contexts[1]["canPublish"], false);

    let key = Uuid::new_v4();
    api.link(
        &path,
        data.students[0],
        key,
        input.clone(),
        StatusCode::FORBIDDEN,
    )
    .await;
    api.link(
        &other,
        data.author,
        key,
        input.clone(),
        StatusCode::FORBIDDEN,
    )
    .await;
    let mut invalid = input.clone();
    invalid["disciplineId"] = json!(data.discipline[1]);
    assert_eq!(
        api.link(&path, data.author, key, invalid, StatusCode::BAD_REQUEST)
            .await["error"]["code"],
        "invalid_material_discipline"
    );
    let mut forged = input.clone();
    forged["authorId"] = json!(data.students[0].id);
    api.link(&path, data.author, key, forged, StatusCode::BAD_REQUEST)
        .await;

    let material = api
        .link(&path, data.author, key, input.clone(), StatusCode::CREATED)
        .await;
    assert_eq!(
        api.link(&path, data.author, key, input.clone(), StatusCode::OK)
            .await,
        material
    );
    let mut changed = input.clone();
    changed["title"] = json!("Outro título");
    api.link(&path, data.author, key, changed, StatusCode::CONFLICT)
        .await;
    let id = material["id"].as_str().unwrap();
    let author: Uuid = sqlx::query_scalar("SELECT author_id FROM study_material WHERE id = $1")
        .bind(Uuid::parse_str(id).unwrap())
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(author, data.author.id);

    let list = api
        .json(
            api.request(
                "GET",
                &format!("{path}?disciplineId={}", data.discipline[0]),
                data.students[0],
            )
            .body(Body::empty())
            .unwrap(),
            StatusCode::OK,
        )
        .await;
    assert_eq!(list["items"], json!([material]));
    for (base, expected) in [(&path, StatusCode::OK), (&other, StatusCode::NOT_FOUND)] {
        let request = api
            .request("GET", &format!("{base}/{id}"), data.students[1])
            .body(Body::empty())
            .unwrap();
        api.json(request, expected).await;
    }

    sqlx::query(
        "DELETE FROM academic_center_material_publisher WHERE ca_id = $1 AND account_id = $2",
    )
    .bind(data.ca[0])
    .bind(data.author.id)
    .execute(&db.pool)
    .await
    .unwrap();
    api.link(
        &path,
        data.author,
        Uuid::new_v4(),
        input,
        StatusCode::FORBIDDEN,
    )
    .await;
    db.close().await;
}

#[tokio::test]
async fn materials_upload_and_download_pdf_at_exact_limit() {
    let db = TestDatabase::new().await;
    let data = fixtures::Fixture::new(
        &db.pool,
        ["Computação", "Matemática"],
        ["Cálculo I", "Álgebra"],
    )
    .await;
    data.grant(&db.pool, data.ca[0]).await;
    let api = HttpFixture::new(&db).await;
    let path = format!("/academic-centers/{}/materials", data.ca[0]);
    let key = Uuid::new_v4();
    let too_large = fixtures::pdf(Some(MAX_PDF_BYTES + 1));
    let rejection = api
        .json(
            api.pdf(&path, data.author, key, data.discipline[0], &too_large),
            StatusCode::PAYLOAD_TOO_LARGE,
        )
        .await;
    assert_eq!(rejection["error"]["code"], "material_pdf_too_large");
    let pdf = fixtures::pdf(Some(MAX_PDF_BYTES));
    let published = api
        .json(
            api.pdf(&path, data.author, key, data.discipline[0], &pdf),
            StatusCode::CREATED,
        )
        .await;
    assert_eq!(
        published["content"],
        json!({"type":"file","fileName":"resumo.pdf","sizeBytes":MAX_PDF_BYTES})
    );
    let recovered = api
        .json(
            api.pdf(&path, data.author, key, data.discipline[0], &pdf),
            StatusCode::OK,
        )
        .await;
    assert_eq!(recovered, published);

    let id = published["id"].as_str().unwrap();
    let request = api
        .request("GET", &format!("{path}/{id}/file"), data.students[0])
        .body(Body::empty())
        .unwrap();
    let response = api.auth.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/pdf");
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "private, no-store"
    );
    assert_eq!(
        to_bytes(response.into_body(), MAX_PDF_BYTES)
            .await
            .unwrap()
            .as_ref(),
        pdf
    );

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM study_material")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
    db.close().await;
}
