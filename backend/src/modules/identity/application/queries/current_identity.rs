//! Contratos para consultar a identidade autenticada.

use async_trait::async_trait;
use thiserror::Error;

/// Identidade obtida após a validação de um token de autenticação.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedIdentity {
    /// Identificador único do usuário no provedor de autenticação.
    pub uid: String,
    /// E-mail associado à identidade, quando disponível.
    pub email: Option<String>,
    /// Indica se o provedor confirma o e-mail como verificado.
    pub email_verified: bool,
}

/// Erros esperados durante a validação da identidade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum IdentityVerificationError {
    /// O token é inválido, expirou ou não atende aos requisitos de identidade.
    #[error("Token de identidade inválido ou expirado")]
    InvalidToken,

    /// A configuração necessária para validar o token está ausente.
    #[error("Configuração de autenticação ausente")]
    MissingConfiguration,

    /// O provedor não está disponível para concluir a validação.
    #[error("Provedor de autenticação indisponível")]
    ProviderUnavailable,
}

/// Porta da aplicação para validar tokens e obter a identidade autenticada.
///
/// A implementação concreta pertence à infraestrutura. Os casos de uso
/// dependem deste contrato, não de um SDK ou provedor específico.
#[async_trait]
pub trait IdentityTokenVerifier: Send + Sync {
    /// Valida um token e retorna a identidade autenticada.
    ///
    /// # Erros
    ///
    /// Retorna [`IdentityVerificationError::InvalidToken`] para tokens que
    /// não podem ser aceitos, [`IdentityVerificationError::MissingConfiguration`]
    /// quando a configuração obrigatória está ausente e
    /// [`IdentityVerificationError::ProviderUnavailable`] quando não é possível
    /// concluir a validação devido a uma indisponibilidade operacional.
    async fn verify_id_token(
        &self,
        token: &str,
    ) -> Result<AuthenticatedIdentity, IdentityVerificationError>;
}
