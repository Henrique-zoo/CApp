//! Entidade de domínio representando um comentário ou interação em uma demanda.
//!
//! Encapsula a mensagem textual, associação ao autor, identificação explícita
//! do papel na interação (`student` vs. `ca_management`) e o instante de registro.

use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::demands::domain::{entities::DemandError, rules::validate_comment_content};

/// Papel do autor na interação com a demanda.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DemandCommentAuthorRole {
    /// Estudante solicitante ou participante da demanda.
    Student,
    /// Representante oficial da gestão do Centro Acadêmico.
    CaManagement,
}

impl DemandCommentAuthorRole {
    /// Retorna a representação textual canônica do papel do autor.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Student => "student",
            Self::CaManagement => "ca_management",
        }
    }
}

impl fmt::Display for DemandCommentAuthorRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for DemandCommentAuthorRole {
    type Err = DemandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "student" | "estudante" | "aluno" => Ok(Self::Student),
            "ca_management" | "ca-management" | "gestao_ca" | "gestão do ca" | "gestao"
            | "gestão" | "management" => Ok(Self::CaManagement),
            other => Err(DemandError::InvalidCommentAuthorRole(other.to_string())),
        }
    }
}

/// Entidade de domínio pura representando um comentário em uma demanda.
///
/// Todos os campos são privados e o acesso se dá via métodos observáveis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemandComment {
    /// Identificador estável do comentário.
    id: Uuid,
    /// Identificador da demanda associada.
    demand_id: Uuid,
    /// Identificador do usuário autor do comentário.
    author_id: Uuid,
    /// Papel do autor no momento da interação.
    author_role: DemandCommentAuthorRole,
    /// Conteúdo textual do comentário.
    content: String,
    /// Instante de registro do comentário em UTC.
    created_at: DateTime<Utc>,
}

impl DemandComment {
    /// Constrói um novo comentário validando invariantes de domínio.
    ///
    /// # Erros
    ///
    /// Retorna [`DemandError::EmptyCommentContent`] caso o texto seja vazio ou só espaços.
    /// Retorna [`DemandError::CommentContentTooLong`] se ultrapassar o limite de caracteres.
    pub fn new(
        id: Uuid,
        demand_id: Uuid,
        author_id: Uuid,
        author_role: DemandCommentAuthorRole,
        content: String,
        created_at: DateTime<Utc>,
    ) -> Result<Self, DemandError> {
        let trimmed = content.trim();
        validate_comment_content(trimmed)?;

        Ok(Self {
            id,
            demand_id,
            author_id,
            author_role,
            content: trimmed.to_string(),
            created_at,
        })
    }

    /// Identificador do comentário.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Identificador da demanda a que pertence.
    pub fn demand_id(&self) -> Uuid {
        self.demand_id
    }

    /// Identificador do usuário autor.
    pub fn author_id(&self) -> Uuid {
        self.author_id
    }

    /// Papel do autor na interação.
    pub fn author_role(&self) -> DemandCommentAuthorRole {
        self.author_role
    }

    /// Conteúdo textual sanitizado do comentário.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Instante de publicação do comentário.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::demands::domain::rules::MAX_DEMAND_COMMENT_CONTENT_LENGTH;

    #[test]
    fn should_create_valid_demand_comment_for_student() {
        let id = Uuid::new_v4();
        let demand_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let now = Utc::now();

        let comment = DemandComment::new(
            id,
            demand_id,
            author_id,
            DemandCommentAuthorRole::Student,
            "   Anexei mais detalhes sobre o pedido   ".to_string(),
            now,
        )
        .expect("comentário válido de estudante");

        assert_eq!(comment.id(), id);
        assert_eq!(comment.demand_id(), demand_id);
        assert_eq!(comment.author_id(), author_id);
        assert_eq!(comment.author_role(), DemandCommentAuthorRole::Student);
        assert_eq!(comment.content(), "Anexei mais detalhes sobre o pedido");
        assert_eq!(comment.created_at(), now);
    }

    #[test]
    fn should_create_valid_demand_comment_for_ca_management() {
        let id = Uuid::new_v4();
        let demand_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let now = Utc::now();

        let comment = DemandComment::new(
            id,
            demand_id,
            author_id,
            DemandCommentAuthorRole::CaManagement,
            "Resposta oficial: entraremos em contato com o departamento.".to_string(),
            now,
        )
        .expect("comentário válido da gestão");

        assert_eq!(comment.author_role(), DemandCommentAuthorRole::CaManagement);
        assert_eq!(
            comment.content(),
            "Resposta oficial: entraremos em contato com o departamento."
        );
    }

    #[test]
    fn should_reject_comment_with_empty_or_whitespace_content() {
        let result = DemandComment::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            DemandCommentAuthorRole::Student,
            "   \t \n  ".to_string(),
            Utc::now(),
        );

        assert_eq!(result, Err(DemandError::EmptyCommentContent));
    }

    #[test]
    fn should_reject_comment_exceeding_max_length() {
        let long_text = "a".repeat(MAX_DEMAND_COMMENT_CONTENT_LENGTH + 1);
        let result = DemandComment::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            DemandCommentAuthorRole::Student,
            long_text,
            Utc::now(),
        );

        assert_eq!(
            result,
            Err(DemandError::CommentContentTooLong {
                max: MAX_DEMAND_COMMENT_CONTENT_LENGTH,
            })
        );
    }

    #[test]
    fn should_parse_and_format_comment_author_role() {
        assert_eq!(
            "student".parse::<DemandCommentAuthorRole>().unwrap(),
            DemandCommentAuthorRole::Student
        );
        assert_eq!(
            "ca_management".parse::<DemandCommentAuthorRole>().unwrap(),
            DemandCommentAuthorRole::CaManagement
        );
        assert_eq!(DemandCommentAuthorRole::Student.to_string(), "student");
        assert_eq!(
            DemandCommentAuthorRole::CaManagement.to_string(),
            "ca_management"
        );
        assert!("invalid_role".parse::<DemandCommentAuthorRole>().is_err());
    }
}
