//! Contratos JSON (DTOs) do módulo de Centros Acadêmicos.
//!
//! Define as estruturas de requisição e resposta para configuração inicial
//! e consulta do mandato de gestão do Centro Acadêmico.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::ca::domain::entities::Mandate;

/// Corpo da requisição de configuração inicial do mandato da gestão.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigureInitialMandateRequest {
    /// Data de início do mandato da gestão (AAAA-MM-DD).
    pub start_date: NaiveDate,
    /// Data de término do mandato da gestão (AAAA-MM-DD), estritamente posterior ao início.
    pub end_date: NaiveDate,
    /// Título opcional representativo da chapa ou gestão.
    #[serde(default)]
    pub title: Option<String>,
}

/// Resposta pública contendo as informações e duração do mandato de gestão.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MandateResponse {
    /// Identificador único do mandato.
    pub id: Uuid,
    /// Identificador do Centro Acadêmico correspondente.
    pub academic_center_id: Uuid,
    /// Identificador do usuário que registrou a configuração inicial do mandato.
    pub created_by_id: Uuid,
    /// Título ou nome da chapa/gestão, se informado.
    pub title: Option<String>,
    /// Data de início do período do mandato (AAAA-MM-DD).
    pub start_date: NaiveDate,
    /// Data de término do período do mandato (AAAA-MM-DD).
    pub end_date: NaiveDate,
    /// Situação operacional do mandato ('active', 'concluded', 'archived').
    pub status: String,
    /// Duração do mandato em dias calculada a partir das datas persistidas.
    pub duration_in_days: i64,
    /// Instante UTC de criação do registro.
    pub created_at: DateTime<Utc>,
    /// Instante UTC da última modificação do registro.
    pub updated_at: DateTime<Utc>,
}

impl From<Mandate> for MandateResponse {
    fn from(mandate: Mandate) -> Self {
        Self {
            id: mandate.id(),
            academic_center_id: mandate.academic_center_id(),
            created_by_id: mandate.created_by_id(),
            title: mandate.title().map(String::from),
            start_date: mandate.start_date(),
            end_date: mandate.end_date(),
            status: mandate.status().as_str().to_string(),
            duration_in_days: mandate.duration_in_days(),
            created_at: mandate.created_at(),
            updated_at: mandate.updated_at(),
        }
    }
}
