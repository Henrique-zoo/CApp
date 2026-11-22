//! Entidade de domínio representando uma pergunta e resposta frequente (FAQ).
//!
//! Entradas de FAQ são mantidas pelo Centro Acadêmico ativo para responder dúvidas
//! comuns da comunidade estudantil.

use std::{error::Error, fmt};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::faq::domain::rules::faq_entry::{
    validate_answer, validate_category, validate_question, validate_timestamps,
};

/// Erros de domínio relacionados a entradas de FAQ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FaqError {
    /// A pergunta não pode ser vazia ou conter apenas espaços em branco.
    EmptyQuestion,

    /// A pergunta ultrapassou o comprimento máximo permitido.
    QuestionTooLong {
        /// Limite máximo em caracteres.
        max: usize,
    },

    /// A resposta não pode ser vazia ou conter apenas espaços em branco.
    EmptyAnswer,

    /// A resposta ultrapassou o comprimento máximo permitido.
    AnswerTooLong {
        /// Limite máximo em caracteres.
        max: usize,
    },

    /// A categoria foi informada com texto em branco.
    EmptyCategory,

    /// A categoria ultrapassou o comprimento máximo permitido.
    CategoryTooLong {
        /// Limite máximo em caracteres.
        max: usize,
    },

    /// Os timestamps de criação e atualização violam a ordem cronológica.
    InconsistentTimestamps {
        /// Data de criação.
        created_at: DateTime<Utc>,
        /// Data de atualização.
        updated_at: DateTime<Utc>,
    },
}

impl fmt::Display for FaqError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyQuestion => write!(f, "a pergunta do FAQ não pode ser vazia"),
            Self::QuestionTooLong { max } => {
                write!(
                    f,
                    "a pergunta do FAQ excede o limite máximo de {max} caracteres"
                )
            }
            Self::EmptyAnswer => write!(f, "a resposta do FAQ não pode ser vazia"),
            Self::AnswerTooLong { max } => {
                write!(
                    f,
                    "a resposta do FAQ excede o limite máximo de {max} caracteres"
                )
            }
            Self::EmptyCategory => {
                write!(f, "a categoria do FAQ não pode ser vazia quando informada")
            }
            Self::CategoryTooLong { max } => {
                write!(
                    f,
                    "a categoria do FAQ excede o limite máximo de {max} caracteres"
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
        }
    }
}

impl Error for FaqError {}

/// Entidade de domínio que representa uma entrada na base de conhecimento (FAQ) do CA.
///
/// Todos os campos são privados para proteger as invariantes de negócio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaqEntry {
    /// Identificador único da entrada de FAQ.
    id: Uuid,
    /// Identificador do Centro Acadêmico proprietário desta entrada.
    academic_center_id: Uuid,
    /// Texto da pergunta frequente.
    question: String,
    /// Resposta fornecida pela gestão do Centro Acadêmico.
    answer: String,
    /// Categoria temática opcional.
    category: Option<String>,
    /// Identificador do usuário que realizou o cadastro inicial.
    created_by: Uuid,
    /// Identificador do usuário que realizou a última modificação.
    updated_by: Uuid,
    /// Instante de criação do registro.
    created_at: DateTime<Utc>,
    /// Instante da última atualização do registro.
    updated_at: DateTime<Utc>,
}

impl FaqEntry {
    /// Cria uma nova entrada de FAQ validando todas as regras e invariantes de negócio.
    ///
    /// Espaços periféricos são removidos da pergunta, resposta e categoria.
    ///
    /// # Erros
    ///
    /// Retorna [`FaqError`] se qualquer parâmetro violar as regras estruturais.
    pub fn new(
        id: Uuid,
        academic_center_id: Uuid,
        question: String,
        answer: String,
        category: Option<String>,
        author_id: Uuid,
        created_at: DateTime<Utc>,
    ) -> Result<Self, FaqError> {
        let trimmed_question = question.trim();
        validate_question(trimmed_question)?;

        let trimmed_answer = answer.trim();
        validate_answer(trimmed_answer)?;

        let sanitized_category = validate_category(category.as_deref())?;

        Ok(Self {
            id,
            academic_center_id,
            question: trimmed_question.to_string(),
            answer: trimmed_answer.to_string(),
            category: sanitized_category,
            created_by: author_id,
            updated_by: author_id,
            created_at,
            updated_at: created_at,
        })
    }

    /// Reconstitui uma entrada de FAQ persistida a partir dos dados do armazenamento.
    ///
    /// Valida as invariantes estruturais e a consistência temporal dos timestamps.
    ///
    /// # Erros
    ///
    /// Retorna [`FaqError`] caso haja violação de integridade.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: Uuid,
        academic_center_id: Uuid,
        question: String,
        answer: String,
        category: Option<String>,
        created_by: Uuid,
        updated_by: Uuid,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, FaqError> {
        let trimmed_question = question.trim();
        validate_question(trimmed_question)?;

        let trimmed_answer = answer.trim();
        validate_answer(trimmed_answer)?;

        let sanitized_category = validate_category(category.as_deref())?;
        validate_timestamps(created_at, updated_at)?;

        Ok(Self {
            id,
            academic_center_id,
            question: trimmed_question.to_string(),
            answer: trimmed_answer.to_string(),
            category: sanitized_category,
            created_by,
            updated_by,
            created_at,
            updated_at,
        })
    }

    /// Identificador único da entrada de FAQ.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Identificador do Centro Acadêmico vinculado.
    pub fn academic_center_id(&self) -> Uuid {
        self.academic_center_id
    }

    /// Texto da pergunta frequente.
    pub fn question(&self) -> &str {
        &self.question
    }

    /// Texto da resposta.
    pub fn answer(&self) -> &str {
        &self.answer
    }

    /// Categoria temática da pergunta, se definida.
    pub fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }

    /// Identificador do usuário que realizou o cadastro.
    pub fn created_by(&self) -> Uuid {
        self.created_by
    }

    /// Identificador do usuário que efetuou a última alteração.
    pub fn updated_by(&self) -> Uuid {
        self.updated_by
    }

    /// Instante de criação da entrada de FAQ.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Instante da última modificação da entrada de FAQ.
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::faq::domain::rules::faq_entry::{
        MAX_ANSWER_LENGTH, MAX_CATEGORY_LENGTH, MAX_QUESTION_LENGTH,
    };

    const CA_ID: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000001");
    const AUTHOR_ID: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000002");

    #[test]
    fn should_create_valid_faq_entry() {
        let now = Utc::now();
        let entry = FaqEntry::new(
            Uuid::new_v4(),
            CA_ID,
            "Como solicitar apoio acadêmico?".to_string(),
            "Entre em contato com a gestão do CA.".to_string(),
            Some("Apoio Acadêmico".to_string()),
            AUTHOR_ID,
            now,
        )
        .expect("deve criar entrada válida");

        assert_eq!(entry.academic_center_id(), CA_ID);
        assert_eq!(entry.question(), "Como solicitar apoio acadêmico?");
        assert_eq!(entry.answer(), "Entre em contato com a gestão do CA.");
        assert_eq!(entry.category(), Some("Apoio Acadêmico"));
        assert_eq!(entry.created_by(), AUTHOR_ID);
        assert_eq!(entry.updated_by(), AUTHOR_ID);
        assert_eq!(entry.created_at(), now);
        assert_eq!(entry.updated_at(), now);
    }

    #[test]
    fn should_trim_whitespace_on_creation() {
        let now = Utc::now();
        let entry = FaqEntry::new(
            Uuid::new_v4(),
            CA_ID,
            "  Qual o horário da biblioteca?  ".to_string(),
            "  Das 08h às 22h.  ".to_string(),
            Some("  Geral  ".to_string()),
            AUTHOR_ID,
            now,
        )
        .expect("deve criar com trim");

        assert_eq!(entry.question(), "Qual o horário da biblioteca?");
        assert_eq!(entry.answer(), "Das 08h às 22h.");
        assert_eq!(entry.category(), Some("Geral"));
    }

    #[test]
    fn should_reject_empty_question() {
        let now = Utc::now();
        let err = FaqEntry::new(
            Uuid::new_v4(),
            CA_ID,
            "   ".to_string(),
            "Resposta válida.".to_string(),
            None,
            AUTHOR_ID,
            now,
        )
        .unwrap_err();

        assert_eq!(err, FaqError::EmptyQuestion);
    }

    #[test]
    fn should_reject_question_too_long() {
        let now = Utc::now();
        let long_question = "A".repeat(MAX_QUESTION_LENGTH + 1);
        let err = FaqEntry::new(
            Uuid::new_v4(),
            CA_ID,
            long_question,
            "Resposta válida.".to_string(),
            None,
            AUTHOR_ID,
            now,
        )
        .unwrap_err();

        assert_eq!(
            err,
            FaqError::QuestionTooLong {
                max: MAX_QUESTION_LENGTH
            }
        );
    }

    #[test]
    fn should_reject_empty_answer() {
        let now = Utc::now();
        let err = FaqEntry::new(
            Uuid::new_v4(),
            CA_ID,
            "Pergunta válida?".to_string(),
            "   ".to_string(),
            None,
            AUTHOR_ID,
            now,
        )
        .unwrap_err();

        assert_eq!(err, FaqError::EmptyAnswer);
    }

    #[test]
    fn should_reject_answer_too_long() {
        let now = Utc::now();
        let long_answer = "B".repeat(MAX_ANSWER_LENGTH + 1);
        let err = FaqEntry::new(
            Uuid::new_v4(),
            CA_ID,
            "Pergunta válida?".to_string(),
            long_answer,
            None,
            AUTHOR_ID,
            now,
        )
        .unwrap_err();

        assert_eq!(
            err,
            FaqError::AnswerTooLong {
                max: MAX_ANSWER_LENGTH
            }
        );
    }

    #[test]
    fn should_reject_empty_category_if_provided() {
        let now = Utc::now();
        let err = FaqEntry::new(
            Uuid::new_v4(),
            CA_ID,
            "Pergunta válida?".to_string(),
            "Resposta válida.".to_string(),
            Some("   ".to_string()),
            AUTHOR_ID,
            now,
        )
        .unwrap_err();

        assert_eq!(err, FaqError::EmptyCategory);
    }

    #[test]
    fn should_reject_category_too_long() {
        let now = Utc::now();
        let long_category = "C".repeat(MAX_CATEGORY_LENGTH + 1);
        let err = FaqEntry::new(
            Uuid::new_v4(),
            CA_ID,
            "Pergunta válida?".to_string(),
            "Resposta válida.".to_string(),
            Some(long_category),
            AUTHOR_ID,
            now,
        )
        .unwrap_err();

        assert_eq!(
            err,
            FaqError::CategoryTooLong {
                max: MAX_CATEGORY_LENGTH
            }
        );
    }

    #[test]
    fn should_reconstitute_valid_faq_entry() {
        let created_at = Utc::now();
        let updated_at = created_at + chrono::Duration::hours(1);
        let entry_id = Uuid::new_v4();

        let entry = FaqEntry::reconstitute(
            entry_id,
            CA_ID,
            "Pergunta?".to_string(),
            "Resposta.".to_string(),
            None,
            AUTHOR_ID,
            AUTHOR_ID,
            created_at,
            updated_at,
        )
        .expect("deve reconstituir");

        assert_eq!(entry.id(), entry_id);
        assert_eq!(entry.created_at(), created_at);
        assert_eq!(entry.updated_at(), updated_at);
    }

    #[test]
    fn should_reject_reconstitution_when_updated_at_is_before_created_at() {
        let created_at = Utc::now();
        let updated_at = created_at - chrono::Duration::seconds(10);

        let err = FaqEntry::reconstitute(
            Uuid::new_v4(),
            CA_ID,
            "Pergunta?".to_string(),
            "Resposta.".to_string(),
            None,
            AUTHOR_ID,
            AUTHOR_ID,
            created_at,
            updated_at,
        )
        .unwrap_err();

        assert_eq!(
            err,
            FaqError::InconsistentTimestamps {
                created_at,
                updated_at
            }
        );
    }
}
