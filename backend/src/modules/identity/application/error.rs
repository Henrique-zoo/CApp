//! Falhas dos casos de uso e dos contratos de persistência de identidade.
//!
//! Os erros não conhecem status HTTP. A camada API decide código e mensagem pública.
//! Os adaptadores registram diagnósticos técnicos antes de convertê-los para os
//! erros de repositório; a conversão para `Internal` não preserva a causa original.

/// Falhas de autenticação e de comprovação institucional.
///
/// As mensagens de `Display` podem ser usadas pela API, mas a classificação
/// programática depende da variante. Falhas de dependência não significam ausência
/// de vínculo, e ausência de evidência reutilizável não significa token inválido.
#[derive(Debug, thiserror::Error)]
pub enum AuthenticationError {
    /// A credencial não passou pela validação.
    #[error("Token de autenticação inválido")]
    InvalidToken,

    /// A credencial ultrapassou seu prazo de validade.
    #[error("Token de autenticação expirado")]
    ExpiredToken,

    /// As credenciais Firebase e Microsoft representam contas diferentes.
    #[error("As identidades autenticadas não correspondem")]
    IdentityMismatch,

    /// O tenant id não é de uma instituição cadastrada.
    #[error("Sua instituição ainda não está cadastrada no CApp.")]
    UnknownInstitution,

    /// A consulta não confirmou o vínculo estudantil exigido.
    #[error("Você não foi reconhecido como aluno da instituição {institution_name}.")]
    StudentMembershipRequired {
        /// Nome público da instituição consultada, para contextualizar a recusa.
        institution_name: String,
    },

    /// Uma dependência necessária está temporariamente indisponível.
    #[error("Serviço de autenticação temporariamente indisponível")]
    ProviderUnavailable,

    /// Uma falha interna impediu a conclusão do caso de uso.
    #[error("Falha interna na autenticação")]
    Internal,

    /// O identificador Microsoft está ausente ou não possui o formato exigido para a autenticação institucional.
    #[error("Identificador Microsoft ausente ou inválido")]
    InvalidMicrosoftId,

    /// Não existe confirmação institucional reutilizável para a identidade.
    ///
    /// O cliente deve repetir a verificação institucional com um
    /// Microsoft access token válido.
    #[error("É necessário verificar novamente seu vínculo institucional.")]
    InstitutionalVerificationRequired,
}

/// Falha técnica na consulta do cadastro institucional.
///
/// Instituição ausente é `Ok(None)` no contrato de busca, não uma variante de erro.
#[derive(Debug)]
pub enum InstitutionRepositoryError {
    /// Falha de consulta ou mapeamento; o diagnóstico detalhado fica nos logs do adaptador.
    InternalServer,
}

impl From<InstitutionRepositoryError> for AuthenticationError {
    /// Converte falha de persistência em falha interna, sem dependência de HTTP.
    fn from(value: InstitutionRepositoryError) -> Self {
        match value {
            InstitutionRepositoryError::InternalServer => AuthenticationError::Internal,
        }
    }
}

/// Falha de persistência ao consultar ou registrar uma verificação institucional.
#[derive(Debug, thiserror::Error)]
#[error("Falha no armazenamento da verificação institucional")]
pub struct InstitutionalVerificationRepositoryError;

impl From<InstitutionalVerificationRepositoryError> for AuthenticationError {
    /// Converte falha no armazenamento de evidência em falha interna do caso de uso.
    fn from(_: InstitutionalVerificationRepositoryError) -> Self {
        Self::Internal
    }
}
