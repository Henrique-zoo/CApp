//! Entidade de domínio que representa uma notícia no mural do CApp.
//!
//! A notícia é o elemento central de comunicação e divulgação institucional
//! de um Centro Acadêmico (CA), vinculada à Universidade de Brasília (UnB).

use std::{error::Error, fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::news::domain::rules::{
    validate_body, validate_news, validate_status_transition, validate_title,
};

/// Erros decorrentes de violações de regras de negócio ou integridade de uma notícia.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NewsError {
    /// O título da notícia não pode ser vazio ou composto somente por espaços.
    EmptyTitle,

    /// O título da notícia é menor que o limite mínimo permitido.
    TitleTooShort {
        /// Limite mínimo em caracteres.
        min: usize,
        /// Comprimento verificado.
        actual: usize,
    },

    /// O título da notícia ultrapassou o limite máximo permitido.
    TitleTooLong {
        /// Limite máximo em caracteres.
        max: usize,
        /// Comprimento verificado.
        actual: usize,
    },

    /// O corpo da notícia não pode ser vazio ou composto somente por espaços.
    EmptyBody,

    /// O corpo da notícia é menor que o limite mínimo permitido.
    BodyTooShort {
        /// Limite mínimo em caracteres.
        min: usize,
        /// Comprimento verificado.
        actual: usize,
    },

    /// O corpo da notícia ultrapassou o limite máximo permitido.
    BodyTooLong {
        /// Limite máximo em caracteres.
        max: usize,
        /// Comprimento verificado.
        actual: usize,
    },

    /// A quantidade de tags ultrapassou o limite máximo permitido.
    TooManyTags {
        /// Limite máximo de tags.
        max: usize,
        /// Quantidade informada.
        actual: usize,
    },

    /// Uma tag/categoria não pode ser vazia ou composta somente por espaços.
    EmptyTag,

    /// Uma tag/categoria ultrapassou o limite máximo de caracteres.
    TagTooLong {
        /// Limite máximo em caracteres.
        max: usize,
        /// Comprimento verificado.
        actual: usize,
    },

    /// O status informado não é reconhecido pelo domínio.
    InvalidStatus(String),

    /// A transição de ciclo de vida solicitada é inválida ou proibida.
    InvalidStatusTransition {
        /// Status de origem.
        from: NewsStatus,
        /// Status de destino pretendido.
        to: NewsStatus,
    },

    /// Uma notícia marcada como publicada exige data de publicação.
    PublishedWithoutDate,

    /// Uma notícia em elaboração ou pendente de análise não pode ter data de publicação.
    UnpublishedWithDate,

    /// Uma notícia marcada como arquivada exige data de arquivamento.
    ArchivedWithoutDate,

    /// Os timestamps de criação e atualização violam a coerência temporal.
    InconsistentTimestamps {
        /// Instante de criação.
        created_at: DateTime<Utc>,
        /// Instante de atualização.
        updated_at: DateTime<Utc>,
    },

    /// A data de publicação não pode ser anterior à data de criação.
    InconsistentPublicationDate {
        /// Instante de criação.
        created_at: DateTime<Utc>,
        /// Instante de publicação.
        published_at: DateTime<Utc>,
    },

    /// A data de arquivamento não pode ser anterior à data de criação.
    InconsistentArchiveDate {
        /// Instante de criação.
        created_at: DateTime<Utc>,
        /// Instante de arquivamento.
        archived_at: DateTime<Utc>,
    },

    /// Tentativa de modificar o conteúdo de uma notícia já arquivada.
    CannotModifyArchivedNews,
}

impl fmt::Display for NewsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTitle => write!(f, "o título da notícia não pode ser vazio"),
            Self::TitleTooShort { min, actual } => {
                write!(
                    f,
                    "o título da notícia deve ter no mínimo {min} caracteres (informado: {actual})"
                )
            }
            Self::TitleTooLong { max, actual } => {
                write!(
                    f,
                    "o título da notícia excede o limite de {max} caracteres (informado: {actual})"
                )
            }
            Self::EmptyBody => write!(f, "o conteúdo da notícia não pode ser vazio"),
            Self::BodyTooShort { min, actual } => {
                write!(
                    f,
                    "o conteúdo da notícia deve ter no mínimo {min} caracteres (informado: {actual})"
                )
            }
            Self::BodyTooLong { max, actual } => {
                write!(
                    f,
                    "o conteúdo da notícia excede o limite de {max} caracteres (informado: {actual})"
                )
            }
            Self::TooManyTags { max, actual } => {
                write!(
                    f,
                    "a notícia pode ter no máximo {max} tags (informado: {actual})"
                )
            }
            Self::EmptyTag => write!(f, "a tag não pode ser vazia"),
            Self::TagTooLong { max, actual } => {
                write!(
                    f,
                    "a tag excede o limite de {max} caracteres (informado: {actual})"
                )
            }
            Self::InvalidStatus(status) => {
                write!(f, "status de notícia inválido ou desconhecido: '{status}'")
            }
            Self::InvalidStatusTransition { from, to } => {
                write!(f, "transição de status inválida de '{from}' para '{to}'")
            }
            Self::PublishedWithoutDate => {
                write!(
                    f,
                    "uma notícia publicada deve possuir a data de publicação registrada"
                )
            }
            Self::UnpublishedWithDate => {
                write!(
                    f,
                    "notícia em elaboração ou pendente não pode possuir data de publicação"
                )
            }
            Self::ArchivedWithoutDate => {
                write!(
                    f,
                    "uma notícia arquivada deve possuir a data de arquivamento registrada"
                )
            }
            Self::InconsistentTimestamps {
                created_at,
                updated_at,
            } => {
                write!(
                    f,
                    "data de atualização ({updated_at}) não pode ser anterior à data de criação ({created_at})"
                )
            }
            Self::InconsistentPublicationDate {
                created_at,
                published_at,
            } => {
                write!(
                    f,
                    "data de publicação ({published_at}) não pode ser anterior à data de criação ({created_at})"
                )
            }
            Self::InconsistentArchiveDate {
                created_at,
                archived_at,
            } => {
                write!(
                    f,
                    "data de arquivamento ({archived_at}) não pode ser anterior à data de criação ({created_at})"
                )
            }
            Self::CannotModifyArchivedNews => {
                write!(f, "não é permitido modificar uma notícia já arquivada")
            }
        }
    }
}

impl Error for NewsError {}

/// Situação da notícia no seu ciclo de vida editorial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NewsStatus {
    /// Notícia em elaboração/rascunho, sem exibição pública ou pedido de análise.
    #[default]
    Draft,
    /// Proposta submetida pelo autor aguardando validação/homologação de gestor.
    PendingReview,
    /// Notícia aprovada ou publicada diretamente, disponível nos murais autorizados.
    Published,
    /// Proposta rejeitada na análise de homologação.
    Rejected,
    /// Notícia retirada da exibição corrente, preservada para histórico.
    Archived,
}

impl NewsStatus {
    /// Retorna a representação textual canônica em snake_case.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::PendingReview => "pending_review",
            Self::Published => "published",
            Self::Rejected => "rejected",
            Self::Archived => "archived",
        }
    }

    /// Indica se a notícia está publicada e disponível publicamente.
    pub const fn is_published(&self) -> bool {
        matches!(self, Self::Published)
    }

    /// Indica se a notícia está em estado de rascunho/elaboração.
    pub const fn is_draft(&self) -> bool {
        matches!(self, Self::Draft)
    }

    /// Indica se a notícia está arquivada.
    pub const fn is_archived(&self) -> bool {
        matches!(self, Self::Archived)
    }

    /// Indica se a notícia está pendente de validação/homologação.
    pub const fn is_pending_review(&self) -> bool {
        matches!(self, Self::PendingReview)
    }

    /// Indica se a proposta da notícia foi recusada.
    pub const fn is_rejected(&self) -> bool {
        matches!(self, Self::Rejected)
    }
}

impl fmt::Display for NewsStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for NewsStatus {
    type Err = NewsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "draft" | "rascunho" => Ok(Self::Draft),
            "pending_review" | "pending-review" | "pending" | "pendente" | "pendente_revisao" => {
                Ok(Self::PendingReview)
            }
            "published" | "publicada" | "publicado" => Ok(Self::Published),
            "rejected" | "rejeitada" | "rejeitado" => Ok(Self::Rejected),
            "archived" | "arquivada" | "arquivado" => Ok(Self::Archived),
            other => Err(NewsError::InvalidStatus(other.to_string())),
        }
    }
}

/// Entidade de domínio representando uma notícia ou informe no mural de um Centro Acadêmico.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsArticle {
    /// Identificador único e imutável da notícia.
    pub id: Uuid,
    /// Identificador do Centro Acadêmico proprietário e publicador.
    pub academic_center_id: Uuid,
    /// Identificador do autor do conteúdo.
    pub author_id: Uuid,
    /// Título informativo apresentado nos murais e listagens.
    pub title: String,
    /// Conteúdo textual completo da notícia.
    pub body: String,
    /// Situação editorial atual da notícia.
    pub status: NewsStatus,
    /// Categorias ou palavras-chave de indexação.
    pub tags: Vec<String>,
    /// Instante em que o registro foi criado no sistema.
    pub created_at: DateTime<Utc>,
    /// Instante da última modificação do registro.
    pub updated_at: DateTime<Utc>,
    /// Instante em que a notícia foi publicada (se aplicável).
    pub published_at: Option<DateTime<Utc>>,
    /// Instante em que a notícia foi arquivada (se aplicável).
    pub archived_at: Option<DateTime<Utc>>,
}

impl NewsArticle {
    /// Cria uma nova instância de notícia validando todas as suas invariantes de domínio.
    ///
    /// Remove espaços em branco superfluos das extremidades de título, corpo e tags.
    ///
    /// # Erros
    ///
    /// Retorna [`NewsError`] caso título, corpo, tags, datas ou status violem as regras de negócio.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        academic_center_id: Uuid,
        author_id: Uuid,
        title: String,
        body: String,
        status: NewsStatus,
        tags: Vec<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        published_at: Option<DateTime<Utc>>,
        archived_at: Option<DateTime<Utc>>,
    ) -> Result<Self, NewsError> {
        let trimmed_tags = tags.into_iter().map(|tag| tag.trim().to_string()).collect();

        let instance = Self {
            id,
            academic_center_id,
            author_id,
            title: title.trim().to_string(),
            body: body.trim().to_string(),
            status,
            tags: trimmed_tags,
            created_at,
            updated_at,
            published_at,
            archived_at,
        };

        instance.validate()?;
        Ok(instance)
    }

    /// Constrói um novo rascunho de notícia associado ao instante atual.
    ///
    /// # Erros
    ///
    /// Retorna [`NewsError`] se o título, corpo ou tags violarem os critérios de validação.
    pub fn draft(
        id: Uuid,
        academic_center_id: Uuid,
        author_id: Uuid,
        title: String,
        body: String,
        tags: Vec<String>,
    ) -> Result<Self, NewsError> {
        let now = Utc::now();
        Self::new(
            id,
            academic_center_id,
            author_id,
            title,
            body,
            NewsStatus::Draft,
            tags,
            now,
            now,
            None,
            None,
        )
    }

    /// Publica a notícia registrando o instante de publicação e atualizando o status.
    ///
    /// # Erros
    ///
    /// Retorna [`NewsError::InvalidStatusTransition`] se a transição não for permitida
    /// (por exemplo, a partir de arquivada ou rejeitada).
    /// Retorna [`NewsError`] se o título ou corpo atuais não forem válidos para publicação.
    pub fn publish(&mut self, published_at: DateTime<Utc>) -> Result<(), NewsError> {
        validate_status_transition(self.status, NewsStatus::Published)?;
        validate_title(&self.title)?;
        validate_body(&self.body)?;

        if published_at < self.created_at {
            return Err(NewsError::InconsistentPublicationDate {
                created_at: self.created_at,
                published_at,
            });
        }

        self.status = NewsStatus::Published;
        self.published_at = Some(published_at);
        self.updated_at = published_at;
        Ok(())
    }

    /// Arquiva a notícia, retirando-a da listagem corrente e registrando o instante.
    ///
    /// # Erros
    ///
    /// Retorna [`NewsError::InconsistentArchiveDate`] se `archived_at` for anterior a `created_at`.
    /// Retorna [`NewsError::InvalidStatusTransition`] se a notícia já estiver arquivada e a transição for recusada.
    pub fn archive(&mut self, archived_at: DateTime<Utc>) -> Result<(), NewsError> {
        validate_status_transition(self.status, NewsStatus::Archived)?;

        if archived_at < self.created_at {
            return Err(NewsError::InconsistentArchiveDate {
                created_at: self.created_at,
                archived_at,
            });
        }

        self.status = NewsStatus::Archived;
        self.archived_at = Some(archived_at);
        self.updated_at = archived_at;
        Ok(())
    }

    /// Submete a notícia em rascunho para homologação/validação por um gestor.
    ///
    /// # Erros
    ///
    /// Retorna [`NewsError::InvalidStatusTransition`] se o status atual não permitir submissão.
    pub fn submit_for_review(&mut self, at: DateTime<Utc>) -> Result<(), NewsError> {
        validate_status_transition(self.status, NewsStatus::PendingReview)?;
        self.status = NewsStatus::PendingReview;
        self.updated_at = at;
        Ok(())
    }

    /// Registra a recusa/rejeição de uma notícia pendente de homologação.
    ///
    /// # Erros
    ///
    /// Retorna [`NewsError::InvalidStatusTransition`] se o status atual não permitir rejeição.
    pub fn reject(&mut self, at: DateTime<Utc>) -> Result<(), NewsError> {
        validate_status_transition(self.status, NewsStatus::Rejected)?;
        self.status = NewsStatus::Rejected;
        self.updated_at = at;
        Ok(())
    }

    /// Atualiza o conteúdo textual e tags da notícia sem alterar seu estado de publicação.
    ///
    /// # Erros
    ///
    /// - [`NewsError::CannotModifyArchivedNews`]: se a notícia já estiver arquivada.
    /// - [`NewsError::EmptyTitle`]: se o novo título for vazio.
    /// - [`NewsError::EmptyBody`]: se o novo corpo for vazio.
    /// - [`NewsError::InconsistentTimestamps`]: se `updated_at` for anterior a `created_at`.
    pub fn update_content(
        &mut self,
        new_title: String,
        new_body: String,
        new_tags: Option<Vec<String>>,
        updated_at: DateTime<Utc>,
    ) -> Result<(), NewsError> {
        if self.status == NewsStatus::Archived {
            return Err(NewsError::CannotModifyArchivedNews);
        }

        let trimmed_title = new_title.trim().to_string();
        let trimmed_body = new_body.trim().to_string();

        validate_title(&trimmed_title)?;
        validate_body(&trimmed_body)?;

        let tags_to_validate = match &new_tags {
            Some(tags) => tags
                .iter()
                .map(|t| t.trim().to_string())
                .collect::<Vec<_>>(),
            None => self.tags.clone(),
        };
        crate::modules::news::domain::rules::validate_tags(&tags_to_validate)?;

        if updated_at < self.created_at {
            return Err(NewsError::InconsistentTimestamps {
                created_at: self.created_at,
                updated_at,
            });
        }

        self.title = trimmed_title;
        self.body = trimmed_body;
        if new_tags.is_some() {
            self.tags = tags_to_validate;
        }
        self.updated_at = updated_at;
        Ok(())
    }

    /// Valida as invariantes e integridade da notícia delegando ao módulo de regras.
    ///
    /// # Erros
    ///
    /// Retorna [`NewsError`] se qualquer regra for violada.
    pub fn validate(&self) -> Result<(), NewsError> {
        validate_news(self)
    }

    // --- Métodos de Conveniência e Acessores ---

    /// Identificador único da notícia.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Centro Acadêmico responsável pela publicação.
    pub fn academic_center_id(&self) -> Uuid {
        self.academic_center_id
    }

    /// Autor do conteúdo da notícia.
    pub fn author_id(&self) -> Uuid {
        self.author_id
    }

    /// Autor do conteúdo da notícia (alias didático).
    pub fn autor_id(&self) -> Uuid {
        self.author_id
    }

    /// Título da notícia.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Título da notícia (alias didático em português).
    pub fn titulo(&self) -> &str {
        &self.title
    }

    /// Corpo/conteúdo da notícia.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Corpo da notícia (alias didático em português).
    pub fn corpo(&self) -> &str {
        &self.body
    }

    /// Conteúdo da notícia (alias didático em português).
    pub fn conteudo(&self) -> &str {
        &self.body
    }

    /// Status atual da notícia no ciclo de vida editorial.
    pub fn status(&self) -> NewsStatus {
        self.status
    }

    /// Lista de tags associadas.
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Lista de categorias/tags associadas (alias didático em português).
    pub fn categorias(&self) -> &[String] {
        &self.tags
    }

    /// Instante de criação do registro.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Instante de criação do registro (alias didático em português).
    pub fn criado_em(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Instante da última atualização do registro.
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Instante da última atualização do registro (alias didático em português).
    pub fn atualizado_em(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Instante de publicação, se a notícia já foi publicada.
    pub fn published_at(&self) -> Option<DateTime<Utc>> {
        self.published_at
    }

    /// Instante de publicação (alias didático em português).
    pub fn publicado_em(&self) -> Option<DateTime<Utc>> {
        self.published_at
    }

    /// Instante de arquivamento, se a notícia já foi arquivada.
    pub fn archived_at(&self) -> Option<DateTime<Utc>> {
        self.archived_at
    }

    /// Instante de arquivamento (alias didático em português).
    pub fn arquivado_em(&self) -> Option<DateTime<Utc>> {
        self.archived_at
    }
}

/// Alias canônico para a entidade de Notícia no mural.
pub type NewsPost = NewsArticle;

/// Alias adicional para a entidade de Notícia.
pub type News = NewsArticle;

/// Alias em português para a entidade de Notícia.
pub type Noticia = NewsArticle;

/// Alias em português para a situação da notícia.
pub type StatusNoticia = NewsStatus;

/// Alias canônico para o tipo de erro de notícia.
pub type NewsArticleError = NewsError;

/// Alias em português para o tipo de erro de notícia.
pub type NoticiaError = NewsError;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::news::domain::rules::{MAX_BODY_LENGTH, MAX_TITLE_LENGTH};

    const CAER_DIDACTIC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0001);
    const CACC_DIDACTIC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0002);
    const AUTHOR_DIDACTIC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0003);

    fn sample_valid_draft() -> NewsArticle {
        NewsArticle::draft(
            Uuid::new_v4(),
            CACC_DIDACTIC_ID,
            AUTHOR_DIDACTIC_ID,
            "Assembleia geral".to_string(),
            "Pauta da próxima assembleia discente".to_string(),
            vec!["assembleia".to_string(), "cacc".to_string()],
        )
        .expect("deve criar um rascunho de notícia válido")
    }

    #[test]
    fn should_create_valid_draft_successfully() {
        let news = sample_valid_draft();
        assert_eq!(news.academic_center_id, CACC_DIDACTIC_ID);
        assert_eq!(news.author_id, AUTHOR_DIDACTIC_ID);
        assert_eq!(news.title, "Assembleia geral");
        assert_eq!(news.body, "Pauta da próxima assembleia discente");
        assert_eq!(news.status, NewsStatus::Draft);
        assert!(news.published_at.is_none());
        assert!(news.archived_at.is_none());
        assert!(news.created_at <= news.updated_at);
        assert_eq!(news.tags.len(), 2);
    }

    #[test]
    fn should_trim_whitespace_from_fields_on_creation() {
        let now = Utc::now();
        let news = NewsArticle::new(
            Uuid::new_v4(),
            CAER_DIDACTIC_ID,
            AUTHOR_DIDACTIC_ID,
            "   Semana de Redes   ".to_string(),
            "   Conteúdo detalhado da programação   ".to_string(),
            NewsStatus::Draft,
            vec!["  redes  ".to_string(), "  unb  ".to_string()],
            now,
            now,
            None,
            None,
        )
        .expect("deve remover espaços em branco nas extremidades");

        assert_eq!(news.title, "Semana de Redes");
        assert_eq!(news.body, "Conteúdo detalhado da programação");
        assert_eq!(news.tags, vec!["redes".to_string(), "unb".to_string()]);
    }

    #[test]
    fn should_fail_creation_when_title_is_empty_or_whitespace() {
        let err_empty = NewsArticle::draft(
            Uuid::new_v4(),
            CAER_DIDACTIC_ID,
            AUTHOR_DIDACTIC_ID,
            "".to_string(),
            "Conteúdo válido para a notícia".to_string(),
            vec![],
        )
        .unwrap_err();
        assert_eq!(err_empty, NewsError::EmptyTitle);

        let err_ws = NewsArticle::draft(
            Uuid::new_v4(),
            CAER_DIDACTIC_ID,
            AUTHOR_DIDACTIC_ID,
            "   ".to_string(),
            "Conteúdo válido para a notícia".to_string(),
            vec![],
        )
        .unwrap_err();
        assert_eq!(err_ws, NewsError::EmptyTitle);
    }

    #[test]
    fn should_fail_creation_when_title_is_too_short_or_too_long() {
        let err_short = NewsArticle::draft(
            Uuid::new_v4(),
            CAER_DIDACTIC_ID,
            AUTHOR_DIDACTIC_ID,
            "Oi".to_string(),
            "Conteúdo válido para a notícia".to_string(),
            vec![],
        )
        .unwrap_err();
        assert_eq!(err_short, NewsError::TitleTooShort { min: 3, actual: 2 });

        let err_long = NewsArticle::draft(
            Uuid::new_v4(),
            CAER_DIDACTIC_ID,
            AUTHOR_DIDACTIC_ID,
            "T".repeat(MAX_TITLE_LENGTH + 1),
            "Conteúdo válido para a notícia".to_string(),
            vec![],
        )
        .unwrap_err();
        assert_eq!(
            err_long,
            NewsError::TitleTooLong {
                max: MAX_TITLE_LENGTH,
                actual: MAX_TITLE_LENGTH + 1
            }
        );
    }

    #[test]
    fn should_fail_creation_when_body_is_empty_short_or_long() {
        let err_empty = NewsArticle::draft(
            Uuid::new_v4(),
            CAER_DIDACTIC_ID,
            AUTHOR_DIDACTIC_ID,
            "Título válido".to_string(),
            "   ".to_string(),
            vec![],
        )
        .unwrap_err();
        assert_eq!(err_empty, NewsError::EmptyBody);

        let err_short = NewsArticle::draft(
            Uuid::new_v4(),
            CAER_DIDACTIC_ID,
            AUTHOR_DIDACTIC_ID,
            "Título válido".to_string(),
            "1234".to_string(),
            vec![],
        )
        .unwrap_err();
        assert_eq!(err_short, NewsError::BodyTooShort { min: 5, actual: 4 });

        let err_long = NewsArticle::draft(
            Uuid::new_v4(),
            CAER_DIDACTIC_ID,
            AUTHOR_DIDACTIC_ID,
            "Título válido".to_string(),
            "C".repeat(MAX_BODY_LENGTH + 1),
            vec![],
        )
        .unwrap_err();
        assert_eq!(
            err_long,
            NewsError::BodyTooLong {
                max: MAX_BODY_LENGTH,
                actual: MAX_BODY_LENGTH + 1
            }
        );
    }

    #[test]
    fn should_fail_creation_when_published_has_no_date() {
        let now = Utc::now();
        let err = NewsArticle::new(
            Uuid::new_v4(),
            CAER_DIDACTIC_ID,
            AUTHOR_DIDACTIC_ID,
            "Título válido".to_string(),
            "Conteúdo válido da publicação".to_string(),
            NewsStatus::Published,
            vec![],
            now,
            now,
            None,
            None,
        )
        .unwrap_err();

        assert_eq!(err, NewsError::PublishedWithoutDate);
    }

    #[test]
    fn should_fail_creation_when_draft_has_published_date() {
        let now = Utc::now();
        let err = NewsArticle::new(
            Uuid::new_v4(),
            CAER_DIDACTIC_ID,
            AUTHOR_DIDACTIC_ID,
            "Título válido".to_string(),
            "Conteúdo válido da publicação".to_string(),
            NewsStatus::Draft,
            vec![],
            now,
            now,
            Some(now),
            None,
        )
        .unwrap_err();

        assert_eq!(err, NewsError::UnpublishedWithDate);
    }

    #[test]
    fn should_publish_directly_from_draft() {
        let mut news = sample_valid_draft();
        let pub_time = Utc::now() + chrono::Duration::minutes(1);

        news.publish(pub_time).expect("deve publicar diretamente");
        assert_eq!(news.status, NewsStatus::Published);
        assert_eq!(news.published_at, Some(pub_time));
        assert_eq!(news.updated_at, pub_time);
        assert!(news.status.is_published());
    }

    #[test]
    fn should_fail_publishing_when_date_is_before_creation() {
        let mut news = sample_valid_draft();
        let past = news.created_at - chrono::Duration::minutes(10);

        let err = news.publish(past).unwrap_err();
        assert_eq!(
            err,
            NewsError::InconsistentPublicationDate {
                created_at: news.created_at,
                published_at: past
            }
        );
    }

    #[test]
    fn should_archive_published_news() {
        let mut news = sample_valid_draft();
        let pub_time = Utc::now() + chrono::Duration::minutes(1);
        news.publish(pub_time).unwrap();

        let arch_time = pub_time + chrono::Duration::hours(24);
        news.archive(arch_time).expect("deve arquivar com sucesso");

        assert_eq!(news.status, NewsStatus::Archived);
        assert_eq!(news.archived_at, Some(arch_time));
        assert_eq!(news.published_at, Some(pub_time));
        assert!(news.status.is_archived());
    }

    #[test]
    fn should_fail_transitions_from_archived() {
        let mut news = sample_valid_draft();
        let now = Utc::now() + chrono::Duration::minutes(1);
        news.archive(now).unwrap();

        let err = news
            .publish(now + chrono::Duration::minutes(2))
            .unwrap_err();
        assert_eq!(
            err,
            NewsError::InvalidStatusTransition {
                from: NewsStatus::Archived,
                to: NewsStatus::Published
            }
        );
    }

    #[test]
    fn should_update_content_preserving_publication_status() {
        let mut news = sample_valid_draft();
        let pub_time = Utc::now() + chrono::Duration::minutes(1);
        news.publish(pub_time).unwrap();

        let update_time = pub_time + chrono::Duration::minutes(5);
        news.update_content(
            "Assembleia geral discente".to_string(),
            "Pauta atualizada da assembleia".to_string(),
            Some(vec!["pauta-nova".to_string()]),
            update_time,
        )
        .expect("deve atualizar conteúdo com sucesso");

        assert_eq!(news.title, "Assembleia geral discente");
        assert_eq!(news.body, "Pauta atualizada da assembleia");
        assert_eq!(news.tags, vec!["pauta-nova".to_string()]);
        assert_eq!(news.status, NewsStatus::Published);
        assert_eq!(news.published_at, Some(pub_time));
        assert_eq!(news.updated_at, update_time);
    }

    #[test]
    fn should_fail_updating_content_when_news_is_archived() {
        let mut news = sample_valid_draft();
        let now = Utc::now() + chrono::Duration::minutes(1);
        news.archive(now).unwrap();

        let err = news
            .update_content(
                "Novo título".to_string(),
                "Novo corpo da notícia".to_string(),
                None,
                now + chrono::Duration::minutes(2),
            )
            .unwrap_err();

        assert_eq!(err, NewsError::CannotModifyArchivedNews);
    }

    #[test]
    fn should_handle_editorial_review_lifecycle() {
        let mut news = sample_valid_draft();
        let t1 = Utc::now() + chrono::Duration::minutes(1);
        news.submit_for_review(t1)
            .expect("deve submeter para análise");
        assert_eq!(news.status, NewsStatus::PendingReview);
        assert!(news.status.is_pending_review());

        let t2 = t1 + chrono::Duration::minutes(2);
        news.reject(t2).expect("deve rejeitar proposta");
        assert_eq!(news.status, NewsStatus::Rejected);
        assert!(news.status.is_rejected());
    }

    #[test]
    fn should_parse_status_from_strings() {
        assert_eq!("draft".parse::<NewsStatus>().unwrap(), NewsStatus::Draft);
        assert_eq!("rascunho".parse::<NewsStatus>().unwrap(), NewsStatus::Draft);
        assert_eq!(
            "published".parse::<NewsStatus>().unwrap(),
            NewsStatus::Published
        );
        assert_eq!(
            "publicada".parse::<NewsStatus>().unwrap(),
            NewsStatus::Published
        );
        assert_eq!(
            "archived".parse::<NewsStatus>().unwrap(),
            NewsStatus::Archived
        );
        assert_eq!(
            "arquivada".parse::<NewsStatus>().unwrap(),
            NewsStatus::Archived
        );
        assert_eq!(
            "pending_review".parse::<NewsStatus>().unwrap(),
            NewsStatus::PendingReview
        );
        assert_eq!(
            "rejected".parse::<NewsStatus>().unwrap(),
            NewsStatus::Rejected
        );

        assert!(matches!(
            "desconhecido".parse::<NewsStatus>(),
            Err(NewsError::InvalidStatus(_))
        ));
    }

    #[test]
    fn should_format_error_messages_properly() {
        assert_eq!(
            NewsError::EmptyTitle.to_string(),
            "o título da notícia não pode ser vazio"
        );
        assert_eq!(
            NewsError::EmptyBody.to_string(),
            "o conteúdo da notícia não pode ser vazio"
        );
        assert_eq!(
            NewsError::PublishedWithoutDate.to_string(),
            "uma notícia publicada deve possuir a data de publicação registrada"
        );
        assert_eq!(
            NewsError::CannotModifyArchivedNews.to_string(),
            "não é permitido modificar uma notícia já arquivada"
        );
    }

    #[test]
    fn should_provide_portuguese_accessors() {
        let news = sample_valid_draft();
        assert_eq!(news.titulo(), news.title());
        assert_eq!(news.corpo(), news.body());
        assert_eq!(news.conteudo(), news.body());
        assert_eq!(news.autor_id(), news.author_id());
        assert_eq!(news.categorias(), news.tags());
        assert_eq!(news.criado_em(), news.created_at());
        assert_eq!(news.atualizado_em(), news.updated_at());
        assert_eq!(news.publicado_em(), news.published_at());
        assert_eq!(news.arquivado_em(), news.archived_at());
    }
}
