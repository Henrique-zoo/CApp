//! Agregado raiz que controla o ciclo de vida de uma eleição.

use chrono::{DateTime, Timelike, Utc};

use crate::modules::elections::domain::{
    entities::{
        closure::{ElectionClosure, InvalidElectionClosure},
        configuration::{ElectionConfiguration, PreparationConfigurationChange},
        manifest::{Manifest, VerifiedManifest},
        result::{ElectionResult, ResultReason, ResultStatus},
        tally::VerifiedTally,
    },
    rules::{
        board_snapshot::{InvalidBoardSnapshot, validate_confidential_board_snapshot},
        result_consistency::{CountConsistency, validate_result_counts},
    },
    value_objects::{
        calendar_origin::CalendarOrigin,
        election_period::ElectionPeriod,
        fingerprint::Fingerprint,
        identifiers::{AcademicCenterId, ActorId, ElectionId},
        modality::ElectionModality,
        option::ElectionOption,
        options::ElectionOptions,
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
    pub fn manifest_hash(&self) -> Option<&Fingerprint> {
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
///
/// # Exemplo
///
/// Uma eleição identificada pode abrir sem manifesto ou autoridades ElGamal:
///
/// ```
/// use backend::modules::elections::domain::{entities::election::Election,
///     value_objects::{calendar_origin::CalendarOrigin, election_period::ElectionPeriod,
///     fingerprint::Fingerprint, identifiers::{AcademicCenterId, ElectionId, OptionId},
///     modality::ElectionModality, option::{ElectionOption, OptionKind}, options::ElectionOptions}};
/// use chrono::{TimeZone, Utc};
/// use uuid::Uuid;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let id = ElectionId::try_from_uuid(Uuid::from_u128(1))?;
/// let ca = AcademicCenterId::try_from_uuid(Uuid::from_u128(2))?;
/// let start = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
/// let end = Utc.with_ymd_and_hms(2026, 11, 20, 18, 0, 0).unwrap();
/// let origin = CalendarOrigin::new(id, ElectionPeriod::try_new(start, end)?);
/// let mut election = Election::try_new(id, ca, ElectionModality::Identified,
///     origin, Fingerprint::try_from_hex("a".repeat(64))?)?;
/// election.configure_identified(ElectionOptions::try_new(vec![
///     ElectionOption::try_new(OptionId::from_uuid(Uuid::from_u128(3)), OptionKind::Slate, "Aurora")?,
///     ElectionOption::try_new(OptionId::from_uuid(Uuid::nil()), OptionKind::Null, "Nulo")?,
/// ])?)?;
/// election.open(start)?;
/// assert!(election.manifest().is_none());
/// # Ok(()) }
/// ```
///
/// Desserialização direta não pode evitar os construtores:
///
/// ```compile_fail
/// use backend::modules::elections::domain::entities::election::Election;
/// let election: Election = serde_json::from_str("{}").unwrap();
/// ```
///
/// O estado operacional também não pode ser alterado diretamente:
///
/// ```compile_fail
/// use backend::modules::elections::domain::entities::election::{Election, ElectionStatus};
/// fn bypass(election: &mut Election) { election.status = ElectionStatus::Open; }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Election {
    id: ElectionId,
    academic_center_id: AcademicCenterId,
    modality: ElectionModality,
    status: ElectionStatus,
    tally_status: TallyStatus,
    calendar_origin: CalendarOrigin,
    calendar_origin_hash: Fingerprint,
    current_period: ElectionPeriod,
    opened_at: Option<DateTime<Utc>>,
    opening_schedule_revision: Option<u32>,
    opening_configuration_count: Option<usize>,
    current_schedule_hash: Fingerprint,
    schedule_revision: u32,
    schedule_changes: Vec<AppliedScheduleChange>,
    configuration: Option<ElectionConfiguration>,
    configuration_changes: Vec<PreparationConfigurationChange>,
    cancellation: Option<ElectionCancellation>,
    closure: Option<ElectionClosure>,
    closure_hash: Option<Fingerprint>,
    tally_result: Option<VerifiedTally>,
}

/// Erros encontrados ao construir uma eleição.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidElection {
    /// A configuração não corresponde à modalidade escolhida.
    #[error("Configuração incompatível com a modalidade eleitoral")]
    ModalityMismatch,
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

    /// Somente a preparação admite publicar ou invalidar configuração.
    #[error("A configuração só pode ser alterada durante a preparação")]
    ConfigurationAfterPreparation,
}

/// Erros encontrados durante uma transição do ciclo de vida eleitoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ElectionTransitionError {
    /// O agregado verificado diverge da fotografia ou do resultado.
    #[error("Agregado incompatível com o encerramento")]
    TallyAggregateMismatch,
    /// As contribuições verificadas não correspondem às autoridades necessárias.
    #[error("Contribuições incompatíveis com as autoridades do manifesto")]
    TallyContributionsMismatch,

    /// A modalidade ou a evidência não corresponde à configuração.
    #[error("Operação incompatível com a modalidade eleitoral")]
    ModalityMismatch,
    /// A configuração de opções ainda não foi concluída.
    #[error("A eleição não possui opções configuradas")]
    ConfigurationRequired,
    /// A abertura não pode ser antecipada por revisão de calendário.
    #[error("Uma revisão não pode antecipar a abertura")]
    StartCannotMoveEarlier,
    /// Os fatos devem respeitar a ordem temporal do histórico.
    #[error("O instante antecede uma alteração já registrada")]
    NonChronologicalOperation,

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

    /// A origem do quadro corresponde exatamente à contagem zero.
    #[error("A cabeça do quadro é incompatível com a contagem de participantes")]
    InvalidEmptyBoard,
}

impl Election {
    /// Cria uma eleição em preparação com modalidade e calendário inicial.
    ///
    /// Recusa calendário de outra eleição. A configuração pode ser concluída
    /// depois, sem exigir autoridades, cadastro ou chaves de eleitores aqui.
    pub fn try_new(
        id: ElectionId,
        academic_center_id: AcademicCenterId,
        modality: ElectionModality,
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
            modality,
            status: ElectionStatus::Preparing,
            tally_status: TallyStatus::Pending,
            calendar_origin,
            calendar_origin_hash: calendar_origin_hash.clone(),
            current_period,
            opened_at: None,
            opening_schedule_revision: None,
            opening_configuration_count: None,
            current_schedule_hash: calendar_origin_hash,
            schedule_revision: 0,
            schedule_changes: Vec::new(),
            configuration: None,
            configuration_changes: Vec::new(),
            cancellation: None,
            closure: None,
            closure_hash: None,
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
    pub fn manifest(&self) -> Option<&Manifest> {
        match &self.configuration {
            Some(ElectionConfiguration::Confidential(verified)) => Some(verified.manifest()),
            _ => None,
        }
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
    pub fn tally_result(&self) -> Option<&ElectionResult> {
        self.tally_result.as_ref().map(VerifiedTally::result)
    }

    /// Retorna o hash do manifesto publicado, quando disponível.
    pub fn manifest_hash(&self) -> Option<&Fingerprint> {
        match &self.configuration {
            Some(ElectionConfiguration::Confidential(verified)) => Some(verified.hash()),
            _ => None,
        }
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

    /// Retorna a modalidade escolhida na criação.
    pub const fn modality(&self) -> ElectionModality {
        self.modality
    }

    /// Retorna as opções prontas, sem exigir manifesto na modalidade identificada.
    pub fn options(&self) -> Option<&[ElectionOption]> {
        self.configuration
            .as_ref()
            .map(ElectionConfiguration::options)
    }

    /// Configura as opções identificadas durante a preparação.
    ///
    /// A aplicação fornece chapas aprovadas; a coleção valida limites e Nulo.
    /// Depois da abertura nenhuma configuração pode ser alterada.
    pub fn configure_identified(
        &mut self,
        options: ElectionOptions,
    ) -> Result<(), InvalidElection> {
        if self.status != ElectionStatus::Preparing {
            return Err(InvalidElection::ConfigurationAfterPreparation);
        }
        if self.modality != ElectionModality::Identified {
            return Err(InvalidElection::ModalityMismatch);
        }
        self.record_configuration(Some(ElectionConfiguration::Identified(options)));
        Ok(())
    }

    /// Invalida os artefatos derivados após edição da configuração em preparação.
    ///
    /// Preserva o material público anterior no histórico e impede abertura até
    /// nova configuração pronta. Não altera calendário, seus hashes ou revisões.
    /// A aplicação da #131 chama esta operação antes de persistir mudanças que
    /// invalidem opções, autoridades ou chaves; depois obtém novas verificações.
    ///
    /// Retorna `ConfigurationAfterPreparation` em eleição aberta ou terminal.
    /// Repetir a invalidação sem configuração vigente não cria outro registro.
    pub fn invalidate_configuration(&mut self) -> Result<(), InvalidElection> {
        if self.status != ElectionStatus::Preparing {
            return Err(InvalidElection::ConfigurationAfterPreparation);
        }
        self.record_configuration(None);
        Ok(())
    }

    /// Retorna as configurações e invalidações na ordem em que foram aplicadas.
    pub fn configuration_changes(&self) -> &[PreparationConfigurationChange] {
        &self.configuration_changes
    }

    /// Registra somente mudanças efetivas, após as validações específicas.
    fn record_configuration(&mut self, configuration: Option<ElectionConfiguration>) {
        if self.configuration == configuration {
            return;
        }
        self.configuration_changes
            .push(PreparationConfigurationChange::new(
                self.schedule_revision,
                configuration.clone(),
            ));
        self.configuration = configuration;
    }

    /// Publica o manifesto após verificação pelas portas de criptografia e codec.
    ///
    /// Recusa modalidade identificada, referências divergentes e publicação após
    /// preparação. Assinaturas, provas e digest não são calculados neste método.
    pub fn publish_manifest(&mut self, verified: VerifiedManifest) -> Result<(), InvalidElection> {
        if self.status != ElectionStatus::Preparing {
            return Err(InvalidElection::ManifestPublicationAfterPreparation);
        }
        if self.modality != ElectionModality::Confidential {
            return Err(InvalidElection::ModalityMismatch);
        }
        if self.configuration.is_some() {
            return Err(InvalidElection::ManifestAlreadyPublished);
        }
        let manifest = verified.manifest();
        if manifest.election_id() != self.id {
            return Err(InvalidElection::ManifestElectionMismatch);
        }
        if manifest.academic_center_id() != self.academic_center_id {
            return Err(InvalidElection::ManifestAcademicCenterMismatch);
        }
        if manifest.calendar_origin_hash() != &self.calendar_origin_hash {
            return Err(InvalidElection::ManifestCalendarOriginMismatch);
        }
        self.record_configuration(Some(ElectionConfiguration::Confidential(Box::new(
            verified,
        ))));
        Ok(())
    }

    /// Abre a eleição no instante fornecido pela camada de aplicação.
    ///
    /// Exige opções na identificada ou manifesto verificado na confidencial,
    /// dentro do período vigente [starts_at, ends_at). Não exige eleitor ou chave
    /// previamente cadastrado. Retorna erro se a preparação estiver incompleta,
    /// se o estado for incompatível ou se o instante violar calendário/histórico.
    pub fn open(&mut self, opened_at: DateTime<Utc>) -> Result<(), ElectionTransitionError> {
        if self.status != ElectionStatus::Preparing {
            return Err(ElectionTransitionError::InvalidState);
        }

        if self.configuration.is_none() {
            return Err(match self.modality {
                ElectionModality::Confidential => ElectionTransitionError::ManifestRequired,
                ElectionModality::Identified => ElectionTransitionError::ConfigurationRequired,
            });
        }
        if self
            .schedule_changes
            .last()
            .is_some_and(|applied| applied.change.changed_at() > &opened_at)
        {
            return Err(ElectionTransitionError::NonChronologicalOperation);
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
        self.opening_schedule_revision = Some(self.schedule_revision);
        self.opening_configuration_count = Some(self.configuration_changes.len());
        self.status = ElectionStatus::Open;

        Ok(())
    }

    /// Cancela administrativamente uma eleição ainda não terminal.
    ///
    /// O cancelamento preserva o manifesto, o calendário, o histórico de
    /// revisões e eventual registro de encerramento. A autorização do ator
    /// deve ser verificada pela camada de aplicação.
    /// Na confidencial, manifesto publicado exige uma cabeça de quadro; sem
    /// participantes, essa cabeça é o hash do manifesto. Antes do manifesto,
    /// não há cabeça ou participantes. A identificada não exige quadro.
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
        if self.opened_at.is_none() && voter_count.value() != 0 {
            return Err(ElectionTransitionError::CancellationSnapshotMismatch);
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

            if head_hash.as_ref() != closure.head_hash() || voter_count != closure.voter_count() {
                return Err(ElectionTransitionError::CancellationSnapshotMismatch);
            }
        }

        if self
            .schedule_changes
            .last()
            .is_some_and(|applied| applied.change.changed_at() > &cancelled_at)
        {
            return Err(ElectionTransitionError::NonChronologicalOperation);
        }
        if self.modality == ElectionModality::Identified && head_hash.is_some() {
            return Err(ElectionTransitionError::ModalityMismatch);
        }
        if self.modality == ElectionModality::Confidential {
            validate_confidential_board_snapshot(
                self.manifest_hash(),
                head_hash.as_ref(),
                voter_count,
            )
            .map_err(|error| match error {
                InvalidBoardSnapshot::InvalidEmptyBoard => {
                    ElectionTransitionError::InvalidEmptyBoard
                }
                _ => ElectionTransitionError::CancellationSnapshotMismatch,
            })?;
        }
        let reason = reason.into();
        let reason_length = reason.chars().count();

        if !(1..=2000).contains(&reason_length) {
            return Err(ElectionTransitionError::InvalidCancellationReasonLength);
        }

        self.cancellation = Some(ElectionCancellation {
            election_id: self.id,
            manifest_hash: self.manifest_hash().cloned(),
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

    /// Encerra a votação confidencial com a fotografia e digest fornecidos pelo codec.
    ///
    /// A aplicação obtém cabeça e participantes atomicamente (#381) e confere o
    /// digest do encerramento (#380). Repetições recuperam a mesma fotografia.
    pub fn close(
        &mut self,
        closed_at: DateTime<Utc>,
        head_hash: Fingerprint,
        voter_count: VoterCount,
        closure_hash: Fingerprint,
    ) -> Result<&ElectionClosure, ElectionTransitionError> {
        if self.modality != ElectionModality::Confidential {
            return Err(ElectionTransitionError::ModalityMismatch);
        }
        self.close_snapshot(closed_at, Some(head_hash), voter_count, closure_hash)
    }

    /// Encerra a modalidade identificada sem quadro ou manifesto criptográfico.
    ///
    /// O digest identifica a fotografia imutável produzida pelo adaptador; não
    /// impõe o formato JSON do encerramento confidencial à modalidade identificada.
    pub fn close_identified(
        &mut self,
        closed_at: DateTime<Utc>,
        voter_count: VoterCount,
        closure_hash: Fingerprint,
    ) -> Result<&ElectionClosure, ElectionTransitionError> {
        if self.modality != ElectionModality::Identified {
            return Err(ElectionTransitionError::ModalityMismatch);
        }
        self.close_snapshot(closed_at, None, voter_count, closure_hash)
    }

    /// Aplica as invariantes comuns de fechamento; dados de armazenamento vêm da aplicação.
    fn close_snapshot(
        &mut self,
        closed_at: DateTime<Utc>,
        head_hash: Option<Fingerprint>,
        voter_count: VoterCount,
        closure_hash: Fingerprint,
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
        if closed_at.nanosecond() != 0 {
            return Err(ElectionTransitionError::SubsecondPrecision);
        }
        if closed_at < *self.current_period.ends_at() {
            return Err(ElectionTransitionError::ClosingBeforeDeadline);
        }
        let closure = match (self.manifest_hash().cloned(), head_hash) {
            (Some(manifest), Some(head)) => ElectionClosure::try_new(
                self.id,
                manifest,
                self.current_schedule_hash.clone(),
                head,
                voter_count,
                closed_at,
            ),
            (None, None) => ElectionClosure::try_identified(
                self.id,
                self.current_schedule_hash.clone(),
                voter_count,
                closed_at,
            ),
            _ => return Err(ElectionTransitionError::ModalityMismatch),
        }
        .map_err(|error| match error {
            InvalidElectionClosure::SubsecondPrecision => {
                ElectionTransitionError::SubsecondPrecision
            }
            InvalidElectionClosure::InvalidEmptyBoard => ElectionTransitionError::InvalidEmptyBoard,
        })?;
        self.closure = Some(closure);
        self.closure_hash = Some(closure_hash);
        self.status = ElectionStatus::Closed;
        self.closure
            .as_ref()
            .ok_or(ElectionTransitionError::InvalidState)
    }

    /// Retorna o digest do encerramento, independente do resultado que o referencia.
    pub const fn closure_hash(&self) -> Option<&Fingerprint> {
        self.closure_hash.as_ref()
    }

    /// Retorna a situação pública sem confundir cancelamento com apuração.
    ///
    /// O resultado assinado de cancelamento será produzido pelo adaptador a partir
    /// do registro preservado; não há contagem antecipada nessa projeção.
    pub fn result_status(&self) -> Option<ResultStatus> {
        if self.status == ElectionStatus::Cancelled {
            Some(ResultStatus::Cancelled)
        } else {
            self.tally_result().map(ElectionResult::status)
        }
    }

    /// Produz o resultado administrativo sem apurar, preservando a modalidade.
    ///
    /// O codec fornece o digest do registro de cancelamento assinado. Recusa
    /// eleições que ainda não foram canceladas; não publica totais ou vencedor.
    pub fn cancelled_result(
        &self,
        cancellation_hash: Fingerprint,
    ) -> Result<ElectionResult, ElectionTransitionError> {
        let record = self
            .cancellation()
            .ok_or(ElectionTransitionError::InvalidState)?;
        ElectionResult::try_cancelled(
            self.id,
            self.modality,
            self.manifest_hash().cloned(),
            cancellation_hash,
            record.voter_count(),
        )
        .map_err(|_| ElectionTransitionError::InvalidState)
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
        verified: VerifiedTally,
    ) -> Result<&ElectionResult, ElectionTransitionError> {
        let result = verified.result();
        let closure_hash = self
            .closure_hash
            .as_ref()
            .ok_or(ElectionTransitionError::TallyBeforeClosure)?;
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

        if result.modality() != self.modality {
            return Err(ElectionTransitionError::ModalityMismatch);
        }
        if result.election_id() != self.id || closure.election_id() != self.id {
            return Err(ElectionTransitionError::TallyElectionMismatch);
        }
        if result.manifest_hash() != self.manifest_hash()
            || closure.manifest_hash() != self.manifest_hash()
        {
            return Err(ElectionTransitionError::TallyManifestMismatch);
        }
        if result.closure_hash() != Some(closure_hash) {
            return Err(ElectionTransitionError::TallyClosureMismatch);
        }
        let options = self
            .options()
            .ok_or(ElectionTransitionError::ConfigurationRequired)?;
        if result.voter_count() != closure.voter_count() {
            return Err(ElectionTransitionError::TallyVoterCountMismatch);
        }

        if result.status() == ResultStatus::Incomplete
            && result.missing_authorities().iter().any(|missing| {
                !self.manifest().is_some_and(|manifest| {
                    manifest
                        .authorities()
                        .iter()
                        .any(|authority| authority.id() == *missing)
                })
            })
        {
            return Err(ElectionTransitionError::TallyAuthorityMismatch);
        }

        match self.modality {
            ElectionModality::Confidential => {
                let aggregate = verified
                    .aggregate()
                    .ok_or(ElectionTransitionError::TallyAggregateMismatch)?;
                if aggregate.election_id() != self.id
                    || Some(aggregate.manifest_hash()) != self.manifest_hash()
                    || aggregate.closure_hash() != closure_hash
                    || verified.aggregate_hash() != result.aggregate_hash()
                    || !aggregate
                        .ciphertexts()
                        .iter()
                        .map(|c| c.option_id())
                        .eq(options.iter().map(|o| o.id()))
                {
                    return Err(ElectionTransitionError::TallyAggregateMismatch);
                }
                if self.tally_result.as_ref().is_some_and(|previous| {
                    previous.aggregate() != verified.aggregate()
                        || previous.aggregate_hash() != verified.aggregate_hash()
                }) {
                    return Err(ElectionTransitionError::TallyAggregateMismatch);
                }
                let manifest = self
                    .manifest()
                    .ok_or(ElectionTransitionError::ManifestRequired)?;
                let contributions: std::collections::HashSet<_> = verified
                    .contributing_authorities()
                    .iter()
                    .copied()
                    .collect();
                if contributions.len() != verified.contributing_authorities().len()
                    || contributions
                        .iter()
                        .any(|id| !manifest.authorities().iter().any(|a| a.id() == *id))
                {
                    return Err(ElectionTransitionError::TallyContributionsMismatch);
                }
                let missing: std::collections::HashSet<_> = manifest
                    .authorities()
                    .iter()
                    .map(|a| a.id())
                    .filter(|id| !contributions.contains(id))
                    .collect();
                if missing != result.missing_authorities().iter().copied().collect() {
                    return Err(ElectionTransitionError::TallyContributionsMismatch);
                }
            }
            ElectionModality::Identified => {
                if verified.aggregate().is_some()
                    || verified.aggregate_hash().is_some()
                    || !verified.contributing_authorities().is_empty()
                    || result.aggregate_hash().is_some()
                    || result.status() == ResultStatus::Incomplete
                {
                    return Err(ElectionTransitionError::ModalityMismatch);
                }
            }
        }

        match result.status() {
            ResultStatus::Valid => {
                let counts = result
                    .counts()
                    .ok_or(ElectionTransitionError::TallyOptionsMismatch)?;

                match validate_result_counts(options, counts, closure.voter_count()) {
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

                    match validate_result_counts(options, counts, closure.voter_count()) {
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
            if self.tally_result() == Some(result) {
                return self
                    .tally_result()
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

        self.tally_result = Some(verified);

        self.tally_result()
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

        if change.manifest_hash() != self.manifest_hash() {
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

        if self
            .schedule_changes
            .last()
            .is_some_and(|applied| applied.change.changed_at() > change.changed_at())
        {
            return Err(ElectionTransitionError::NonChronologicalOperation);
        }
        let new_period = change.period();

        if new_period.ends_at() <= self.current_period.ends_at() {
            return Err(ElectionTransitionError::EndMustBeExtended);
        }

        match self.status {
            ElectionStatus::Preparing => {
                if new_period.starts_at() < self.current_period.starts_at() {
                    return Err(ElectionTransitionError::StartCannotMoveEarlier);
                }
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
        });

        Ok(())
    }
}

#[cfg(test)]
#[path = "election/tests.rs"]
mod tests;

#[path = "election/reconstitution.rs"]
mod reconstitution;
