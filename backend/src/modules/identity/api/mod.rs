//! Contratos HTTP da verificação institucional.
//!
//! [`handlers`] interpreta credenciais e chama o serviço; [`dto`] serializa o
//! conta provisionada e a evidência; [`error`] converte erros de aplicação em respostas públicas.
//! [`extractors`] oferece proteção reutilizável para handlers que exijam uma
//! confirmação institucional existente. Esse extractor não deve anteceder a
//! operação que cria ou renova a própria confirmação.
//!
//! [`router`] também expõe GET /auth/session e PUT /users/me/profile para restaurar
//! a conta interna e concluir seu perfil. Só o endpoint institucional precisa de
//! access token Microsoft. A autenticação de perfil é conferida pela aplicação.

use axum::{Router, routing::post};

use crate::api::state::AppState;

pub mod dto;
pub mod error;
pub mod extractors;
pub mod handlers;

/// Constrói as rotas de identidade, ainda dependentes do estado.
///
/// # Rotas
///
/// - `POST /auth/institutional/verify`: valida as credenciais Firebase e Microsoft,
///   registra a evidência institucional e retorna a conta provisionada e a evidência.
///
/// # Retorno
///
/// Router que deve receber [`AppState`] antes de atender requisições. Sua construção
/// não consulta o banco ou os provedores. A validação das credenciais ocorre no
/// handler; não exige uma confirmação institucional prévia, pois esta rota também
/// atende ao primeiro acesso e à renovação de uma confirmação vencida.
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/auth/session",
            axum::routing::get(handlers::restore_session),
        )
        .route(
            "/users/me/profile",
            axum::routing::put(handlers::complete_profile),
        )
        .route(
            "/auth/institutional/verify",
            post(handlers::verify_institutional),
        )
}

#[cfg(test)]
mod tests;
