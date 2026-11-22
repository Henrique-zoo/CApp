//! Consulta para listagem e filtragem de demandas de um Centro Acadêmico.

use uuid::Uuid;

use crate::modules::demands::domain::entities::{DemandForum, DemandStatus};

/// Parâmetros da consulta para listagem e filtragem de demandas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListDemandsQuery {
    /// Identificador do Centro Acadêmico alvo da consulta.
    pub academic_center_id: Uuid,

    /// Identificador do usuário solicitante (para verificação de acesso e filtro `mine`).
    pub user_id: Uuid,

    /// Filtro opcional por fórum (`institutional` ou `physical_space`).
    pub forum: Option<DemandForum>,

    /// Filtro opcional por status do ciclo de vida.
    pub status: Option<DemandStatus>,

    /// Quando verdadeiro, restringe a busca às demandas criadas pelo próprio usuário.
    pub mine: bool,

    /// Limite máximo de registros a retornar (paginação).
    pub limit: Option<usize>,

    /// Quantidade de registros a pular (paginação).
    pub offset: Option<usize>,
}

impl ListDemandsQuery {
    /// Cria uma nova consulta de listagem de demandas com os parâmetros especificados.
    pub fn new(
        academic_center_id: Uuid,
        user_id: Uuid,
        forum: Option<DemandForum>,
        status: Option<DemandStatus>,
        mine: bool,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Self {
        Self {
            academic_center_id,
            user_id,
            forum,
            status,
            mine,
            limit,
            offset,
        }
    }
}
