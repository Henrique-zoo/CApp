//! Cadastro privado da chave do eleitor, separado da participação na votação.

use serde::Serialize;
use uuid::Uuid;

/// Identidade estabelecida pelo adaptador de autenticação, nunca pelo corpo HTTP.
#[derive(Debug, Clone, Copy)]
pub struct VoterIdentity {
    /// UUID da conta interna vinculada ao UID Firebase validado.
    pub account_id: Uuid,
    /// Objeto vinculado ao provedor Microsoft no token Firebase atual.
    pub microsoft_object_id: Option<Uuid>,
}

/// Situação da própria conta, recuperável após o fim da elegibilidade.
#[derive(Debug, PartialEq, Eq, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationStatus {
    /// Identificador aleatório e estável apenas para esta eleição.
    pub voter_id: Option<Uuid>,
    /// Chave pública SPKI DER em Base64url sem padding.
    pub public_key: Option<String>,
    /// Indica que um voto e seu recibo foram confirmados na mesma transação.
    pub participated: bool,
}

/// Recusas do cadastro distintas de indisponibilidade técnica.
#[derive(Debug, thiserror::Error)]
pub enum RegistrationError {
    /// Não há quadro publicado para a eleição.
    #[error("Eleição inexistente")]
    NotFound,
    /// Falta vínculo acadêmico válido com a instituição e o curso configurados.
    #[error("Sem elegibilidade para esta eleição")]
    NotEligible,
    /// A abertura ou o encerramento impede novos cadastros.
    #[error("Período de cadastro encerrado")]
    RegistrationClosed,
    /// A conta já possui outra chave nesta eleição.
    #[error("Outra chave já foi cadastrada")]
    KeyConflict,
    /// A prova não corresponde ao contrato, à chave, à conta ou à eleição.
    #[error("Prova de posse inválida")]
    InvalidProof,
    /// Falha técnica sem confirmação da escrita.
    #[error("Falha no armazenamento do cadastro eleitoral")]
    Storage(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl From<sqlx::Error> for RegistrationError {
    fn from(error: sqlx::Error) -> Self {
        Self::Storage(Box::new(error))
    }
}
