//! Identidades designadas para a cerimônia de publicação do manifesto.

use chrono::{DateTime, Utc};
use crypto_bigint::U2048;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use super::board::BoardPublication;
use crate::modules::elections::domain::{
    entities::{
        authority::{Authority, AuthorityRole},
        election::ElectionPeriod,
    },
    identifiers::{AcademicCenterId, AccountId, AuthorityId, ElectionId},
};

/// Atribuição privada recuperada pela conta autenticada.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityAssignment {
    /// Identificador congelado para a prova e para as contribuições.
    pub authority_id: Uuid,
    /// Papel no manifesto, comissão ou chapa.
    pub role: String,
    /// Identificador da chapa quando esta conta a representa.
    pub slate_id: Option<Uuid>,
    /// A chave já foi entregue durante a preparação.
    pub registered: bool,
    /// Início vigente da votação nesta eleição.
    pub starts_at: DateTime<Utc>,
    /// Indica que a comissão cancelou a eleição.
    pub cancelled: bool,
    /// Prazo registrado para a votação nesta eleição.
    pub ends_at: DateTime<Utc>,
    /// Hash do manifesto publicado, disponível depois da preparação.
    pub manifest_hash: Option<String>,
}

/// Prova recebida de uma autoridade, ainda não autenticada criptograficamente.
/// Os identificadores declarados serão comparados à designação autenticada.
pub struct AuthorityRegistrationProof {
    /// Eleição declarada na mensagem assinada.
    pub election: ElectionId,
    /// Chave pública, papel e prova Schnorr declarados.
    pub authority: Authority,
    /// Assinatura RSA-PSS ainda não autenticada.
    pub signature: Vec<u8>,
}

/// Papel e credencial de uma conta na preparação bloqueada.
pub struct AssignedAuthority {
    /// Identificador da autoridade, fixado na designação.
    pub id: AuthorityId,
    /// Conta autenticada que deve entregar a sua própria prova.
    pub account: AccountId,
    /// Papel designado pela comissão para representar uma chapa ou a si.
    pub role: AuthorityRole,
    /// Nome público aprovado para esta chapa.
    pub slate_name: Option<String>,
    /// Transcrição pública entregue e verificada pela conta.
    pub credential: Option<Value>,
}

/// Configuração lida numa transação, antes de congelar o manifesto.
pub struct Preparation {
    /// Eleição do quadro.
    pub election: ElectionId,
    /// Centro acadêmico responsável.
    pub center: AcademicCenterId,
    /// Período vigente na configuração.
    pub period: ElectionPeriod,
    /// Instante do banco após adquirir o bloqueio.
    pub now: DateTime<Utc>,
    /// Política de cadastro escolhida.
    pub requires_registration: bool,
    /// Comissão e chapas designadas em ordem de publicação.
    pub authorities: Vec<AssignedAuthority>,
}

/// Publicação pronta e a chave privada pertencente somente ao servidor.
pub struct PreparedPublication {
    /// Manifesto assinado e período a gravar.
    pub board: BoardPublication,
    /// Identificador da autoridade do servidor.
    pub server_authority: AuthorityId,
    /// Expoente do servidor, destinado à custódia privada.
    pub server_secret: U2048,
}
