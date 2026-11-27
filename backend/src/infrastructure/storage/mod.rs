//! Armazenamento privado de PDFs em um diretório de dados gerenciado pela instalação.

use std::{
    fmt::Write as _,
    io,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use sha2::{Digest, Sha256};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};
use uuid::Uuid;

use crate::modules::materials::application::ports::{MaterialFile, MaterialStorage};

/// Armazena arquivos por CA e hash, sem usar nomes fornecidos pelo usuário.
pub struct DirectoryMaterialStorage {
    root: PathBuf,
}

impl DirectoryMaterialStorage {
    /// Cria e verifica o diretório privado configurado pela instalação.
    pub async fn new(root: PathBuf) -> io::Result<Self> {
        create_private_directory(&root).await?;
        let probe = root.join(format!(".probe-{}", Uuid::new_v4()));
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&probe)
            .await?;
        fs::remove_file(probe).await?;
        Ok(Self { root })
    }

    fn path(&self, ca_id: Uuid, file: &MaterialFile) -> PathBuf {
        let mut name = String::with_capacity(64);
        for byte in file.sha256 {
            write!(name, "{byte:02x}").expect("escrever em String é infalível");
        }
        self.root.join(ca_id.to_string()).join(name)
    }
}

#[async_trait]
impl MaterialStorage for DirectoryMaterialStorage {
    async fn put(&self, ca_id: Uuid, file: &MaterialFile, bytes: &[u8]) -> io::Result<()> {
        let path = self.path(ca_id, file);
        let directory = path.parent().expect("chave com diretório de CA");
        create_private_directory(directory).await?;
        let staging = directory.join(format!(".upload-{}", Uuid::new_v4()));
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        let mut output = options.open(&staging).await?;
        let result = async {
            output.write_all(bytes).await?;
            output.sync_all().await?;
            drop(output);
            fs::rename(&staging, &path).await?;
            fs::File::open(directory).await?.sync_all().await?;
            fs::File::open(&self.root).await?.sync_all().await
        }
        .await;
        if result.is_err() {
            let _ = fs::remove_file(staging).await;
        }
        result
    }

    async fn read(&self, ca_id: Uuid, file: &MaterialFile) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(file.size as usize);
        fs::File::open(self.path(ca_id, file))
            .await?
            .take(file.size as u64 + 1)
            .read_to_end(&mut bytes)
            .await?;
        let sha256: [u8; 32] = Sha256::digest(&bytes).into();
        if bytes.len() != file.size as usize || sha256 != file.sha256 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "PDF armazenado não confere com os metadados",
            ));
        }
        Ok(bytes)
    }
}

async fn create_private_directory(path: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    builder.mode(0o700);
    builder.create(path).await
}
