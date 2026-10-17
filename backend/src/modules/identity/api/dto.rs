//! Contratos HTTP do módulo de identidade institucional.

use serde::Serialize;

/// Identidade retornada após a validação da autenticação.
#[derive(Debug, Serialize)]
pub struct CurrentIdentityResponse {
    /// Identificador único do usuário no Firebase Authentication.
    pub uid: String,

    /// E-mail informado pela identidade autenticada, quando disponível.
    pub email: Option<String>,

    /// Indica se o Firebase considera o e-mail verificado.
    pub email_verified: bool,
}
