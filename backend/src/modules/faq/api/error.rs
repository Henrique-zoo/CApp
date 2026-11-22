//! Mapeamento de erros da camada de aplicação de FAQ para respostas HTTP.

use axum::http::StatusCode;
use serde_json::json;

use crate::{
    api::error::ApiError,
    modules::faq::{
        application::error::FaqApplicationError, domain::entities::faq_entry::FaqError,
    },
};

impl From<FaqApplicationError> for ApiError {
    fn from(error: FaqApplicationError) -> Self {
        match error {
            FaqApplicationError::AcademicCenterNotFound(ca_id) => Self::new(
                StatusCode::NOT_FOUND,
                "academic_center_not_found",
                format!("Centro Acadêmico '{ca_id}' não encontrado"),
                Some(json!({ "academic_center_id": ca_id })),
            ),
            FaqApplicationError::AcademicCenterInactive(ca_id) => Self::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "academic_center_inactive",
                format!("Centro Acadêmico '{ca_id}' não está ativo"),
                Some(json!({ "academic_center_id": ca_id })),
            ),
            FaqApplicationError::UserNotPermittedInCa => Self::new(
                StatusCode::FORBIDDEN,
                "user_not_permitted_in_ca",
                "Usuário sem permissão para cadastrar FAQ neste Centro Acadêmico",
                None,
            ),
            FaqApplicationError::FaqEntryNotFound(entry_id) => Self::new(
                StatusCode::NOT_FOUND,
                "faq_entry_not_found",
                format!("Pergunta frequente '{entry_id}' não encontrada"),
                Some(json!({ "faq_entry_id": entry_id })),
            ),
            FaqApplicationError::InvalidEntry(domain_err) => {
                let (status, code) = match &domain_err {
                    FaqError::EmptyQuestion => (StatusCode::UNPROCESSABLE_ENTITY, "empty_question"),
                    FaqError::QuestionTooLong { .. } => {
                        (StatusCode::UNPROCESSABLE_ENTITY, "question_too_long")
                    }
                    FaqError::EmptyAnswer => (StatusCode::UNPROCESSABLE_ENTITY, "empty_answer"),
                    FaqError::AnswerTooLong { .. } => {
                        (StatusCode::UNPROCESSABLE_ENTITY, "answer_too_long")
                    }
                    FaqError::EmptyCategory => (StatusCode::UNPROCESSABLE_ENTITY, "empty_category"),
                    FaqError::CategoryTooLong { .. } => {
                        (StatusCode::UNPROCESSABLE_ENTITY, "category_too_long")
                    }
                    FaqError::InconsistentTimestamps { .. } => {
                        (StatusCode::UNPROCESSABLE_ENTITY, "inconsistent_timestamps")
                    }
                };
                Self::new(status, code, domain_err.to_string(), None)
            }
            FaqApplicationError::Repository(err) => {
                tracing::error!(%err, "Falha de banco de dados no repositório de FAQ");
                Self::internal_server_error()
            }
            FaqApplicationError::Internal(msg) => {
                tracing::error!(%msg, "Falha interna no módulo de FAQ");
                Self::internal_server_error()
            }
        }
    }
}
