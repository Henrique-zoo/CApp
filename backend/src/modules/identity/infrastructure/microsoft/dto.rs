//! Mensagens privadas do protocolo Microsoft Graph.
//!
//! Os nomes camelCase seguem o JSON externo; esses tipos não são a resposta HTTP do
//! CApp. Valores incompatíveis provocam erro de desserialização e são tratados pelo
//! cliente como resposta inesperada, sem interpretar a falha como ausência de vínculo.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Resposta da consulta ao usuário autenticado.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GraphUserResponse {
    /// Identificador do objeto Microsoft retornado por `/me`; o serviço exige UUID.
    pub id: String,
    /// Nome exibido pelo diretório, se disponível.
    pub display_name: Option<String>,
    /// E-mail informado pelo diretório; pode estar ausente ou nulo.
    pub mail: Option<String>,
    /// Nome de entrada no diretório; não garante existência de caixa postal.
    pub user_principal_name: Option<String>,
    /// Texto usado como curso por convenção da integração institucional.
    pub job_title: Option<String>,
}

/// Consulta ao único grupo configurado para elegibilidade institucional.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GraphCheckMemberGroupsRequest {
    /// Grupo selecionado pelo serviço; serializado como array `groupIds` de um elemento.
    pub group_ids: [Uuid; 1],
}

/// Grupos solicitados nos quais a participação foi confirmada.
#[derive(Deserialize)]
pub(super) struct GraphCheckMemberGroupsResponse {
    /// IDs confirmados dentre os grupos solicitados; vetor vazio representa ausência.
    pub value: Vec<Uuid>,
}

/// Resposta do Graph contendo a única organização do tenant.
#[derive(Deserialize)]
pub(super) struct GraphOrganizationsResponse {
    /// Envelope com exatamente uma organização; outra cardinalidade falha no Serde.
    pub value: [GraphOrganizationResponse; 1],
}

/// Identificador da organização retornada pelo Graph.
#[derive(Deserialize)]
pub(super) struct GraphOrganizationResponse {
    /// UUID organizacional usado como tenant na consulta ao cadastro institucional.
    pub id: Uuid,
}
