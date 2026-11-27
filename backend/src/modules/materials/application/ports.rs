//! Contratos de persistência, autorização no CA e acesso a arquivos.

use async_trait::async_trait;
use uuid::Uuid;

use crate::modules::materials::domain::entities::StudyMaterial;

use super::services::MaterialError;

/// Conta interna e objeto Microsoft comprovados pelo token.
#[derive(Clone, Copy)]
pub struct MaterialAccount {
    /// Conta interna usada para autoria e concessões.
    pub id: Uuid,
    /// Identidade externa usada para conferir a evidência institucional.
    pub microsoft_object_id: Option<Uuid>,
}

/// Centro autorizado para consulta e sua capacidade de publicação.
pub struct MaterialCenter {
    /// Identificador do CA.
    pub id: Uuid,
    /// Nome exibido no seletor.
    pub name: String,
    /// Concessão efetiva para o CA.
    pub can_publish: bool,
}

/// Disciplina disponível no contexto.
pub struct MaterialDiscipline {
    /// Identificador da disciplina.
    pub id: Uuid,
    /// Nome da disciplina.
    pub name: String,
}

/// Metadados do PDF que acompanha um material.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialFile {
    /// Nome de apresentação, independente da chave usada no storage.
    pub name: String,
    /// Tamanho do PDF em bytes.
    pub size: i32,
    /// Identificador do conteúdo dentro do CA.
    pub sha256: [u8; 32],
}

/// Publicação e metadados necessários para consultá-la.
pub struct PublishedMaterial {
    /// Entidade persistida.
    pub material: StudyMaterial,
    /// Metadados presentes para o tipo arquivo.
    pub file: Option<MaterialFile>,
}

/// Resultado da gravação, inclusive recuperação após repetição.
pub struct Publication {
    /// Publicação gravada ou recuperada.
    pub value: PublishedMaterial,
    /// Indica se a requisição criou uma publicação.
    pub created: bool,
}

/// Repositório que aplica os filtros de visibilidade e a autorização de gravação.
#[async_trait]
pub trait MaterialRepository: Send + Sync {
    /// Lista os CAs ativos da instituição do estudante.
    async fn centers(&self, account: MaterialAccount)
    -> Result<Vec<MaterialCenter>, MaterialError>;
    /// Resolve o CA e a concessão do solicitante.
    async fn center(
        &self,
        account: MaterialAccount,
        ca_id: Uuid,
    ) -> Result<MaterialCenter, MaterialError>;
    /// Lista as disciplinas validadas para o CA.
    async fn disciplines(&self, ca_id: Uuid) -> Result<Vec<MaterialDiscipline>, MaterialError>;
    /// Verifica que a disciplina pertence ao contexto.
    async fn require_discipline(&self, ca_id: Uuid, id: Uuid) -> Result<(), MaterialError>;
    /// Confere novamente a autorização e grava a publicação em uma transação.
    async fn publish(
        &self,
        account: MaterialAccount,
        request_id: Uuid,
        value: PublishedMaterial,
    ) -> Result<Publication, MaterialError>;
    /// Lista apenas publicações do CA especificado.
    async fn list(
        &self,
        ca_id: Uuid,
        discipline_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PublishedMaterial>, MaterialError>;
    /// Recupera uma publicação pelo CA e pelo identificador.
    async fn find(&self, ca_id: Uuid, id: Uuid) -> Result<PublishedMaterial, MaterialError>;
}

/// Bytes privados e endereçados por conteúdo dentro de cada CA.
#[async_trait]
pub trait MaterialStorage: Send + Sync {
    /// Grava os bytes antes da publicação dos metadados.
    async fn put(&self, ca_id: Uuid, file: &MaterialFile, bytes: &[u8]) -> std::io::Result<()>;
    /// Recupera os bytes de uma publicação autorizada.
    async fn read(&self, ca_id: Uuid, file: &MaterialFile) -> std::io::Result<Vec<u8>>;
}
