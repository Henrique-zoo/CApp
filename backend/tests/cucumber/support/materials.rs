//! Estado e integração de armazenamento controlados por cenário de materiais.

#[path = "../../fixtures/materials/mod.rs"]
pub(crate) mod fixtures;

use async_trait::async_trait;
use backend::{
    infrastructure::database::PgDatabase,
    modules::materials::application::{
        ports::{MaterialFile, MaterialStorage, Publication},
        services::{self, MaterialError, MaterialInput, MaterialUpload},
    },
};
use std::{
    io,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
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
    pdf: Mutex<Option<StoredPdf>>,
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
        *self.pdf.lock().unwrap() = Some(StoredPdf {
            ca_id,
            sha: file.sha256,
            bytes: bytes.to_vec(),
        });
        Ok(())
    }

    async fn read(&self, ca_id: Uuid, file: &MaterialFile) -> io::Result<Vec<u8>> {
        self.pdf
            .lock()
            .unwrap()
            .as_ref()
            .filter(|stored| stored.ca_id == ca_id && stored.sha == file.sha256)
            .map(|stored| stored.bytes.clone())
            .ok_or_else(|| io::ErrorKind::NotFound.into())
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
