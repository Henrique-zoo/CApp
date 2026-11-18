//! Agregado raiz que controla o ciclo de vida de uma eleição.

use chrono::{DateTime, Timelike, Utc};

use crate::modules::elections::domain::{
    entities::{
        closure::ElectionClosure,
        manifest::Manifest,
        result::{ElectionResult, ResultReason, ResultStatus},
    },
    rules::result_consistency::{CountConsistency, validate_result_counts},
    value_objects::{
        calendar_origin::CalendarOrigin,
        election_period::ElectionPeriod,
        fingerprint::Fingerprint,
        identifiers::{AcademicCenterId, ActorId, ElectionId},
        schedule_change::ScheduleChange,
        voter_count::VoterCount,
    },
};

/// Estado operacional da eleição.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElectionStatus {
    /// Configuração e preparação das autoridades.
    Preparing,

    /// Recebimento de votos.
    Open,

    /// Votação encerrada, aguardando ou concluindo a apuração.
    Closed,

    /// Cancelamento administrativo.
    Cancelled,
}

/// Situação independente da apuração eleitoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TallyStatus {
    /// Apuração ainda não publicada.
    Pending,

    /// Faltam contribuições válidas de autoridades.
    Incomplete,

    /// Apuração concluída, válida ou anulada.
    Final,
}

/// Versão do contrato de registro de cancelamento eleitoral.
pub const ELECTION_CANCELLATION_VERSION: u8 = 1;

/// Registro imutável de cancelamento administrativo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectionCancellation {
    election_id: ElectionId,
    manifest_hash: Option<Fingerprint>,
    schedule_hash: Option<Fingerprint>,
    head_hash: Option<Fingerprint>,
    voter_count: VoterCount,
    requested_by: ActorId,
    cancelled_at: DateTime<Utc>,
    reason: String,
}

impl ElectionCancellation {
    /// Retorna o identificador da eleição.
    pub const fn election_id(&self) -> ElectionId {
        self.election_id
    }

    /// Retorna o hash do manifesto, quando disponível.
    pub const fn manifest_hash(&self) -> Option<&Fingerprint> {
        self.manifest_hash.as_ref()
    }

    /// Retorna o hash do calendário vigente, quando disponível.
    pub const fn schedule_hash(&self) -> Option<&Fingerprint> {
        self.schedule_hash.as_ref()
    }

    /// Retorna o hash do último registro, quando disponível.
    pub const fn head_hash(&self) -> Option<&Fingerprint> {
        self.head_hash.as_ref()
    }

    /// Retorna a quantidade de eleitores distintos.
    pub const fn voter_count(&self) -> VoterCount {
        self.voter_count
    }

    /// Retorna o responsável pelo cancelamento.
    pub const fn requested_by(&self) -> ActorId {
        self.requested_by
    }

    /// Retorna o instante do cancelamento.
    pub const fn cancelled_at(&self) -> &DateTime<Utc> {
        &self.cancelled_at
    }

    /// Retorna o motivo do cancelamento.
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Retorna a versão do contrato.
    pub const fn version(&self) -> u8 {
        ELECTION_CANCELLATION_VERSION
    }

    /// Retorna o tipo do registro no contrato.
    pub const fn kind(&self) -> &'static str {
        "cancellation"
    }
}

/// Revisão de calendário aplicada ao agregado e seu hash resultante.
///
/// O hash é calculado externamente pelo codec. O domínio apenas preserva
/// o vínculo entre o registro aplicado e o hash produzido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedScheduleChange {
    change: ScheduleChange,
    resulting_schedule_hash: Fingerprint,
    was_open: bool,
}

impl AppliedScheduleChange {
    /// Retorna o registro da alteração de calendário.
    pub const fn schedule_change(&self) -> &ScheduleChange {
        &self.change
    }

    /// Retorna o hash do calendário produzido por esta revisão.
    pub const fn resulting_schedule_hash(&self) -> &Fingerprint {
        &self.resulting_schedule_hash
    }

    /// Retorna o hash do calendário anterior à revisão.
    pub fn previous_schedule_hash(&self) -> &Fingerprint {
        self.change.previous_schedule_hash()
    }
}

/// Agregado raiz responsável pelas invariantes do ciclo de vida eleitoral.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Election {
    id: ElectionId,
    academic_center_id: AcademicCenterId,
    status: ElectionStatus,
    tally_status: TallyStatus,
    calendar_origin: CalendarOrigin,
    calendar_origin_hash: Fingerprint,
    current_period: ElectionPeriod,
    opened_at: Option<DateTime<Utc>>,
    current_schedule_hash: Fingerprint,
    schedule_revision: u32,
    schedule_changes: Vec<AppliedScheduleChange>,
    manifest: Option<Manifest>,
    manifest_hash: Option<Fingerprint>,
    cancellation: Option<ElectionCancellation>,
    closure: Option<ElectionClosure>,
    tally_result: Option<ElectionResult>,
}

/// Erros encontrados ao construir uma eleição.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidElection {
    /// A origem do calendário pertence a outra eleição.
    #[error("A origem do calendário pertence a outra eleição")]
    CalendarOriginElectionMismatch,

    /// O manifesto pertence a outra eleição.
    #[error("O manifesto pertence a outra eleição")]
    ManifestElectionMismatch,

    /// O manifesto pertence a outro Centro Acadêmico.
    #[error("O manifesto pertence a outro Centro Acadêmico")]
    ManifestAcademicCenterMismatch,

    /// O manifesto referencia outra origem de calendário.
    #[error("O manifesto referencia outra origem de calendário")]
    ManifestCalendarOriginMismatch,

    /// Já existe um manifesto publicado para esta eleição.
    #[error("O manifesto já foi publicado")]
    ManifestAlreadyPublished,

    /// O manifesto só pode ser publicado durante a preparação.
    #[error("O manifesto só pode ser publicado durante a preparação")]
    ManifestPublicationAfterPreparation,
}

/// Erros encontrados durante uma transição do ciclo de vida eleitoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ElectionTransitionError {
    /// A operação não é permitida no estado operacional atual.
    #[error("A operação não é permitida no estado atual da eleição")]
    InvalidState,

    /// A apuração exige uma eleição encerrada.
    #[error("A eleição precisa estar encerrada para publicar a apuração")]
    TallyBeforeClosure,

    /// O resultado pertence a outra eleição.
    #[error("O resultado pertence a outra eleição")]
    TallyElectionMismatch,

    /// O resultado referencia outro manifesto.
    #[error("O resultado referencia um manifesto diferente")]
    TallyManifestMismatch,

    /// O resultado referencia outro encerramento.
    #[error("O resultado referencia um encerramento diferente")]
    TallyClosureMismatch,

    /// O resultado informa quantidade diferente da fotografia de encerramento.
    #[error("A quantidade de eleitores diverge do encerramento")]
    TallyVoterCountMismatch,

    /// As opções ou sua ordem divergem do manifesto publicado.
    #[error("As opções contabilizadas divergem do manifesto")]
    TallyOptionsMismatch,

    /// Uma autoridade pendente não pertence ao manifesto publicado.
    #[error("Uma autoridade pendente não pertence ao manifesto")]
    TallyAuthorityMismatch,

    /// O motivo informado não corresponde à consistência das contagens.
    #[error("O motivo do resultado diverge da consistência das contagens")]
    TallyReasonMismatch,

    /// A apuração já foi finalizada.
    #[error("A apuração já foi finalizada e não pode ser substituída")]
    TallyAlreadyFinal,

    /// Resultado administrativo cancelado não é uma publicação de apuração.
    #[error("Resultado de cancelamento não pode ser publicado como apuração")]
    CancelledResultCannotBePublished,

    /// A revisão pertence a outra eleição.
    #[error("A revisão pertence a outra eleição")]
    ScheduleElectionMismatch,

    /// A revisão não referencia o hash de manifesto vigente.
    #[error("A revisão referencia um manifesto diferente")]
    ScheduleManifestMismatch,

    /// A revisão não referencia o calendário vigente.
    #[error("A revisão não referencia o hash de calendário vigente")]
    PreviousScheduleHashMismatch,

    /// A revisão não é a próxima da sequência.
    #[error("A revisão não é sequencial")]
    InvalidScheduleRevision,

    /// O novo encerramento não é posterior ao encerramento atual.
    #[error("O novo encerramento deve ser posterior ao encerramento atual")]
    EndMustBeExtended,

    /// A prorrogação tentou modificar o início da votação.
    #[error("O início da votação não pode ser alterado após sua abertura")]
    StartCannotChange,

    /// A prorrogação ocorreu depois do prazo vigente.
    #[error("A votação só pode ser prorrogada antes do prazo vigente")]
    ExtensionAfterDeadline,

    /// A revisão foi registrada antes da abertura da votação.
    #[error("A revisão não pode anteceder a abertura da votação")]
    ScheduleChangeBeforeOpening,

    /// O cancelamento foi registrado antes da abertura da votação.
    #[error("O cancelamento não pode anteceder a abertura da votação")]
    CancellationBeforeOpening,

    /// O instante de referência não possui precisão de segundos.
    #[error("O instante deve ter precisão exata de segundos")]
    SubsecondPrecision,

    /// A nova revisão não possui hash diferente do anterior.
    #[error("O novo hash de calendário deve ser diferente do anterior")]
    UnchangedScheduleHash,

    /// A eleição precisa ter manifesto publicado para ser aberta.
    #[error("A eleição não possui manifesto publicado")]
    ManifestRequired,

    /// A abertura não pode ocorrer antes do início vigente.
    #[error("A abertura não pode ocorrer antes do início do calendário vigente")]
    OpeningBeforeStart,

    /// A abertura deve ocorrer antes do encerramento vigente.
    #[error("A abertura deve ocorrer antes do encerramento vigente")]
    OpeningAtOrAfterEnd,

    /// O encerramento não pode ocorrer antes do prazo vigente.
    #[error("A eleição não pode ser encerrada antes do prazo vigente")]
    ClosingBeforeDeadline,

    /// O motivo do cancelamento não possui tamanho permitido.
    #[error("O motivo do cancelamento deve possuir entre 1 e 2000 caracteres")]
    InvalidCancellationReasonLength,

    /// O cancelamento ocorreu antes do encerramento já registrado.
    #[error("O cancelamento não pode anteceder o encerramento da eleição")]
    CancellationBeforeClosure,

    /// Os dados do cancelamento divergem da fotografia de encerramento.
    #[error("Os dados do cancelamento divergem do encerramento registrado")]
    CancellationSnapshotMismatch,
}

impl Election {
    /// Cria uma eleição em preparação com seu calendário inicial validado.
    pub fn try_new(
        id: ElectionId,
        academic_center_id: AcademicCenterId,
        calendar_origin: CalendarOrigin,
        calendar_origin_hash: Fingerprint,
    ) -> Result<Self, InvalidElection> {
        if calendar_origin.election_id() != id {
            return Err(InvalidElection::CalendarOriginElectionMismatch);
        }

        let current_period = calendar_origin.period().clone();

        Ok(Self {
            id,
            academic_center_id,
            status: ElectionStatus::Preparing,
            tally_status: TallyStatus::Pending,
            calendar_origin,
            calendar_origin_hash: calendar_origin_hash.clone(),
            current_period,
            opened_at: None,
            current_schedule_hash: calendar_origin_hash,
            schedule_revision: 0,
            schedule_changes: Vec::new(),
            manifest: None,
            manifest_hash: None,
            cancellation: None,
            closure: None,
            tally_result: None,
        })
    }

    /// Retorna o identificador da eleição.
    pub const fn id(&self) -> ElectionId {
        self.id
    }

    /// Retorna o Centro Acadêmico responsável.
    pub const fn academic_center_id(&self) -> AcademicCenterId {
        self.academic_center_id
    }

    /// Retorna o estado operacional.
    pub const fn status(&self) -> ElectionStatus {
        self.status
    }

    /// Retorna a situação da apuração.
    pub const fn tally_status(&self) -> TallyStatus {
        self.tally_status
    }

    /// Retorna o calendário original.
    pub const fn calendar_origin(&self) -> &CalendarOrigin {
        &self.calendar_origin
    }

    /// Retorna o hash do calendário original.
    pub const fn calendar_origin_hash(&self) -> &Fingerprint {
        &self.calendar_origin_hash
    }

    /// Retorna o período atualmente vigente.
    pub const fn current_period(&self) -> &ElectionPeriod {
        &self.current_period
    }

    /// Retorna o instante em que a votação foi aberta.
    pub const fn opened_at(&self) -> Option<DateTime<Utc>> {
        self.opened_at
    }

    /// Retorna o hash da revisão de calendário vigente.
    pub const fn current_schedule_hash(&self) -> &Fingerprint {
        &self.current_schedule_hash
    }

    /// Retorna o manifesto publicado, quando disponível.
    pub const fn manifest(&self) -> Option<&Manifest> {
        self.manifest.as_ref()
    }

    /// Retorna o registro de cancelamento, quando existente.
    pub const fn cancellation(&self) -> Option<&ElectionCancellation> {
        self.cancellation.as_ref()
    }

    /// Retorna o registro imutável de encerramento, quando existente.
    pub const fn closure(&self) -> Option<&ElectionClosure> {
        self.closure.as_ref()
    }

    /// Retorna o último resultado publicado da apuração, quando existente.
    pub const fn tally_result(&self) -> Option<&ElectionResult> {
        self.tally_result.as_ref()
    }

    /// Retorna o hash do manifesto publicado, quando disponível.
    pub const fn manifest_hash(&self) -> Option<&Fingerprint> {
        self.manifest_hash.as_ref()
    }

    /// Retorna a última revisão do calendário.
    ///
    /// O calendário original corresponde à revisão zero.
    pub const fn schedule_revision(&self) -> u32 {
        self.schedule_revision
    }

    /// Retorna o histórico ordenado das revisões do calendário.
    pub fn schedule_changes(&self) -> &[AppliedScheduleChange] {
        &self.schedule_changes
    }

    /// Publica o manifesto e seu hash calculado pelo codec.
    pub fn publish_manifest(
        &mut self,
        manifest: Manifest,
        manifest_hash: Fingerprint,
    ) -> Result<(), InvalidElection> {
        if self.status != ElectionStatus::Preparing {
            return Err(InvalidElection::ManifestPublicationAfterPreparation);
        }

        if self.manifest.is_some() {
            return Err(InvalidElection::ManifestAlreadyPublished);
        }

        if manifest.election_id() != self.id {
            return Err(InvalidElection::ManifestElectionMismatch);
        }

        if manifest.academic_center_id() != self.academic_center_id {
            return Err(InvalidElection::ManifestAcademicCenterMismatch);
        }

        if manifest.calendar_origin_hash() != &self.calendar_origin_hash {
            return Err(InvalidElection::ManifestCalendarOriginMismatch);
        }

        self.manifest = Some(manifest);
        self.manifest_hash = Some(manifest_hash);

        Ok(())
    }

    /// Abre a eleição no instante fornecido pela camada de aplicação.
    ///
    /// A abertura exige manifesto publicado e ocorre somente dentro do
    /// período vigente, respeitando o intervalo [starts_at, ends_at).
    pub fn open(&mut self, opened_at: DateTime<Utc>) -> Result<(), ElectionTransitionError> {
        if self.status != ElectionStatus::Preparing {
            return Err(ElectionTransitionError::InvalidState);
        }

        if self.manifest.is_none() {
            return Err(ElectionTransitionError::ManifestRequired);
        }

        if opened_at.nanosecond() != 0 {
            return Err(ElectionTransitionError::SubsecondPrecision);
        }

        if &opened_at < self.current_period.starts_at() {
            return Err(ElectionTransitionError::OpeningBeforeStart);
        }

        if &opened_at >= self.current_period.ends_at() {
            return Err(ElectionTransitionError::OpeningAtOrAfterEnd);
        }

        self.opened_at = Some(opened_at);
        self.status = ElectionStatus::Open;

        Ok(())
    }

    /// Cancela administrativamente uma eleição ainda não terminal.
    ///
    /// O cancelamento preserva o manifesto, o calendário, o histórico de
    /// revisões e eventual registro de encerramento. A autorização do ator
    /// deve ser verificada pela camada de aplicação.
    pub fn cancel(
        &mut self,
        requested_by: ActorId,
        cancelled_at: DateTime<Utc>,
        head_hash: Option<Fingerprint>,
        voter_count: VoterCount,
        reason: impl Into<String>,
    ) -> Result<&ElectionCancellation, ElectionTransitionError> {
        if self.status == ElectionStatus::Cancelled {
            return self
                .cancellation
                .as_ref()
                .ok_or(ElectionTransitionError::InvalidState);
        }

        if self.tally_status == TallyStatus::Final {
            return Err(ElectionTransitionError::InvalidState);
        }

        if cancelled_at.nanosecond() != 0 {
            return Err(ElectionTransitionError::SubsecondPrecision);
        }

        if self
            .opened_at
            .is_some_and(|opened_at| cancelled_at < opened_at)
        {
            return Err(ElectionTransitionError::CancellationBeforeOpening);
        }

        if let Some(closure) = &self.closure {
            if cancelled_at < *closure.closed_at() {
                return Err(ElectionTransitionError::CancellationBeforeClosure);
            }

            if head_hash.as_ref() != Some(closure.head_hash())
                || voter_count != closure.voter_count()
            {
                return Err(ElectionTransitionError::CancellationSnapshotMismatch);
            }
        }

        let reason = reason.into();
        let reason_length = reason.chars().count();

        if !(1..=2000).contains(&reason_length) {
            return Err(ElectionTransitionError::InvalidCancellationReasonLength);
        }

        self.cancellation = Some(ElectionCancellation {
            election_id: self.id,
            manifest_hash: self.manifest_hash.clone(),
            schedule_hash: Some(self.current_schedule_hash.clone()),
            head_hash,
            voter_count,
            requested_by,
            cancelled_at,
            reason,
        });
        self.status = ElectionStatus::Cancelled;

        self.cancellation
            .as_ref()
            .ok_or(ElectionTransitionError::InvalidState)
    }

    /// Encerra a votação e registra o snapshot imutável de encerramento.
    ///
    /// O hash da última submissão e a quantidade de eleitores distintos
    /// são fornecidos pela camada de aplicação, que deve obtê-los
    /// atomicamente com o encerramento da votação.
    ///
    /// Chamadas repetidas após o encerramento retornam o registro existente,
    /// sem substituir seus dados.
    pub fn close(
        &mut self,
        closed_at: DateTime<Utc>,
        head_hash: Fingerprint,
        voter_count: VoterCount,
    ) -> Result<&ElectionClosure, ElectionTransitionError> {
        if self.status == ElectionStatus::Closed {
            return self
                .closure
                .as_ref()
                .ok_or(ElectionTransitionError::InvalidState);
        }

        if self.status != ElectionStatus::Open {
            return Err(ElectionTransitionError::InvalidState);
        }

        let manifest_hash = self
            .manifest_hash
            .clone()
            .ok_or(ElectionTransitionError::ManifestRequired)?;

        if closed_at.nanosecond() != 0 {
            return Err(ElectionTransitionError::SubsecondPrecision);
        }

        if closed_at < *self.current_period.ends_at() {
            return Err(ElectionTransitionError::ClosingBeforeDeadline);
        }

        let closure = ElectionClosure::try_new(
            self.id,
            manifest_hash,
            self.current_schedule_hash.clone(),
            head_hash,
            voter_count,
            closed_at,
        )
        .map_err(|_| ElectionTransitionError::SubsecondPrecision)?;

        self.closure = Some(closure);
        self.status = ElectionStatus::Closed;

        self.closure
            .as_ref()
            .ok_or(ElectionTransitionError::InvalidState)
    }

    /// Publica ou atualiza o resultado da apuração eleitoral.
    ///
    /// O hash do encerramento é calculado externamente pelo codec.
    /// O agregado valida as referências e as invariantes de domínio,
    /// sem executar operações criptográficas.
    ///
    /// Resultados incompletos podem ser atualizados até a finalização.
    /// Uma apuração final é imutável; uma repetição idêntica é idempotente.
    pub fn publish_tally(
        &mut self,
        result: ElectionResult,
        closure_hash: &Fingerprint,
    ) -> Result<&ElectionResult, ElectionTransitionError> {
        if self.status != ElectionStatus::Closed {
            return Err(ElectionTransitionError::TallyBeforeClosure);
        }

        if result.status() == ResultStatus::Cancelled {
            return Err(ElectionTransitionError::CancelledResultCannotBePublished);
        }

        let closure = self
            .closure
            .as_ref()
            .ok_or(ElectionTransitionError::TallyBeforeClosure)?;

        let manifest = self
            .manifest
            .as_ref()
            .ok_or(ElectionTransitionError::ManifestRequired)?;

        let manifest_hash = self
            .manifest_hash
            .as_ref()
            .ok_or(ElectionTransitionError::ManifestRequired)?;

        if result.election_id() != self.id || closure.election_id() != self.id {
            return Err(ElectionTransitionError::TallyElectionMismatch);
        }

        if result.manifest_hash() != Some(manifest_hash) || closure.manifest_hash() != manifest_hash
        {
            return Err(ElectionTransitionError::TallyManifestMismatch);
        }

        if result.closure_hash() != Some(closure_hash) {
            return Err(ElectionTransitionError::TallyClosureMismatch);
        }

        if result.voter_count() != closure.voter_count() {
            return Err(ElectionTransitionError::TallyVoterCountMismatch);
        }

        if result.status() == ResultStatus::Incomplete
            && result.missing_authorities().iter().any(|missing| {
                !manifest
                    .authorities()
                    .iter()
                    .any(|authority| authority.id() == *missing)
            })
        {
            return Err(ElectionTransitionError::TallyAuthorityMismatch);
        }

        match result.status() {
            ResultStatus::Valid => {
                let counts = result
                    .counts()
                    .ok_or(ElectionTransitionError::TallyOptionsMismatch)?;

                match validate_result_counts(manifest.options(), counts, closure.voter_count()) {
                    Ok(CountConsistency::Consistent) => {}
                    Ok(CountConsistency::SumMismatch { .. }) => {
                        return Err(ElectionTransitionError::TallyReasonMismatch);
                    }
                    Err(_) => return Err(ElectionTransitionError::TallyOptionsMismatch),
                }
            }
            ResultStatus::Annulled => match result.reason() {
                Some(ResultReason::AggregateSumMismatch) => {
                    let counts = result
                        .counts()
                        .ok_or(ElectionTransitionError::TallyOptionsMismatch)?;

                    match validate_result_counts(manifest.options(), counts, closure.voter_count())
                    {
                        Ok(CountConsistency::SumMismatch { .. }) => {}
                        Ok(CountConsistency::Consistent) => {
                            return Err(ElectionTransitionError::TallyReasonMismatch);
                        }
                        Err(_) => {
                            return Err(ElectionTransitionError::TallyOptionsMismatch);
                        }
                    }
                }
                Some(ResultReason::TallyOutOfRange) => {
                    if result.counts().is_some() {
                        return Err(ElectionTransitionError::TallyReasonMismatch);
                    }
                }
                _ => return Err(ElectionTransitionError::TallyReasonMismatch),
            },
            ResultStatus::Incomplete => {}
            ResultStatus::Cancelled => {
                return Err(ElectionTransitionError::CancelledResultCannotBePublished);
            }
        }

        if self.tally_status == TallyStatus::Final {
            if self.tally_result.as_ref() == Some(&result) {
                return self
                    .tally_result
                    .as_ref()
                    .ok_or(ElectionTransitionError::TallyAlreadyFinal);
            }

            return Err(ElectionTransitionError::TallyAlreadyFinal);
        }

        self.tally_status = match result.status() {
            ResultStatus::Incomplete => TallyStatus::Incomplete,
            ResultStatus::Valid | ResultStatus::Annulled => TallyStatus::Final,
            ResultStatus::Cancelled => {
                return Err(ElectionTransitionError::CancelledResultCannotBePublished);
            }
        };

        self.tally_result = Some(result);

        self.tally_result
            .as_ref()
            .ok_or(ElectionTransitionError::InvalidState)
    }

    /// Aplica uma revisão de calendário validando estado, sequência e cadeia.
    ///
    /// O hash da revisão é calculado externamente pelo codec.
    pub fn change_schedule(
        &mut self,
        change: ScheduleChange,
        new_schedule_hash: Fingerprint,
    ) -> Result<(), ElectionTransitionError> {
        if !matches!(
            self.status,
            ElectionStatus::Preparing | ElectionStatus::Open
        ) {
            return Err(ElectionTransitionError::InvalidState);
        }

        if change.election_id() != self.id {
            return Err(ElectionTransitionError::ScheduleElectionMismatch);
        }

        if change.manifest_hash() != self.manifest_hash.as_ref() {
            return Err(ElectionTransitionError::ScheduleManifestMismatch);
        }

        if change.previous_schedule_hash() != &self.current_schedule_hash {
            return Err(ElectionTransitionError::PreviousScheduleHashMismatch);
        }

        let expected_revision = self
            .schedule_revision
            .checked_add(1)
            .ok_or(ElectionTransitionError::InvalidScheduleRevision)?;

        if change.revision() != expected_revision {
            return Err(ElectionTransitionError::InvalidScheduleRevision);
        }

        if new_schedule_hash == self.current_schedule_hash {
            return Err(ElectionTransitionError::UnchangedScheduleHash);
        }

        if change.changed_at().nanosecond() != 0 {
            return Err(ElectionTransitionError::SubsecondPrecision);
        }

        let new_period = change.period();

        if new_period.ends_at() <= self.current_period.ends_at() {
            return Err(ElectionTransitionError::EndMustBeExtended);
        }

        match self.status {
            ElectionStatus::Preparing => {
                // Durante a preparação, o contrato permite alterar a abertura.
            }
            ElectionStatus::Open => {
                if new_period.starts_at() != self.current_period.starts_at() {
                    return Err(ElectionTransitionError::StartCannotChange);
                }

                let opened_at = self
                    .opened_at
                    .ok_or(ElectionTransitionError::InvalidState)?;

                if change.changed_at() < &opened_at {
                    return Err(ElectionTransitionError::ScheduleChangeBeforeOpening);
                }

                if change.changed_at() >= self.current_period.ends_at() {
                    return Err(ElectionTransitionError::ExtensionAfterDeadline);
                }
            }
            ElectionStatus::Closed | ElectionStatus::Cancelled => {
                return Err(ElectionTransitionError::InvalidState);
            }
        }

        self.current_period = new_period.clone();
        self.current_schedule_hash = new_schedule_hash.clone();
        self.schedule_revision = change.revision();
        self.schedule_changes.push(AppliedScheduleChange {
            change,
            resulting_schedule_hash: new_schedule_hash,
            was_open: self.status == ElectionStatus::Open,
        });

        Ok(())
    }
}

#[cfg(test)]
#[path = "election/tests.rs"]
mod tests;

impl Election {
    /// Produz uma fotografia completa do estado persistido do agregado.
    ///
    /// O snapshot preserva os metadados internos necessários para validar
    /// posteriormente o histórico e reconstituir o mesmo estado.
    pub fn to_snapshot(&self) -> super::election_snapshot::ElectionSnapshot {
        use super::election_snapshot::{
            AppliedScheduleChangeSnapshot, ElectionCancellationSnapshot, ElectionSnapshot,
        };

        let schedule_changes = self
            .schedule_changes
            .iter()
            .map(|applied| AppliedScheduleChangeSnapshot {
                change: applied.change.clone(),
                resulting_schedule_hash: applied.resulting_schedule_hash.clone(),
                was_open: applied.was_open,
            })
            .collect();

        let cancellation = self
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
            });

        ElectionSnapshot {
            id: self.id,
            academic_center_id: self.academic_center_id,
            status: self.status,
            tally_status: self.tally_status,
            calendar_origin: self.calendar_origin.clone(),
            calendar_origin_hash: self.calendar_origin_hash.clone(),
            current_period: self.current_period.clone(),
            opened_at: self.opened_at,
            current_schedule_hash: self.current_schedule_hash.clone(),
            schedule_revision: self.schedule_revision,
            schedule_changes,
            manifest: self.manifest.clone(),
            manifest_hash: self.manifest_hash.clone(),
            cancellation,
            closure: self.closure.clone(),
            tally_result: self.tally_result.clone(),
        }
    }

    /// Reconstitui uma eleição persistida validando suas invariantes.
    ///
    /// O snapshot é tratado como entrada não confiável: suas referências,
    /// histórico e estados são verificados antes da criação do agregado.
    pub fn try_reconstitute(
        snapshot: super::election_snapshot::ElectionSnapshot,
    ) -> Result<Self, super::election_snapshot::ElectionReconstitutionError> {
        use super::election_snapshot::ElectionReconstitutionError as ReconstitutionError;

        if snapshot.calendar_origin.election_id() != snapshot.id {
            return Err(ReconstitutionError::CalendarOriginElectionMismatch);
        }

        if snapshot.manifest.is_some() != snapshot.manifest_hash.is_some() {
            return Err(ReconstitutionError::ManifestHashMismatch);
        }

        if let Some(manifest) = &snapshot.manifest
            && (manifest.election_id() != snapshot.id
                || manifest.academic_center_id() != snapshot.academic_center_id
                || manifest.calendar_origin_hash() != &snapshot.calendar_origin_hash)
        {
            return Err(ReconstitutionError::ManifestReferenceMismatch);
        }

        if snapshot.schedule_revision as usize != snapshot.schedule_changes.len()
            || snapshot.schedule_revision > crate::modules::elections::domain::value_objects::schedule_change::MAX_SCHEDULE_REVISION
        {
            return Err(ReconstitutionError::ScheduleRevisionMismatch);
        }

        let mut expected_previous_hash = snapshot.calendar_origin_hash.clone();
        let mut expected_period = snapshot.calendar_origin.period().clone();

        for (index, applied) in snapshot.schedule_changes.iter().enumerate() {
            let change = &applied.change;
            let expected_revision = (index + 1) as u32;

            if change.election_id() != snapshot.id
                || change.revision() != expected_revision
                || change.previous_schedule_hash() != &expected_previous_hash
            {
                return Err(ReconstitutionError::InvalidScheduleHistory);
            }

            if change.manifest_hash() != snapshot.manifest_hash.as_ref()
                && change.manifest_hash().is_some()
            {
                return Err(ReconstitutionError::InvalidScheduleHistory);
            }

            if applied.resulting_schedule_hash == expected_previous_hash
                || change.period().ends_at() <= expected_period.ends_at()
            {
                return Err(ReconstitutionError::InvalidScheduleHistory);
            }

            if applied.was_open {
                let opened_at = snapshot
                    .opened_at
                    .ok_or(ReconstitutionError::InvalidScheduleHistory)?;

                if change.period().starts_at() != expected_period.starts_at()
                    || change.changed_at() < &opened_at
                    || change.changed_at() >= expected_period.ends_at()
                {
                    return Err(ReconstitutionError::InvalidScheduleHistory);
                }
            }

            expected_previous_hash = applied.resulting_schedule_hash.clone();
            expected_period = change.period().clone();
        }

        if expected_previous_hash != snapshot.current_schedule_hash {
            return Err(ReconstitutionError::ScheduleHashChainMismatch);
        }

        if expected_period != snapshot.current_period {
            return Err(ReconstitutionError::CurrentPeriodMismatch);
        }

        if snapshot.opened_at.is_some_and(|opened_at| {
            opened_at.nanosecond() != 0 || !snapshot.current_period.contains(&opened_at)
        }) {
            return Err(ReconstitutionError::InvalidOpening);
        }

        match snapshot.status {
            ElectionStatus::Preparing => {
                if snapshot.opened_at.is_some()
                    || snapshot.closure.is_some()
                    || snapshot.cancellation.is_some()
                    || snapshot.tally_status != TallyStatus::Pending
                    || snapshot.tally_result.is_some()
                {
                    return Err(ReconstitutionError::InvalidLifecycleState);
                }
            }
            ElectionStatus::Open => {
                if snapshot.manifest.is_none()
                    || snapshot.opened_at.is_none()
                    || snapshot.closure.is_some()
                    || snapshot.cancellation.is_some()
                    || snapshot.tally_status != TallyStatus::Pending
                    || snapshot.tally_result.is_some()
                {
                    return Err(ReconstitutionError::InvalidLifecycleState);
                }
            }
            ElectionStatus::Closed => {
                if snapshot.manifest.is_none()
                    || snapshot.opened_at.is_none()
                    || snapshot.closure.is_none()
                    || snapshot.cancellation.is_some()
                {
                    return Err(ReconstitutionError::InvalidLifecycleState);
                }
            }
            ElectionStatus::Cancelled => {
                if snapshot.cancellation.is_none() || snapshot.tally_status == TallyStatus::Final {
                    return Err(ReconstitutionError::InvalidLifecycleState);
                }
            }
        }

        if let Some(closure) = &snapshot.closure
            && (snapshot.manifest_hash.as_ref() != Some(closure.manifest_hash())
                || closure.election_id() != snapshot.id
                || closure.schedule_hash() != &snapshot.current_schedule_hash
                || closure.closed_at() < snapshot.current_period.ends_at()
                || snapshot.opened_at.is_none())
        {
            return Err(ReconstitutionError::InvalidClosure);
        }

        let cancellation = snapshot.cancellation.map(|record| ElectionCancellation {
            election_id: record.election_id,
            manifest_hash: record.manifest_hash,
            schedule_hash: record.schedule_hash,
            head_hash: record.head_hash,
            voter_count: record.voter_count,
            requested_by: record.requested_by,
            cancelled_at: record.cancelled_at,
            reason: record.reason,
        });

        if let Some(record) = &cancellation {
            if record.election_id != snapshot.id
                || record.manifest_hash != snapshot.manifest_hash
                || record.schedule_hash.as_ref() != Some(&snapshot.current_schedule_hash)
                || record.cancelled_at.nanosecond() != 0
                || !(1..=2_000).contains(&record.reason.chars().count())
                || snapshot
                    .opened_at
                    .is_some_and(|opened_at| record.cancelled_at < opened_at)
            {
                return Err(ReconstitutionError::InvalidCancellation);
            }

            if let Some(closure) = &snapshot.closure
                && (record.cancelled_at < *closure.closed_at()
                    || record.head_hash.as_ref() != Some(closure.head_hash())
                    || record.voter_count != closure.voter_count())
            {
                return Err(ReconstitutionError::InvalidCancellation);
            }
        }

        if snapshot.status == ElectionStatus::Cancelled && cancellation.is_none() {
            return Err(ReconstitutionError::InvalidCancellation);
        }

        let schedule_changes = snapshot
            .schedule_changes
            .into_iter()
            .map(|applied| AppliedScheduleChange {
                change: applied.change,
                resulting_schedule_hash: applied.resulting_schedule_hash,
                was_open: applied.was_open,
            })
            .collect();

        let mut election = Self {
            id: snapshot.id,
            academic_center_id: snapshot.academic_center_id,
            status: snapshot.status,
            tally_status: TallyStatus::Pending,
            calendar_origin: snapshot.calendar_origin,
            calendar_origin_hash: snapshot.calendar_origin_hash,
            current_period: snapshot.current_period,
            opened_at: snapshot.opened_at,
            current_schedule_hash: snapshot.current_schedule_hash,
            schedule_revision: snapshot.schedule_revision,
            schedule_changes,
            manifest: snapshot.manifest,
            manifest_hash: snapshot.manifest_hash,
            cancellation,
            closure: snapshot.closure,
            tally_result: None,
        };

        match (snapshot.tally_status, snapshot.tally_result) {
            (TallyStatus::Pending, None) => {}
            (TallyStatus::Pending, Some(_)) => {
                return Err(ReconstitutionError::InvalidTallyState);
            }
            (TallyStatus::Incomplete, Some(result)) | (TallyStatus::Final, Some(result)) => {
                if election.closure.is_none()
                    || election.manifest.is_none()
                    || (election.status == ElectionStatus::Cancelled
                        && snapshot.tally_status == TallyStatus::Final)
                {
                    return Err(ReconstitutionError::InvalidTallyState);
                }

                let closure_hash = result
                    .closure_hash()
                    .ok_or(ReconstitutionError::InvalidTallyResult)?
                    .clone();

                election.status = ElectionStatus::Closed;

                election
                    .publish_tally(result, &closure_hash)
                    .map_err(|_| ReconstitutionError::InvalidTallyResult)?;

                if election.tally_status != snapshot.tally_status {
                    return Err(ReconstitutionError::InvalidTallyState);
                }

                if snapshot.status == ElectionStatus::Cancelled {
                    election.status = ElectionStatus::Cancelled;
                }
            }
            (TallyStatus::Incomplete | TallyStatus::Final, None) => {
                return Err(ReconstitutionError::InvalidTallyState);
            }
        }

        election.status = snapshot.status;

        Ok(election)
    }
}
