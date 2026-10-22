//! Contratos HTTP da verificação institucional.
//!
//! [`handlers`] interpreta credenciais e chama o serviço; [`dto`] serializa o
//! perfil e a validade; [`error`] converte erros de aplicação em respostas públicas.
//! [`extractors`] oferece proteção reutilizável para handlers que exijam uma
//! confirmação institucional existente. Esse extractor não deve anteceder a
//! operação que cria ou renova a própria confirmação.
//!
//! Atualmente [`router`] registra somente `POST /auth/institutional/verify`;
//! nenhuma rota exige o extractor de confirmação persistida ainda.

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
///   registra a evidência institucional e retorna o perfil e sua validade.
///
/// # Retorno
///
/// Router que deve receber [`AppState`] antes de atender requisições. Sua construção
/// não consulta o banco ou os provedores. A validação das credenciais ocorre no
/// handler; não exige uma confirmação institucional prévia, pois esta rota também
/// atende ao primeiro acesso e à renovação de uma confirmação vencida.
pub fn router() -> Router<AppState> {
    Router::new().route(
        "/auth/institutional/verify",
        post(handlers::verify_institutional),
    )
}

#[cfg(test)]
mod tests;
