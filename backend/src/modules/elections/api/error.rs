//! Converte falhas eleitorais em respostas HTTP para o quadro e a própria conta.
//!
//! Os códigos públicos distinguem recusas de falhas de armazenamento. Os
//! detalhes técnicos ficam nos logs do servidor.

use axum::http::StatusCode;

use crate::{
    api::error::ApiError,
    modules::elections::application::{board::BoardError, error::RegistrationError},
};

pub(super) fn public_read_error(error: BoardError) -> ApiError {
    match error {
        BoardError::NotFound => ApiError::new(
            StatusCode::NOT_FOUND,
            "board_not_found",
            "Quadro eleitoral não encontrado.",
            None,
        ),
        BoardError::Storage(error) => {
            tracing::warn!(%error, "Falha na leitura do quadro público.");
            ApiError::service_unavailable()
        }
        error => {
            tracing::error!(%error, "Falha de consistência no quadro público.");
            ApiError::internal_server_error()
        }
    }
}

pub(super) fn receipt_not_found() -> ApiError {
    ApiError::new(
        StatusCode::NOT_FOUND,
        "receipt_not_found",
        "Recibo eleitoral não encontrado.",
        None,
    )
}

pub(super) fn receipt_read_error(error: BoardError) -> ApiError {
    match error {
        BoardError::NotFound => receipt_not_found(),
        BoardError::Storage(error) => {
            tracing::warn!(%error, "Falha na leitura do próprio recibo eleitoral.");
            ApiError::service_unavailable()
        }
        error => {
            tracing::error!(%error, "Falha de consistência no recibo eleitoral.");
            ApiError::internal_server_error()
        }
    }
}

impl From<RegistrationError> for ApiError {
    fn from(error: RegistrationError) -> Self {
        let (status, code, message) = match error {
            RegistrationError::NotFound => (
                StatusCode::NOT_FOUND,
                "election_not_found",
                "Eleição não encontrada.",
            ),
            RegistrationError::NotEligible => (
                StatusCode::FORBIDDEN,
                "voter_not_eligible",
                "Eleitor sem vínculo vigente com o curso da eleição.",
            ),
            RegistrationError::NotRequired => (
                StatusCode::CONFLICT,
                "registration_not_required",
                "Esta eleição vincula a chave no primeiro voto.",
            ),
            RegistrationError::Closed => (
                StatusCode::CONFLICT,
                "registration_closed",
                "O período de cadastro da eleição terminou.",
            ),
            RegistrationError::KeyConflict => (
                StatusCode::CONFLICT,
                "registration_key_conflict",
                "A chave conflita com um cadastro existente.",
            ),
            RegistrationError::InvalidProof => (
                StatusCode::BAD_REQUEST,
                "invalid_registration_proof",
                "Prova de cadastro inválida.",
            ),
            RegistrationError::Storage(_) => {
                tracing::warn!(%error, "Falha de armazenamento no cadastro eleitoral.");
                return Self::service_unavailable();
            }
        };
        Self::new(status, code, message, None)
    }
}

pub(super) fn ballot_error(error: BoardError) -> ApiError {
    let (status, code, message) = match error {
        BoardError::InvalidEvidence(_) | BoardError::VoterMismatch => (
            StatusCode::BAD_REQUEST,
            "invalid_ballot",
            "Cédula inválida para a conta e a eleição.",
        ),
        BoardError::NotEligible => (
            StatusCode::FORBIDDEN,
            "voter_not_eligible",
            "Eleitor sem vínculo vigente com o CA.",
        ),
        BoardError::RegistrationRequired => (
            StatusCode::CONFLICT,
            "registration_required",
            "Cadastre a chave eleitoral antes da abertura.",
        ),
        BoardError::RegisteredKeyMismatch => (
            StatusCode::CONFLICT,
            "registration_key_conflict",
            "A chave da cédula não corresponde à chave cadastrada.",
        ),
        BoardError::NotFound => (
            StatusCode::NOT_FOUND,
            "election_not_found",
            "Eleição não encontrada.",
        ),
        BoardError::Closed | BoardError::VotingEnded => (
            StatusCode::CONFLICT,
            "voting_ended",
            "O período de votação terminou.",
        ),
        BoardError::VotingNotStarted => (
            StatusCode::CONFLICT,
            "voting_not_started",
            "O período de votação ainda não começou.",
        ),
        BoardError::AlreadyParticipated => (
            StatusCode::CONFLICT,
            "already_voted",
            "A conta já votou nesta eleição.",
        ),
        BoardError::SubmissionConflict => (
            StatusCode::CONFLICT,
            "submission_conflict",
            "O conteúdo diverge da cédula confirmada.",
        ),
        BoardError::LimitExceeded => (
            StatusCode::CONFLICT,
            "election_full",
            "A eleição atingiu o limite de participantes.",
        ),
        BoardError::SigningUnavailable | BoardError::Storage(_) => {
            return ApiError::service_unavailable();
        }
        BoardError::InvalidArtifact(_) | BoardError::AlreadyExists | BoardError::NotYetEnded => {
            tracing::error!(%error, "Estado inconsistente ao registrar cédula.");
            return ApiError::internal_server_error();
        }
    };
    ApiError::new(status, code, message, None)
}
