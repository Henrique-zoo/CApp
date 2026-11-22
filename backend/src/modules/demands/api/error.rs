//! Mapeamento de erros da camada de aplicação de demandas para respostas HTTP.

use axum::http::StatusCode;
use serde_json::json;

use crate::{
    api::error::ApiError,
    modules::demands::{application::error::DemandApplicationError, domain::entities::DemandError},
};

impl From<DemandApplicationError> for ApiError {
    fn from(error: DemandApplicationError) -> Self {
        match error {
            DemandApplicationError::AcademicCenterNotFound(ca_id) => Self::new(
                StatusCode::NOT_FOUND,
                "academic_center_not_found",
                format!("Centro Acadêmico '{ca_id}' não encontrado"),
                Some(json!({ "academic_center_id": ca_id })),
            ),
            DemandApplicationError::AcademicCenterInactive(ca_id) => Self::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "academic_center_inactive",
                format!("Centro Acadêmico '{ca_id}' não está ativo"),
                Some(json!({ "academic_center_id": ca_id })),
            ),
            DemandApplicationError::StudentMembershipRequired => Self::new(
                StatusCode::FORBIDDEN,
                "student_membership_required",
                "Vínculo estudantil ativo é obrigatório para registrar demandas",
                None,
            ),
            DemandApplicationError::UserNotPermittedInCa => Self::new(
                StatusCode::FORBIDDEN,
                "user_not_permitted_in_ca",
                "Usuário sem permissão para criar demandas neste Centro Acadêmico",
                None,
            ),
            DemandApplicationError::CourseNameRequired => Self::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "course_name_required",
                "Demandas do fórum institucional exigem associação válida a um curso",
                None,
            ),
            DemandApplicationError::CourseNotRepresentedByCa { ca_id, course_name } => Self::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "course_not_represented_by_ca",
                format!(
                    "O curso '{course_name}' não é representado pelo Centro Acadêmico '{ca_id}'"
                ),
                Some(json!({
                    "academic_center_id": ca_id,
                    "course_name": course_name,
                })),
            ),
            DemandApplicationError::DemandNotFound(demand_id) => Self::new(
                StatusCode::NOT_FOUND,
                "demand_not_found",
                format!("Demanda '{demand_id}' não encontrada"),
                Some(json!({ "demand_id": demand_id })),
            ),
            DemandApplicationError::DemandBelongsToAnotherCa { demand_id, ca_id } => Self::new(
                StatusCode::BAD_REQUEST,
                "demand_belongs_to_another_ca",
                format!("A demanda '{demand_id}' pertence a outro Centro Acadêmico"),
                Some(json!({
                    "demand_id": demand_id,
                    "academic_center_id": ca_id,
                })),
            ),
            DemandApplicationError::UserNotPermittedToComment => Self::new(
                StatusCode::FORBIDDEN,
                "user_not_permitted_in_ca",
                "Usuário sem permissão para interagir ou visualizar comentários desta demanda",
                None,
            ),
            DemandApplicationError::InvalidDemand(domain_err) => {
                let (status, code) = match &domain_err {
                    DemandError::EmptyDescription => {
                        (StatusCode::UNPROCESSABLE_ENTITY, "empty_description")
                    }
                    DemandError::DescriptionTooLong { .. } => {
                        (StatusCode::UNPROCESSABLE_ENTITY, "description_too_long")
                    }
                    DemandError::EmptyCommentContent => {
                        (StatusCode::UNPROCESSABLE_ENTITY, "empty_comment")
                    }
                    DemandError::CommentContentTooLong { .. } => {
                        (StatusCode::UNPROCESSABLE_ENTITY, "comment_too_long")
                    }
                    DemandError::InvalidCommentAuthorRole(_) => {
                        (StatusCode::BAD_REQUEST, "invalid_comment_author_role")
                    }
                    DemandError::EmptyTitle => (StatusCode::UNPROCESSABLE_ENTITY, "empty_title"),
                    DemandError::TitleTooLong { .. } => {
                        (StatusCode::UNPROCESSABLE_ENTITY, "title_too_long")
                    }
                    DemandError::EmptyCategory => {
                        (StatusCode::UNPROCESSABLE_ENTITY, "empty_category")
                    }
                    DemandError::CategoryTooLong { .. } => {
                        (StatusCode::UNPROCESSABLE_ENTITY, "category_too_long")
                    }
                    DemandError::EmptyCourseName => {
                        (StatusCode::UNPROCESSABLE_ENTITY, "empty_course_name")
                    }
                    DemandError::CourseNameTooLong { .. } => {
                        (StatusCode::UNPROCESSABLE_ENTITY, "course_name_too_long")
                    }
                    DemandError::CourseNotAllowedForPhysicalSpace => (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "course_not_allowed_for_physical_space",
                    ),
                    DemandError::CoordinationForwardingNotAllowed => (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "coordination_forwarding_not_allowed",
                    ),
                    DemandError::InvalidForum(_) => (StatusCode::BAD_REQUEST, "invalid_forum"),
                    DemandError::InvalidStatus(_) => (StatusCode::BAD_REQUEST, "invalid_status"),
                    DemandError::InvalidStatusTransition { .. } => (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "invalid_status_transition",
                    ),
                    DemandError::InconsistentTimestamps { .. } => {
                        (StatusCode::UNPROCESSABLE_ENTITY, "inconsistent_timestamps")
                    }
                    _ => (StatusCode::BAD_REQUEST, "invalid_demand"),
                };
                Self::new(status, code, domain_err.to_string(), None)
            }
            DemandApplicationError::Repository(err) => {
                tracing::error!(%err, "Falha de banco de dados no repositório de demandas");
                Self::internal_server_error()
            }
            DemandApplicationError::Affiliation(err) => {
                tracing::error!(%err, "Falha ao consultar afiliação");
                Self::internal_server_error()
            }
        }
    }
}
