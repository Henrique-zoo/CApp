//! Estado e integração de armazenamento controlados por cenário de materiais.

#[path = "../../fixtures/materials/auth.rs"]
mod auth;
#[path = "../../fixtures/materials/mod.rs"]
pub(crate) mod fixtures;

use async_trait::async_trait;
use axum::{
    body::{Body, Bytes, to_bytes},
    http::{HeaderMap, StatusCode},
};
use backend::{
    infrastructure::database::PgDatabase,
    modules::materials::application::{
        ports::{MaterialAccount, MaterialFile, MaterialStorage, Publication},
        services::{self, MaterialError, MaterialInput, MaterialUpload},
    },
};
use serde_json::Value;
use sha2::Digest;
use sqlx::PgPool;
use std::{
    io,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tower::ServiceExt;
use uuid::Uuid;

pub(crate) struct MaterialsScenario {
    pub(crate) fixture: fixtures::Fixture,
    pub(crate) ca_names: [String; 2],
    pub(crate) discipline_names: [String; 2],
    pub(crate) upload: Option<(String, Vec<u8>)>,
    pub(crate) storage: ScenarioStorage,
    pub(crate) request_id: Uuid,
    pub(crate) result: Option<Result<Publication, MaterialError>>,
    pub(crate) found: Option<(usize, Uuid)>,
}

#[derive(Default)]
pub(crate) struct ScenarioStorage {
    pub(crate) fail_next: AtomicBool,
    pdfs: Mutex<Vec<StoredPdf>>,
}

struct StoredPdf {
    ca_id: Uuid,
    sha: [u8; 32],
    bytes: Vec<u8>,
}

#[async_trait]
impl MaterialStorage for ScenarioStorage {
    async fn put(&self, ca_id: Uuid, file: &MaterialFile, bytes: &[u8]) -> io::Result<()> {
        if self.fail_next.swap(false, Ordering::SeqCst) {
            return Err(io::Error::other("falha controlada de armazenamento"));
        }
        self.pdfs.lock().unwrap().push(StoredPdf {
            ca_id,
            sha: file.sha256,
            bytes: bytes.to_vec(),
        });
        Ok(())
    }

    async fn read(&self, ca_id: Uuid, file: &MaterialFile) -> io::Result<Vec<u8>> {
        self.pdfs
            .lock()
            .unwrap()
            .iter()
            .find(|stored| stored.ca_id == ca_id && stored.sha == file.sha256)
            .map(|stored| stored.bytes.clone())
            .ok_or_else(|| io::ErrorKind::NotFound.into())
    }
}

pub(crate) const MATERIAL_LINK: &str = "https://exemplo.edu.br/referencias";
pub(crate) const MATERIAL_PDF: &str = "referencias.pdf";

pub(crate) struct MaterialReadScenario {
    pub(crate) fixture: fixtures::Fixture,
    pool: PgPool,
    pub(crate) http: auth::LocalMaterialAuth,
    pub(crate) storage: Arc<ScenarioStorage>,
    ca_names: [String; 2],
    pub(crate) active: usize,
    pub(crate) selected_discipline: Option<Uuid>,
    seeds: Vec<(usize, String, Uuid)>,
    pub(crate) response: Option<MaterialResponse>,
}

pub(crate) struct MaterialResponse {
    pub(crate) status: StatusCode,
    pub(crate) headers: HeaderMap,
    pub(crate) body: Bytes,
}

impl MaterialResponse {
    pub(crate) fn json(&self, status: StatusCode) -> Value {
        assert_eq!(self.status, status, "{:?}", self.body);
        serde_json::from_slice(&self.body).expect("JSON de materiais")
    }
}

impl MaterialReadScenario {
    pub(crate) async fn new(pool: PgPool, active_name: String) -> Self {
        let ca_names = [active_name, "CA de Matemática".to_owned()];
        let fixture = fixtures::Fixture::new(
            &pool,
            ca_names.each_ref().map(String::as_str),
            ["Cálculo I", "Cálculo I"],
        )
        .await;
        for ca in fixture.ca {
            fixture.grant(&pool, ca).await;
        }
        let storage = Arc::new(ScenarioStorage::default());
        let project = std::env::var("FIREBASE_PROJECT_ID").expect("projeto Firebase do runner");
        let http = auth::LocalMaterialAuth::new(&pool, project, Some(storage.clone())).await;
        Self {
            fixture,
            pool,
            http,
            storage,
            ca_names,
            active: 0,
            selected_discipline: None,
            seeds: Vec::new(),
            response: None,
        }
    }

    pub(crate) fn ca(&self, name: &str) -> usize {
        self.ca_names
            .iter()
            .position(|item| item == name)
            .expect("CA da fixture")
    }

    pub(crate) fn root(&self) -> String {
        format!(
            "/academic-centers/{}/materials",
            self.fixture.ca[self.active]
        )
    }

    pub(crate) async fn ensure_discipline(&self, ca: usize, name: &str) -> Uuid {
        match self.discipline(ca, name).await {
            Some(id) => id,
            None => sqlx::query_scalar(
                "INSERT INTO academic_center_discipline (ca_id, name)
                     VALUES ($1, $2) RETURNING id",
            )
            .bind(self.fixture.ca[ca])
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .unwrap(),
        }
    }

    pub(crate) async fn discipline(&self, ca: usize, name: &str) -> Option<Uuid> {
        sqlx::query_scalar(
            "SELECT id FROM academic_center_discipline WHERE ca_id = $1 AND name = $2",
        )
        .bind(self.fixture.ca[ca])
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .unwrap()
    }

    pub(crate) async fn clear_catalog(&self) {
        sqlx::query("DELETE FROM academic_center_discipline WHERE ca_id = $1")
            .bind(self.fixture.ca[self.active])
            .execute(&self.pool)
            .await
            .unwrap();
    }

    pub(crate) async fn seed(&mut self, ca: usize, title: String, pdf: bool) -> Uuid {
        self.seed_in(ca, self.fixture.discipline[ca], title, pdf)
            .await
    }

    pub(crate) async fn seed_in(
        &mut self,
        ca: usize,
        discipline_id: Uuid,
        title: String,
        pdf: bool,
    ) -> Uuid {
        let upload = if pdf {
            MaterialUpload::Pdf(MATERIAL_PDF.to_owned(), fixtures::pdf(None))
        } else {
            MaterialUpload::Link(MATERIAL_LINK.to_owned())
        };
        let result = services::publish(
            &PgDatabase::new(self.pool.clone()),
            Some(self.storage.as_ref()),
            self.fixture.author,
            self.fixture.ca[ca],
            Uuid::new_v4(),
            MaterialInput {
                title: title.clone(),
                description: String::new(),
                discipline_id,
            },
            upload,
        )
        .await
        .expect("publicação da fixture");
        let id = result.value.material.id();
        self.seeds.push((ca, title, id));
        id
    }

    pub(crate) fn id(&self, title: &str) -> Uuid {
        self.seeds
            .iter()
            .find(|(ca, name, _)| *ca == self.active && name == title)
            .map(|(_, _, id)| *id)
            .expect("material da fixture no CA ativo")
    }

    pub(crate) async fn describe_last(&self, description: String) {
        let id = self.seeds.last().expect("material para descrever").2;
        sqlx::query("UPDATE study_material SET description = $1 WHERE id = $2")
            .bind(description)
            .bind(id)
            .execute(&self.pool)
            .await
            .unwrap();
    }

    pub(crate) async fn use_pdf_for_last(&self) {
        let (ca, _, id) = self.seeds.last().expect("material para disponibilizar");
        let bytes = fixtures::pdf(None);
        let file = MaterialFile {
            name: MATERIAL_PDF.to_owned(),
            size: bytes.len() as i32,
            sha256: sha2::Sha256::digest(&bytes).into(),
        };
        self.storage
            .put(self.fixture.ca[*ca], &file, &bytes)
            .await
            .unwrap();
        sqlx::query(
            "UPDATE study_material SET kind = 'file', external_url = NULL, file_name = $1,
                file_size = $2, file_sha256 = $3 WHERE id = $4",
        )
        .bind(&file.name)
        .bind(file.size)
        .bind(file.sha256.as_slice())
        .bind(id)
        .execute(&self.pool)
        .await
        .unwrap();
    }

    pub(crate) async fn fail_list_dependency(&self) {
        // Afeta só a consulta do material. A identidade e a autorização continuam disponíveis.
        sqlx::query("ALTER TABLE study_material RENAME TO unavailable_material")
            .execute(&self.pool)
            .await
            .unwrap();
    }

    pub(crate) async fn archive(&self, id: Uuid) {
        let update = sqlx::query("UPDATE study_material SET status = 'archived' WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .unwrap();
        assert_eq!(update.rows_affected(), 1);
    }

    pub(crate) async fn set_publisher(&self, ca: Option<usize>) {
        sqlx::query("DELETE FROM academic_center_material_publisher WHERE account_id = $1")
            .bind(self.fixture.author.id)
            .execute(&self.pool)
            .await
            .unwrap();
        if let Some(ca) = ca {
            self.fixture.grant(&self.pool, self.fixture.ca[ca]).await;
        }
    }

    pub(crate) async fn get(&mut self, path: String) {
        self.request("GET", path, self.fixture.students[0]).await;
    }

    async fn request(&mut self, method: &str, path: String, account: MaterialAccount) {
        let request = self
            .http
            .request(method, &path, account)
            .body(Body::empty())
            .unwrap();
        let response = self.http.app.clone().oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        self.response = Some(MaterialResponse {
            status: parts.status,
            headers: parts.headers,
            body: to_bytes(body, services::MAX_PDF_BYTES).await.unwrap(),
        });
    }

    pub(crate) fn response(&self) -> &MaterialResponse {
        self.response
            .as_ref()
            .expect("consulta executada no router")
    }

    pub(crate) fn titles(&self) -> Vec<String> {
        self.response().json(StatusCode::OK)["items"]
            .as_array()
            .expect("lista de materiais")
            .iter()
            .map(|item| item["title"].as_str().expect("título").to_owned())
            .collect()
    }

    pub(crate) fn assert_selected_discipline(&self) {
        let expected = self
            .selected_discipline
            .expect("disciplina selecionada")
            .to_string();
        let response = self.response().json(StatusCode::OK);
        for item in response["items"]
            .as_array()
            .expect("lista filtrada por disciplina")
        {
            assert_eq!(item["disciplineId"], expected);
            assert_eq!(item["caId"], self.fixture.ca[self.active].to_string());
        }
    }
}

impl MaterialsScenario {
    pub(crate) fn ca(&self, name: &str) -> usize {
        self.ca_names
            .iter()
            .position(|stored| stored == name)
            .expect("CA do cenário")
    }

    pub(crate) fn discipline(&self, name: &str) -> Uuid {
        let index = self
            .discipline_names
            .iter()
            .position(|stored| stored == name)
            .expect("disciplina do cenário");
        self.fixture.discipline[index]
    }

    pub(crate) async fn publish(
        &mut self,
        db: &PgDatabase,
        name: String,
        discipline: Uuid,
        upload: MaterialUpload,
    ) {
        self.result = Some(
            services::publish(
                db,
                Some(&self.storage),
                self.fixture.author,
                self.fixture.ca[0],
                self.request_id,
                MaterialInput {
                    title: name,
                    description: String::new(),
                    discipline_id: discipline,
                },
                upload,
            )
            .await,
        );
    }

    pub(crate) fn pdf(&self, name: &str) -> MaterialUpload {
        let (stored, bytes) = self.upload.as_ref().expect("arquivo preparado");
        assert_eq!(stored, name);
        MaterialUpload::Pdf(stored.clone(), bytes.clone())
    }

    pub(crate) fn rejection(&self) -> &MaterialError {
        match self.result.as_ref().expect("tentativa executada") {
            Ok(_) => panic!("a publicação deveria ter sido recusada"),
            Err(error) => error,
        }
    }
}
