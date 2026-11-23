//! Caso de uso: Criação de consulta estudantil pela gestão do CA.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::ca::{
    application::{
        error::PollApplicationError,
        ports::{AcademicCenterAuthorizationPort, AcademicCenterRepository, PollRepository},
    },
    domain::entities::{AcademicCenterStatus, StudentPoll},
};

/// Dados de entrada para o comando de criação de consulta estudantil.
#[derive(Debug, Clone)]
pub struct CreatePollCommand {
    /// Identificador do Centro Acadêmico.
    pub academic_center_id: Uuid,
    /// Identificador do usuário que está criando a consulta.
    pub created_by_id: Uuid,
    /// Título oficial da consulta.
    pub title: String,
    /// Descrição ou proposta em discussão (opcional).
    pub description: Option<String>,
    /// Opções de voto disponíveis (mínimo de 2 distintas).
    pub options: Vec<String>,
    /// Instante de início do período de votação.
    pub starts_at: DateTime<Utc>,
    /// Instante de término do período de votação (opcional).
    pub ends_at: Option<DateTime<Utc>>,
    /// Público-alvo / critério de elegibilidade (opcional).
    pub target_audience: Option<String>,
    /// Modalidade de voto ('identified' ou 'secret').
    pub vote_modality: Option<String>,
}

/// Executa o caso de uso de criação de consulta estudantil.
pub async fn execute_create_poll<CA, PR, AP>(
    ca_repo: &CA,
    poll_repo: &PR,
    auth_port: &AP,
    cmd: CreatePollCommand,
) -> Result<StudentPoll, PollApplicationError>
where
    CA: AcademicCenterRepository,
    PR: PollRepository,
    AP: AcademicCenterAuthorizationPort,
{
    let ca = ca_repo.find_by_id(cmd.academic_center_id).await?.ok_or(
        PollApplicationError::AcademicCenterNotFound(cmd.academic_center_id),
    )?;

    if ca.status() != AcademicCenterStatus::Active {
        return Err(PollApplicationError::AcademicCenterInactive(
            cmd.academic_center_id,
        ));
    }

    let is_permitted = auth_port
        .is_user_permitted_to_manage(cmd.created_by_id, &ca)
        .await?;

    if !is_permitted {
        return Err(PollApplicationError::UserNotPermittedInCa {
            user_id: cmd.created_by_id,
            academic_center_id: cmd.academic_center_id,
        });
    }

    let poll = StudentPoll::new(
        cmd.academic_center_id,
        cmd.created_by_id,
        cmd.title,
        cmd.description,
        cmd.options,
        cmd.starts_at,
        cmd.ends_at,
        cmd.target_audience,
        cmd.vote_modality,
    )?;

    poll_repo.save(&poll).await?;

    Ok(poll)
}
