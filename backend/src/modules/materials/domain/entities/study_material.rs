//! Entidade de domínio responsável por representar materiais acadêmicos.
//!
//! Um material pertence a um Centro Acadêmico, possui um autor, está associado
//! a uma disciplina e disponibiliza seu conteúdo por meio de um arquivo
//! armazenado ou de um endereço externo.
//!
//! Esta entidade representa somente conceitos e regras pertencentes ao domínio.
//! Ela não consulta o banco de dados, não executa operações HTTP e não verifica
//! permissões de publicação.
//!
//! A existência dos identificadores relacionados, a compatibilidade da
//! disciplina com o CA, a situação do arquivo e as permissões do autor devem
//! ser verificadas pelas camadas responsáveis.
//!
//! O conteúdo é representado por uma enumeração para garantir que cada material
//! possua exatamente uma origem: arquivo ou link externo.

use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::materials::domain::{
    errors::StudyMaterialError,
    rules::{validate_external_link, validate_title},
};

/// Tipo de conteúdo disponibilizado por um material acadêmico.
///
/// Os valores canônicos correspondem ao enum `study_material_kind` do modelo
/// entidade-relacionamento.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StudyMaterialKind {
    /// Conteúdo disponibilizado por meio de um arquivo armazenado.
    File,

    /// Conteúdo disponibilizado por meio de um endereço externo.
    ExternalLink,
}

impl StudyMaterialKind {
    /// Retorna a representação textual canônica do tipo.
    ///
    /// | Variante | Valor |
    /// | --- | --- |
    /// | [`StudyMaterialKind::File`] | `file` |
    /// | [`StudyMaterialKind::ExternalLink`] | `external_link` |
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::ExternalLink => "external_link",
        }
    }
}

impl fmt::Display for StudyMaterialKind {
    /// Escreve a representação textual canônica do tipo.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for StudyMaterialKind {
    type Err = StudyMaterialError;

    /// Converte uma representação textual para o tipo de conteúdo.
    ///
    /// A conversão ignora espaços nas extremidades e diferenças entre
    /// letras maiúsculas e minúsculas.
    ///
    /// # Erros
    ///
    /// Retorna [`StudyMaterialError::InvalidKind`] quando o valor não
    /// corresponde a `file` ou `external_link`.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "file" => Ok(Self::File),
            "external_link" => Ok(Self::ExternalLink),
            invalid => Err(StudyMaterialError::InvalidKind(invalid.to_owned())),
        }
    }
}

/// Estado de disponibilização de um material acadêmico.
///
/// Os estados correspondem à proposta de `study_material_status` no ERD.
/// O estado `archived` representa um material retirado da listagem corrente,
/// preservando os registros e referências necessários ao histórico.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StudyMaterialStatus {
    /// Material disponibilizado aos usuários autorizados.
    Published,

    /// Material retirado da listagem corrente.
    Archived,
}

impl StudyMaterialStatus {
    /// Retorna a representação textual canônica do estado.
    ///
    /// | Variante | Valor |
    /// | --- | --- |
    /// | [`StudyMaterialStatus::Published`] | `published` |
    /// | [`StudyMaterialStatus::Archived`] | `archived` |
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Published => "published",
            Self::Archived => "archived",
        }
    }

    /// Informa se o material está publicado.
    pub const fn is_published(self) -> bool {
        matches!(self, Self::Published)
    }

    /// Informa se o material está arquivado.
    pub const fn is_archived(self) -> bool {
        matches!(self, Self::Archived)
    }
}

impl fmt::Display for StudyMaterialStatus {
    /// Escreve a representação textual canônica do estado.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for StudyMaterialStatus {
    type Err = StudyMaterialError;

    /// Converte uma representação textual para o estado do material.
    ///
    /// # Erros
    ///
    /// Retorna [`StudyMaterialError::InvalidStatus`] quando o valor não
    /// corresponde a `published` ou `archived`.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "published" => Ok(Self::Published),
            "archived" => Ok(Self::Archived),
            invalid => Err(StudyMaterialError::InvalidStatus(invalid.to_owned())),
        }
    }
}

/// Origem do conteúdo disponibilizado por um material acadêmico.
///
/// A enumeração evita representar simultaneamente um arquivo e um link externo
/// ou representar um material sem nenhuma origem de conteúdo.
///
/// A existência e a situação do arquivo armazenado são verificadas pela
/// aplicação e pela infraestrutura de armazenamento, não por este tipo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudyMaterialContent {
    /// Conteúdo disponibilizado por um arquivo armazenado.
    File {
        /// Identificador dos metadados do arquivo armazenado.
        file_id: Uuid,
    },

    /// Conteúdo disponibilizado por um endereço externo.
    ExternalLink {
        /// Endereço externo do conteúdo.
        url: String,
    },
}

impl StudyMaterialContent {
    /// Cria uma referência a um arquivo armazenado.
    ///
    /// A verificação de existência, pertencimento ao CA e estado do arquivo
    /// é responsabilidade das camadas de aplicação e infraestrutura.
    pub const fn file(file_id: Uuid) -> Self {
        Self::File { file_id }
    }

    /// Cria uma referência a um endereço externo.
    ///
    /// Remove espaços em branco das extremidades e rejeita um endereço vazio.
    ///
    /// Esta operação não verifica o esquema, a acessibilidade ou o destino
    /// remoto do endereço. Essas verificações pertencem à aplicação.
    ///
    /// # Erros
    ///
    /// Retorna [`StudyMaterialError::EmptyExternalLink`] quando o endereço
    /// estiver vazio ou contiver somente espaços.
    pub fn external_link(url: impl Into<String>) -> Result<Self, StudyMaterialError> {
        let normalized_url = url.into().trim().to_owned();

        if normalized_url.is_empty() {
            return Err(StudyMaterialError::EmptyExternalLink);
        }

        Ok(Self::ExternalLink {
            url: normalized_url,
        })
    }

    /// Retorna o tipo correspondente à origem do conteúdo.
    pub const fn kind(&self) -> StudyMaterialKind {
        match self {
            Self::File { .. } => StudyMaterialKind::File,
            Self::ExternalLink { .. } => StudyMaterialKind::ExternalLink,
        }
    }

    /// Informa se o conteúdo corresponde a um arquivo armazenado.
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    /// Informa se o conteúdo corresponde a um link externo.
    pub const fn is_external_link(&self) -> bool {
        matches!(self, Self::ExternalLink { .. })
    }

    /// Retorna o identificador do arquivo, quando aplicável.
    pub const fn file_id(&self) -> Option<Uuid> {
        match self {
            Self::File { file_id } => Some(*file_id),
            Self::ExternalLink { .. } => None,
        }
    }

    /// Retorna o endereço externo, quando aplicável.
    pub fn external_url(&self) -> Option<&str> {
        match self {
            Self::File { .. } => None,
            Self::ExternalLink { url } => Some(url.as_str()),
        }
    }

    /// Normaliza a origem do conteúdo antes de associá-la à entidade.
    ///
    /// A validação é repetida aqui porque as variantes públicas da enumeração
    /// também podem ser construídas diretamente.
    fn normalized(self) -> Result<Self, StudyMaterialError> {
        match self {
            Self::File { file_id } => Ok(Self::File { file_id }),
            Self::ExternalLink { url } => Self::external_link(url),
        }
    }
}

/// Material acadêmico disponibilizado por um Centro Acadêmico.
///
/// O material possui identidade própria, contexto organizacional, autoria,
/// disciplina, informações de apresentação, origem do conteúdo e estado.
///
/// Seus campos são privados para impedir modificações que contornem as
/// validações da entidade. As alterações futuras deverão ser realizadas
/// por operações explícitas de domínio ou casos de uso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudyMaterial {
    /// Identificador interno e estável do material.
    id: Uuid,

    /// Centro Acadêmico responsável pelo material.
    ca_id: Uuid,

    /// Disciplina à qual o material está associado.
    discipline_id: Uuid,

    /// Usuário responsável pela autoria do material.
    author_id: Uuid,

    /// Título apresentado nas listagens e detalhes.
    title: String,

    /// Descrição textual apresentada junto ao material.
    description: String,

    /// Origem exclusiva do conteúdo disponibilizado.
    content: StudyMaterialContent,

    /// Estado atual de disponibilização do material.
    status: StudyMaterialStatus,

    /// Instante de criação do registro.
    created_at: DateTime<Utc>,
}

impl StudyMaterial {
    /// Constrói um material acadêmico validando suas invariantes de domínio.
    ///
    /// Os identificadores são recebidos como UUIDs. A existência das entidades
    /// referenciadas e a compatibilidade entre CA, autor e disciplina serão
    /// verificadas nas camadas responsáveis.
    ///
    /// O título, a descrição e o endereço externo são normalizados pela remoção
    /// de espaços em branco nas extremidades.
    ///
    /// A descrição pode estar vazia, conforme o modelo de dados, mas o título
    /// e a origem do conteúdo precisam ser válidos.
    ///
    /// # Erros
    ///
    /// - [`StudyMaterialError::EmptyTitle`] quando o título estiver vazio.
    /// - [`StudyMaterialError::EmptyExternalLink`] quando o endereço externo
    ///   estiver vazio.
    ///
    /// # Observação
    ///
    /// O estado inicial é informado pelo chamador. As regras de autorização
    /// necessárias para publicar um material pertencem à camada de aplicação.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        ca_id: Uuid,
        discipline_id: Uuid,
        author_id: Uuid,
        title: impl Into<String>,
        description: impl Into<String>,
        content: StudyMaterialContent,
        status: StudyMaterialStatus,
        created_at: DateTime<Utc>,
    ) -> Result<Self, StudyMaterialError> {
        let material = Self {
            id,
            ca_id,
            discipline_id,
            author_id,
            title: title.into().trim().to_owned(),
            description: description.into().trim().to_owned(),
            content: content.normalized()?,
            status,
            created_at,
        };

        material.validate()?;

        Ok(material)
    }

    /// Constrói um material cujo conteúdo é um arquivo armazenado.
    ///
    /// A existência e a situação do arquivo não são verificadas neste
    /// construtor.
    ///
    /// # Erros
    ///
    /// Retorna [`StudyMaterialError::EmptyTitle`] quando o título estiver vazio.
    #[allow(clippy::too_many_arguments)]
    pub fn new_file(
        id: Uuid,
        ca_id: Uuid,
        discipline_id: Uuid,
        author_id: Uuid,
        title: impl Into<String>,
        description: impl Into<String>,
        file_id: Uuid,
        status: StudyMaterialStatus,
        created_at: DateTime<Utc>,
    ) -> Result<Self, StudyMaterialError> {
        Self::new(
            id,
            ca_id,
            discipline_id,
            author_id,
            title,
            description,
            StudyMaterialContent::file(file_id),
            status,
            created_at,
        )
    }

    /// Constrói um material cujo conteúdo é um endereço externo.
    ///
    /// O endereço é normalizado, mas não é consultado na rede nem tem
    /// seu destino verificado.
    ///
    /// # Erros
    ///
    /// - [`StudyMaterialError::EmptyTitle`] quando o título estiver vazio.
    /// - [`StudyMaterialError::EmptyExternalLink`] quando o endereço estiver vazio.
    #[allow(clippy::too_many_arguments)]
    pub fn new_external_link(
        id: Uuid,
        ca_id: Uuid,
        discipline_id: Uuid,
        author_id: Uuid,
        title: impl Into<String>,
        description: impl Into<String>,
        url: impl Into<String>,
        status: StudyMaterialStatus,
        created_at: DateTime<Utc>,
    ) -> Result<Self, StudyMaterialError> {
        Self::new(
            id,
            ca_id,
            discipline_id,
            author_id,
            title,
            description,
            StudyMaterialContent::ExternalLink { url: url.into() },
            status,
            created_at,
        )
    }

    /// Valida as invariantes atuais do material.
    ///
    /// Pode ser utilizada por outras operações de domínio para verificar
    /// novamente a integridade estrutural da entidade.
    ///
    /// # Erros
    ///
    /// Retorna um erro quando o título ou a origem do conteúdo forem inválidos.
    pub fn validate(&self) -> Result<(), StudyMaterialError> {
        validate_title(&self.title)?;
        if let StudyMaterialContent::ExternalLink { url } = &self.content {
            validate_external_link(url)?;
        }

        Ok(())
    }

    /// Retorna o identificador estável do material.
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Retorna o identificador do Centro Acadêmico responsável.
    pub const fn ca_id(&self) -> Uuid {
        self.ca_id
    }

    /// Retorna o identificador da disciplina associada.
    pub const fn discipline_id(&self) -> Uuid {
        self.discipline_id
    }

    /// Retorna o identificador do autor responsável.
    pub const fn author_id(&self) -> Uuid {
        self.author_id
    }

    /// Retorna o título normalizado do material.
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    /// Retorna a descrição normalizada do material.
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Retorna a origem do conteúdo.
    pub const fn content(&self) -> &StudyMaterialContent {
        &self.content
    }

    /// Retorna o tipo de conteúdo, derivado da origem.
    pub const fn kind(&self) -> StudyMaterialKind {
        self.content.kind()
    }

    /// Retorna o estado atual do material.
    pub const fn status(&self) -> StudyMaterialStatus {
        self.status
    }

    /// Informa se o material está publicado.
    pub const fn is_published(&self) -> bool {
        self.status.is_published()
    }

    /// Informa se o material está arquivado.
    pub const fn is_archived(&self) -> bool {
        self.status.is_archived()
    }

    /// Retorna o instante de criação do material.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identifiers() -> (Uuid, Uuid, Uuid, Uuid, Uuid) {
        (
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        )
    }

    #[test]
    fn creates_material_with_file_content() {
        let (id, ca_id, discipline_id, author_id, file_id) = identifiers();
        let created_at = Utc::now();

        let material = StudyMaterial::new_file(
            id,
            ca_id,
            discipline_id,
            author_id,
            "  Lista de Cálculo I  ",
            "  Exercícios da primeira unidade.  ",
            file_id,
            StudyMaterialStatus::Published,
            created_at,
        )
        .unwrap();

        assert_eq!(material.id(), id);
        assert_eq!(material.ca_id(), ca_id);
        assert_eq!(material.discipline_id(), discipline_id);
        assert_eq!(material.author_id(), author_id);
        assert_eq!(material.title(), "Lista de Cálculo I");
        assert_eq!(material.description(), "Exercícios da primeira unidade.");
        assert_eq!(material.kind(), StudyMaterialKind::File);
        assert_eq!(material.status(), StudyMaterialStatus::Published);
        assert_eq!(material.created_at(), created_at);
        assert!(material.is_published());
        assert!(!material.is_archived());
        assert_eq!(material.content().file_id(), Some(file_id));
        assert_eq!(material.content().external_url(), None);
        assert!(material.validate().is_ok());
    }

    #[test]
    fn creates_material_with_external_link() {
        let (id, ca_id, discipline_id, author_id, _) = identifiers();
        let created_at = Utc::now();

        let material = StudyMaterial::new_external_link(
            id,
            ca_id,
            discipline_id,
            author_id,
            "  Referências de Estruturas de Dados  ",
            "  Material complementar.  ",
            "  https://example.test/estruturas  ",
            StudyMaterialStatus::Published,
            created_at,
        )
        .unwrap();

        assert_eq!(material.title(), "Referências de Estruturas de Dados");
        assert_eq!(material.description(), "Material complementar.");
        assert_eq!(material.kind(), StudyMaterialKind::ExternalLink);
        assert_eq!(
            material.content().external_url(),
            Some("https://example.test/estruturas")
        );
        assert_eq!(material.content().file_id(), None);
        assert!(material.content().is_external_link());
        assert!(!material.content().is_file());
    }

    #[test]
    fn rejects_empty_title() {
        let (id, ca_id, discipline_id, author_id, file_id) = identifiers();

        let result = StudyMaterial::new_file(
            id,
            ca_id,
            discipline_id,
            author_id,
            "   ",
            "Descrição válida",
            file_id,
            StudyMaterialStatus::Published,
            Utc::now(),
        );

        assert_eq!(result, Err(StudyMaterialError::EmptyTitle));
    }

    #[test]
    fn rejects_empty_external_link() {
        let result = StudyMaterialContent::external_link("   ");

        assert_eq!(result, Err(StudyMaterialError::EmptyExternalLink));
    }

    #[test]
    fn rejects_empty_link_when_content_is_constructed_directly() {
        let (id, ca_id, discipline_id, author_id, _) = identifiers();

        let result = StudyMaterial::new(
            id,
            ca_id,
            discipline_id,
            author_id,
            "Material válido",
            "",
            StudyMaterialContent::ExternalLink {
                url: "   ".to_owned(),
            },
            StudyMaterialStatus::Published,
            Utc::now(),
        );

        assert_eq!(result, Err(StudyMaterialError::EmptyExternalLink));
    }

    #[test]
    fn accepts_empty_description() {
        let (id, ca_id, discipline_id, author_id, file_id) = identifiers();

        let material = StudyMaterial::new_file(
            id,
            ca_id,
            discipline_id,
            author_id,
            "Material sem descrição",
            "",
            file_id,
            StudyMaterialStatus::Published,
            Utc::now(),
        )
        .unwrap();

        assert_eq!(material.description(), "");
    }

    #[test]
    fn content_kind_is_derived_from_its_origin() {
        let file = StudyMaterialContent::file(Uuid::new_v4());
        let link = StudyMaterialContent::external_link("https://example.test").unwrap();

        assert_eq!(file.kind(), StudyMaterialKind::File);
        assert_eq!(link.kind(), StudyMaterialKind::ExternalLink);
    }

    #[test]
    fn file_content_exposes_only_file_identifier() {
        let file_id = Uuid::new_v4();
        let content = StudyMaterialContent::file(file_id);

        assert_eq!(content.file_id(), Some(file_id));
        assert_eq!(content.external_url(), None);
        assert!(content.is_file());
        assert!(!content.is_external_link());
    }

    #[test]
    fn external_link_content_exposes_only_url() {
        let content = StudyMaterialContent::external_link("https://example.test/material").unwrap();

        assert_eq!(
            content.external_url(),
            Some("https://example.test/material")
        );
        assert_eq!(content.file_id(), None);
        assert!(!content.is_file());
        assert!(content.is_external_link());
    }

    #[test]
    fn status_helpers_identify_material_state() {
        assert!(StudyMaterialStatus::Published.is_published());
        assert!(!StudyMaterialStatus::Published.is_archived());

        assert!(StudyMaterialStatus::Archived.is_archived());
        assert!(!StudyMaterialStatus::Archived.is_published());
    }

    #[test]
    fn kind_uses_canonical_textual_values() {
        assert_eq!(StudyMaterialKind::File.as_str(), "file");
        assert_eq!(StudyMaterialKind::ExternalLink.as_str(), "external_link");

        assert_eq!(StudyMaterialKind::File.to_string(), "file");
        assert_eq!(StudyMaterialKind::ExternalLink.to_string(), "external_link");
    }

    #[test]
    fn status_uses_canonical_textual_values() {
        assert_eq!(StudyMaterialStatus::Published.as_str(), "published");
        assert_eq!(StudyMaterialStatus::Archived.as_str(), "archived");

        assert_eq!(StudyMaterialStatus::Published.to_string(), "published");
        assert_eq!(StudyMaterialStatus::Archived.to_string(), "archived");
    }

    #[test]
    fn parses_valid_kinds() {
        assert_eq!(
            "file".parse::<StudyMaterialKind>(),
            Ok(StudyMaterialKind::File)
        );

        assert_eq!(
            "external_link".parse::<StudyMaterialKind>(),
            Ok(StudyMaterialKind::ExternalLink)
        );

        assert_eq!(
            " EXTERNAL_LINK ".parse::<StudyMaterialKind>(),
            Ok(StudyMaterialKind::ExternalLink)
        );
    }

    #[test]
    fn rejects_unknown_kind() {
        assert_eq!(
            "pdf".parse::<StudyMaterialKind>(),
            Err(StudyMaterialError::InvalidKind("pdf".to_owned()))
        );
    }

    #[test]
    fn parses_valid_statuses() {
        assert_eq!(
            "published".parse::<StudyMaterialStatus>(),
            Ok(StudyMaterialStatus::Published)
        );

        assert_eq!(
            "archived".parse::<StudyMaterialStatus>(),
            Ok(StudyMaterialStatus::Archived)
        );

        assert_eq!(
            " ARCHIVED ".parse::<StudyMaterialStatus>(),
            Ok(StudyMaterialStatus::Archived)
        );
    }

    #[test]
    fn rejects_unknown_status() {
        assert_eq!(
            "draft".parse::<StudyMaterialStatus>(),
            Err(StudyMaterialError::InvalidStatus("draft".to_owned()))
        );
    }

    #[test]
    fn preserves_all_relationship_identifiers() {
        let (id, ca_id, discipline_id, author_id, file_id) = identifiers();

        let material = StudyMaterial::new_file(
            id,
            ca_id,
            discipline_id,
            author_id,
            "Resumo de Cálculo",
            "Resumo da disciplina",
            file_id,
            StudyMaterialStatus::Published,
            Utc::now(),
        )
        .unwrap();

        assert_eq!(material.id(), id);
        assert_eq!(material.ca_id(), ca_id);
        assert_eq!(material.discipline_id(), discipline_id);
        assert_eq!(material.author_id(), author_id);
        assert_eq!(material.content().file_id(), Some(file_id));
    }

    #[test]
    fn equality_compares_the_complete_material() {
        let (id, ca_id, discipline_id, author_id, file_id) = identifiers();
        let created_at = Utc::now();

        let first = StudyMaterial::new_file(
            id,
            ca_id,
            discipline_id,
            author_id,
            "Lista de Cálculo",
            "Exercícios",
            file_id,
            StudyMaterialStatus::Published,
            created_at,
        )
        .unwrap();

        let second = StudyMaterial::new_file(
            id,
            ca_id,
            discipline_id,
            author_id,
            "Lista de Cálculo",
            "Exercícios",
            file_id,
            StudyMaterialStatus::Published,
            created_at,
        )
        .unwrap();

        assert_eq!(first, second);
    }
}
