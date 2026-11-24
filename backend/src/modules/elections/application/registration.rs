//! Tipos de entrada, evidência verificada e leitura do cadastro privado.
//!
//! A aplicação usa a conta autenticada e o horário do banco. O adaptador
//! carrega a situação do quadro e do eleitor sob o mesmo bloqueio usado no
//! aceite do voto, para que a política não possa mudar durante a operação.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::elections::domain::identifiers::AccountId;

/// Prova de posse recebida pela aplicação, independentemente do JSON HTTP.
///
/// A chave pública e a assinatura pertencem à própria prova; o identificador
/// da eleição e da conta é fornecido pelo contexto autenticado do caso de uso.
#[derive(Debug, Clone)]
pub struct RegistrationProof {
    public_key: Box<[u8]>,
    signature: Box<[u8]>,
}

impl RegistrationProof {
    /// Constrói uma prova de posse ainda não verificada.
    pub fn new(public_key: Vec<u8>, signature: Vec<u8>) -> Self {
        Self {
            public_key: public_key.into_boxed_slice(),
            signature: signature.into_boxed_slice(),
        }
    }

    /// Retorna os bytes da chave pública apresentada pelo eleitor.
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    /// Retorna os bytes da assinatura apresentada como prova de posse.
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }
}

/// Chave cuja prova de posse foi verificada criptograficamente.
///
/// A construção é restrita à implementação interna do módulo eleitoral.
#[derive(Debug)]
pub struct VerifiedRegistrationKey(Box<[u8]>);

impl VerifiedRegistrationKey {
    pub(crate) fn from_verified_proof(proof: RegistrationProof) -> Self {
        Self(proof.public_key)
    }

    /// Retorna uma referência aos bytes da chave pública verificada.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Consome o objeto e devolve os bytes da chave pública verificada.
    pub fn into_vec(self) -> Vec<u8> {
        self.0.into_vec()
    }
}

/// Conta e vínculo de identidade obtidos da credencial autenticada.
#[derive(Debug, Clone, Copy)]
pub struct VoterIdentity {
    /// Conta do CApp usada para identificar o eleitor no quadro público.
    pub account_id: AccountId,
    /// Objeto Microsoft vinculado à sessão, necessário para comprovar o vínculo.
    pub microsoft_object_id: Option<Uuid>,
}

/// Estado da própria conta, inclusive após perder a elegibilidade ou o prazo.
#[derive(Debug, PartialEq, Eq)]
pub struct RegistrationStatus {
    /// Exigência escolhida pela comissão e congelada com o quadro.
    pub requires_registration: bool,
    /// Conta com participação confirmada ou chave cadastrada nesta eleição.
    pub account_id: Option<AccountId>,
    /// Chave pública SPKI em DER.
    pub public_key: Option<Vec<u8>>,
    /// Indica aceite de voto e emissão de recibo já confirmados.
    pub participated: bool,
}

/// Período e evidência lidos enquanto a eleição está bloqueada para cadastro.
pub struct RegistrationWindow {
    /// Instante de abertura, que encerra o prazo para novos cadastros.
    pub starts_at: DateTime<Utc>,
    /// Indica que a eleição já foi encerrada.
    pub closed: bool,
    /// Instante obtido depois dos bloqueios necessários.
    pub now: DateTime<Utc>,
    /// Validade do vínculo institucional encontrado para a conta.
    pub eligible_until: Option<DateTime<Utc>>,
}
