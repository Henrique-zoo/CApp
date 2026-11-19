//! Reconstituição por reprodução das mesmas operações usadas pelo agregado.
//!
//! O adaptador verifica assinaturas/digests antes de fornecer os objetos tipados.
//! Aqui são conferidos história, referências e estados, sem confiar em flags que
//! desativem validações de prorrogação ou em hashes extraídos do próprio resultado.

use super::*;
use crate::modules::elections::domain::entities::election_snapshot::{
    AppliedScheduleChangeSnapshot, ElectionCancellationSnapshot,
    ElectionReconstitutionError as Error, ElectionSnapshot,
};

impl Election {
    /// Exporta os dados necessários à reconstituição validada.
    ///
    /// Não é um DTO público. A configuração confidencial mantém a evidência de
    /// verificação, enquanto o adaptador persiste somente seu material público.
    pub fn to_snapshot(&self) -> ElectionSnapshot {
        ElectionSnapshot {
            id: self.id,
            academic_center_id: self.academic_center_id,
            modality: self.modality,
            configuration: self.configuration.clone(),
            status: self.status,
            tally_status: self.tally_status,
            calendar_origin: self.calendar_origin.clone(),
            calendar_origin_hash: self.calendar_origin_hash.clone(),
            current_period: self.current_period.clone(),
            opened_at: self.opened_at,
            opening_schedule_revision: self.opening_schedule_revision,
            current_schedule_hash: self.current_schedule_hash.clone(),
            schedule_revision: self.schedule_revision,
            schedule_changes: self
                .schedule_changes
                .iter()
                .map(|applied| AppliedScheduleChangeSnapshot {
                    change: applied.change.clone(),
                    resulting_schedule_hash: applied.resulting_schedule_hash.clone(),
                })
                .collect(),
            cancellation: self
                .cancellation
                .as_ref()
                .map(|record| ElectionCancellationSnapshot {
                    election_id: record.election_id,
                    manifest_hash: record.manifest_hash.clone(),
                    schedule_hash: record.schedule_hash.clone(),
                    head_hash: record.head_hash.clone(),
                    voter_count: record.voter_count,
                    requested_by: record.requested_by,
                    cancelled_at: record.cancelled_at,
                    reason: record.reason.clone(),
                }),
            closure: self.closure.clone(),
            closure_hash: self.closure_hash.clone(),
            tally_result: self.tally_result.clone(),
        }
    }

    /// Reproduz calendário, abertura, encerramento, apuração e cancelamento.
    ///
    /// Retorna erro se a configuração não pertence à modalidade/eleição, se o
    /// histórico viola uma transição ou se o estado final diverge da reprodução.
    /// A revisão de abertura também permite ordenar fatos no mesmo segundo.
    pub fn try_reconstitute(snapshot: ElectionSnapshot) -> Result<Self, Error> {
        let mut election = Self::try_new(
            snapshot.id,
            snapshot.academic_center_id,
            snapshot.modality,
            snapshot.calendar_origin.clone(),
            snapshot.calendar_origin_hash.clone(),
        )
        .map_err(|_| Error::CalendarOriginElectionMismatch)?;

        if snapshot
            .configuration
            .as_ref()
            .is_some_and(|c| c.modality() != snapshot.modality)
        {
            return Err(Error::InvalidConfiguration);
        }
        if snapshot.schedule_revision as usize != snapshot.schedule_changes.len() {
            return Err(Error::ScheduleRevisionMismatch);
        }
        if snapshot.opened_at.is_some() != snapshot.opening_schedule_revision.is_some()
            || snapshot
                .opening_schedule_revision
                .is_some_and(|revision| revision > snapshot.schedule_revision)
        {
            return Err(Error::InvalidOpening);
        }
        if let Some(ElectionConfiguration::Identified(options)) = &snapshot.configuration {
            election
                .configure_identified(options.clone())
                .map_err(|_| Error::InvalidConfiguration)?;
        }

        for (index, applied) in snapshot.schedule_changes.iter().enumerate() {
            if snapshot.opening_schedule_revision == Some(index as u32) {
                election.restore_configuration(&snapshot.configuration)?;
                election
                    .open(snapshot.opened_at.ok_or(Error::InvalidOpening)?)
                    .map_err(|_| Error::InvalidOpening)?;
            }
            if applied.change.manifest_hash().is_some() {
                election.restore_configuration(&snapshot.configuration)?;
            }
            election
                .change_schedule(
                    applied.change.clone(),
                    applied.resulting_schedule_hash.clone(),
                )
                .map_err(|_| Error::InvalidScheduleHistory)?;
        }
        if snapshot.opening_schedule_revision == Some(snapshot.schedule_revision) {
            election.restore_configuration(&snapshot.configuration)?;
            election
                .open(snapshot.opened_at.ok_or(Error::InvalidOpening)?)
                .map_err(|_| Error::InvalidOpening)?;
        }
        election.restore_configuration(&snapshot.configuration)?;
        if election.current_period != snapshot.current_period {
            return Err(Error::CurrentPeriodMismatch);
        }
        if election.current_schedule_hash != snapshot.current_schedule_hash {
            return Err(Error::ScheduleHashChainMismatch);
        }

        match (&snapshot.closure, &snapshot.closure_hash) {
            (Some(closure), Some(hash)) => {
                if closure.modality() != snapshot.modality {
                    return Err(Error::InvalidClosure);
                }
                let restored = election
                    .close_snapshot(
                        *closure.closed_at(),
                        closure.head_hash().cloned(),
                        closure.voter_count(),
                        hash.clone(),
                    )
                    .map_err(|_| Error::InvalidClosure)?;
                if restored != closure {
                    return Err(Error::InvalidClosure);
                }
            }
            (None, None) => {}
            _ => return Err(Error::InvalidClosure),
        }
        if let Some(result) = snapshot.tally_result {
            election
                .publish_tally(result)
                .map_err(|_| Error::InvalidTallyResult)?;
        }
        if election.tally_status != snapshot.tally_status {
            return Err(Error::InvalidTallyState);
        }
        if let Some(record) = snapshot.cancellation {
            if record.election_id != snapshot.id
                || record.manifest_hash.as_ref() != election.manifest_hash()
                || record.schedule_hash.as_ref() != Some(election.current_schedule_hash())
            {
                return Err(Error::InvalidCancellation);
            }
            election
                .cancel(
                    record.requested_by,
                    record.cancelled_at,
                    record.head_hash,
                    record.voter_count,
                    record.reason,
                )
                .map_err(|_| Error::InvalidCancellation)?;
        }
        if election.status != snapshot.status {
            return Err(Error::InvalidLifecycleState);
        }
        Ok(election)
    }

    /// Disponibiliza a configuração quando a sequência exige sua publicação.
    fn restore_configuration(
        &mut self,
        configuration: &Option<ElectionConfiguration>,
    ) -> Result<(), Error> {
        if self.configuration.is_none()
            && let Some(ElectionConfiguration::Confidential(verified)) = configuration
        {
            self.publish_manifest(verified.as_ref().clone())
                .map_err(|_| Error::ManifestReferenceMismatch)?;
        }
        Ok(())
    }
}
