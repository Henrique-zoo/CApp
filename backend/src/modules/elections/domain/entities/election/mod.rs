//! Agregado raiz que controla o ciclo de vida de uma eleição.

use std::{collections::HashSet, mem};

use chrono::{DateTime, Timelike, Utc};
use uuid::Uuid;

use crate::modules::elections::domain::{
    counts::VoterCount,
    entities::{
        configuration::ElectionConfiguration,
        result::{FinalTallyResult, TallyComputation},
    },
    error::{ElectionTransitionError, InvalidElectionPeriod},
    identifiers::{AcademicCenterId, AuthorityId, ElectionId},
};

/// Define quais dados podem compor votos, configuração e apuração.
///
/// Ambas as modalidades usam identidade autenticada, assinatura e participação
/// única. Somente a confidencial exige ElGamal e autoridades de decifração.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElectionModality {
    /// O registro associa a conta à opção escolhida, com acesso autorizado.
    Identified,
    /// O servidor recebe somente a escolha cifrada no dispositivo.
    Confidential,
}

/// Estado operacional da eleição.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElectionState {
    /// Configuração e preparação das autoridades.
    Preparing,

    /// Configuração pronta, aguardando a abertura da votação.
    WaitingStart(ElectionConfiguration),

    /// Recebimento de votos.
    Open(Voting),

    /// Votação encerrada, com o conjunto final de votos fixado pela aplicação.
    /// Na modalidade confidencial, aguarda as contribuições para a apuração.
    Closed {
        /// Dados da votação aberta.
        voting: Voting,
        /// Instante em que a votação foi encerrada.
        closed_at: DateTime<Utc>,
        /// Total de votos distintos fixado na transação de encerramento.
        voter_count: VoterCount,
    },

    /// Apuração iniciada após a verificação das contribuições exigidas.
    CountingVotes {
        /// Dados da votação aberta.
        voting: Voting,
        /// Instante em que a votação foi encerrada.
        closed_at: DateTime<Utc>,
        /// Total de votos distintos fixado no encerramento.
        voter_count: VoterCount,
    },

    /// Resultado definitivo da apuração, válido ou anulado.
    Finalized {
        /// Dados da votação aberta.
        voting: Voting,
        /// Instante em que a votação foi encerrada.
        closed_at: DateTime<Utc>,
        /// Total de votos distintos fixado no encerramento.
        voter_count: VoterCount,
        /// Resultado definitivo da apuração.
        result: FinalTallyResult,
    },

    /// Cancelamento administrativo antes de haver um resultado definitivo.
    Cancelled {
        /// Instante em que a eleição foi cancelada.
        cancelled_at: DateTime<Utc>,
        /// Justificativa registrada no cancelamento.
        reason: String,
        /// UUID da conta que solicitou o cancelamento.
        requested_by: Uuid,
    },
}

/// Dados registrados na abertura e preservados durante a apuração.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Voting {
    /// Instante real em que a votação abriu.
    pub opened_at: DateTime<Utc>,
    /// Configuração imutável usada para aceitar votos.
    pub configuration: ElectionConfiguration,
}

/// Intervalo de tempo de uma eleição.
///
/// O intervalo segue a convenção `[starts_at, ends_at)`: o instante inicial
/// está incluído e o instante final não está incluído.
///
/// Os instantes são armazenados em UTC e devem ter precisão exata de segundos,
/// conforme o contrato eleitoral.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectionPeriod {
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
}

impl ElectionPeriod {
    /// Constrói um período eleitoral validando ordem e precisão temporal.
    pub fn try_new(
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
    ) -> Result<Self, InvalidElectionPeriod> {
        if starts_at.nanosecond() != 0 || ends_at.nanosecond() != 0 {
            return Err(InvalidElectionPeriod::SubsecondPrecision);
        }

        if starts_at >= ends_at {
            return Err(InvalidElectionPeriod::StartMustPrecedeEnd);
        }

        Ok(Self { starts_at, ends_at })
    }

    /// Retorna o instante de início do período.
    pub fn starts_at(&self) -> &DateTime<Utc> {
        &self.starts_at
    }

    /// Retorna o instante de encerramento do período.
    pub fn ends_at(&self) -> &DateTime<Utc> {
        &self.ends_at
    }

    /// Confere que uma revisão só adia o período e preserva uma abertura já ocorrida.
    /// O instante de abertura efetiva é ausente durante a preparação.
    pub fn validate_revision(
        &self,
        new_period: &Self,
        opened_at: Option<DateTime<Utc>>,
        changed_at: DateTime<Utc>,
    ) -> Result<(), ElectionTransitionError> {
        use ElectionTransitionError as Error;

        if let Some(opened_at) = opened_at {
            if new_period.starts_at() != self.starts_at() {
                return Err(Error::StartCannotChange);
            }
            if changed_at < opened_at {
                return Err(Error::ScheduleChangeBeforeOpening);
            }
            if changed_at >= *self.ends_at() {
                return Err(Error::ExtensionAfterDeadline);
            }
        } else if new_period.starts_at() < self.starts_at() {
            return Err(Error::StartCannotMoveEarlier);
        }
        if new_period.ends_at() <= self.ends_at() {
            return Err(Error::EndMustBeExtended);
        }

        Ok(())
    }

    /// Informa se o instante está dentro do intervalo `[starts_at, ends_at)`.
    pub fn contains(&self, instant: &DateTime<Utc>) -> bool {
        instant >= &self.starts_at && instant < &self.ends_at
    }
}

/// Agregado raiz responsável pelas invariantes do ciclo de vida eleitoral.
///
/// # Exemplo
///
/// Uma eleição identificada pode abrir sem manifesto ou autoridades ElGamal:
///
/// ```
/// use backend::modules::elections::domain::{
///     choices::{ElectionOption, ElectionOptions},
///     counts::VoterCount,
///     entities::{
///         configuration::ElectionConfiguration,
///         election::{Election, ElectionModality, ElectionPeriod, ElectionTransition},
///     },
///     identifiers::{AcademicCenterId, ElectionId, SlateId},
/// };
/// use chrono::{TimeZone, Utc};
/// use uuid::Uuid;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let id = ElectionId::try_from_uuid(Uuid::from_u128(1))?;
/// let ca = AcademicCenterId::try_from_uuid(Uuid::from_u128(2))?;
/// let start = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
/// let end = Utc.with_ymd_and_hms(2026, 11, 20, 18, 0, 0).unwrap();
/// let now = Utc.with_ymd_and_hms(2026, 11, 19, 10, 0, 0).unwrap();
/// let period = ElectionPeriod::try_new(start, end)?;
/// let mut election = Election::new(id, ca, period, ElectionModality::Identified, now);
/// let options = ElectionOptions::try_new(vec![
///     ElectionOption::try_slate(SlateId::try_from_uuid(Uuid::from_u128(3))?, "Aurora")?,
///     ElectionOption::Null,
/// ])?;
/// election.advance(ElectionTransition::CompletePreparation {
///     configuration: ElectionConfiguration::Identified(options),
/// })?;
/// election.advance(ElectionTransition::Open { opened_at: start })?;
/// election.advance(ElectionTransition::Close { closed_at: end, voter_count: VoterCount::ZERO })?;
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
/// use backend::modules::elections::domain::entities::election::Election;
/// fn bypass(election: &mut Election) { let _ = &mut election.state; }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Election {
    id: ElectionId,
    academic_center_id: AcademicCenterId,
    created_at: DateTime<Utc>,
    period: ElectionPeriod,
    modality: ElectionModality,
    state: ElectionState,
}

/// Ação a aplicar ao estado atual de uma eleição.
pub enum ElectionTransition {
    /// Torna a configuração pronta para abertura. Na secreta, a aplicação
    /// registra a publicação do manifesto junto da transição.
    CompletePreparation {
        /// Configuração já conferida para a modalidade escolhida.
        configuration: ElectionConfiguration,
    },
    /// Abre a votação dentro do período vigente.
    Open {
        /// Instante da tentativa obtido do relógio do backend.
        opened_at: DateTime<Utc>,
    },
    /// Encerra a votação e fixa o conjunto de votos aceitos na persistência.
    Close {
        /// Instante do encerramento obtido do relógio do backend.
        closed_at: DateTime<Utc>,
        /// Total confirmado na mesma transação que encerra a votação.
        voter_count: VoterCount,
    },
    /// Inicia a apuração de uma eleição encerrada.
    StartCounting {
        /// Autoridades com contribuição verificada para o agregado final.
        /// A aplicação fornece uma lista vazia na votação identificada.
        contributing_authorities: Vec<AuthorityId>,
    },
    /// Determina o resultado definitivo a partir da apuração do conjunto final.
    Finalize {
        /// Contagens calculadas pela aplicação ou falha ao recuperar um total.
        /// A falha de recuperação só se aplica à votação secreta.
        computation: TallyComputation,
    },
    /// Cancela a eleição por decisão de um integrante autorizado da comissão.
    Cancel {
        /// Instante do cancelamento obtido do relógio do backend.
        cancelled_at: DateTime<Utc>,
        /// UUID da conta autenticada que solicitou o cancelamento.
        requested_by: Uuid,
        /// Motivo registrado no histórico administrativo.
        reason: String,
    },
}

impl Election {
    /// Cria uma eleição em preparação com modalidade e calendário inicial.
    ///
    /// Registra `now` como instante de criação. A aplicação obtém esse valor do
    /// relógio do backend. A configuração e as contribuições das autoridades
    /// são reunidas durante a preparação.
    pub fn new(
        id: ElectionId,
        academic_center_id: AcademicCenterId,
        period: ElectionPeriod,
        modality: ElectionModality,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            academic_center_id,
            created_at: now,
            period,
            modality,
            state: ElectionState::Preparing,
        }
    }

    /// Retorna o identificador da eleição.
    pub const fn id(&self) -> ElectionId {
        self.id
    }

    /// Retorna o Centro Acadêmico responsável.
    pub const fn academic_center_id(&self) -> AcademicCenterId {
        self.academic_center_id
    }

    /// Retorna a fase atual, incluindo os dados registrados nessa fase.
    pub const fn state(&self) -> &ElectionState {
        &self.state
    }

    /// Retorna a modalidade escolhida na criação.
    pub const fn modality(&self) -> ElectionModality {
        self.modality
    }

    /// Retorna os horários previstos atualmente vigentes.
    pub const fn current_period(&self) -> &ElectionPeriod {
        &self.period
    }

    /// Aplica uma ação permitida para a fase atual.
    ///
    /// A aplicação fornece os instantes do relógio confiável e, na apuração
    /// secreta, os IDs das autoridades cujas contribuições foram verificadas
    /// para o agregado final. O encerramento, a contagem e a publicação são
    /// persistidos pelas operações correspondentes da aplicação.
    ///
    /// Retorna um erro se a fase, o instante ou os dados forem incompatíveis.
    /// Uma ação recusada preserva o estado anterior.
    pub fn advance(
        &mut self,
        transition: ElectionTransition,
    ) -> Result<(), ElectionTransitionError> {
        use ElectionTransitionError as Error;

        // Confira a fase e valide os dados antes de mover a configuração ou a votação.
        self.state = match (&self.state, transition) {
            (
                ElectionState::Preparing,
                ElectionTransition::CompletePreparation { configuration },
            ) => {
                self.validate_configuration(&configuration)?;

                ElectionState::WaitingStart(configuration)
            }
            (ElectionState::WaitingStart(_), ElectionTransition::Open { opened_at }) => {
                self.validate_opening(opened_at)?;

                let ElectionState::WaitingStart(configuration) =
                    mem::replace(&mut self.state, ElectionState::Preparing)
                else {
                    unreachable!();
                };

                ElectionState::Open(Voting {
                    opened_at,
                    configuration,
                })
            }
            (
                ElectionState::Open(_),
                ElectionTransition::Close {
                    closed_at,
                    voter_count,
                },
            ) => {
                self.validate_closing(closed_at)?;

                let ElectionState::Open(voting) =
                    mem::replace(&mut self.state, ElectionState::Preparing)
                else {
                    unreachable!();
                };

                ElectionState::Closed {
                    voting,
                    closed_at,
                    voter_count,
                }
            }
            (
                ElectionState::Closed { .. }
                | ElectionState::CountingVotes { .. }
                | ElectionState::Finalized { .. },
                ElectionTransition::Close { .. },
            ) => return Ok(()),
            (
                ElectionState::Closed { voting, .. },
                ElectionTransition::StartCounting {
                    contributing_authorities,
                },
            ) => {
                if !Self::has_required_contributions(
                    &voting.configuration,
                    &contributing_authorities,
                ) {
                    return Err(Error::ContributionsMismatch);
                }

                let ElectionState::Closed {
                    voting,
                    closed_at,
                    voter_count,
                } = mem::replace(&mut self.state, ElectionState::Preparing)
                else {
                    unreachable!();
                };

                ElectionState::CountingVotes {
                    voting,
                    closed_at,
                    voter_count,
                }
            }
            (
                ElectionState::CountingVotes { .. } | ElectionState::Finalized { .. },
                ElectionTransition::Finalize {
                    computation: TallyComputation::TotalOutOfRange,
                },
            ) if self.modality != ElectionModality::Confidential => {
                return Err(Error::ModalityMismatch);
            }
            (
                ElectionState::CountingVotes {
                    voting,
                    voter_count,
                    ..
                },
                ElectionTransition::Finalize { computation },
            ) => {
                let result = FinalTallyResult::from_computation(
                    computation,
                    voting.configuration.options(),
                    *voter_count,
                )?;

                let ElectionState::CountingVotes {
                    voting,
                    closed_at,
                    voter_count,
                } = mem::replace(&mut self.state, ElectionState::Preparing)
                else {
                    unreachable!();
                };

                ElectionState::Finalized {
                    voting,
                    closed_at,
                    voter_count,
                    result,
                }
            }
            (
                ElectionState::Finalized {
                    voting,
                    voter_count,
                    result: current,
                    ..
                },
                ElectionTransition::Finalize { computation },
            ) => {
                let result = FinalTallyResult::from_computation(
                    computation,
                    voting.configuration.options(),
                    *voter_count,
                )?;
                return if *current == result {
                    Ok(())
                } else {
                    Err(Error::InvalidState)
                };
            }
            (
                state,
                ElectionTransition::Cancel {
                    cancelled_at,
                    requested_by,
                    reason,
                },
            ) => {
                Self::validate_cancellation(state, cancelled_at, requested_by, &reason)?;
                if matches!(state, ElectionState::Cancelled { .. }) {
                    return Ok(());
                }

                ElectionState::Cancelled {
                    cancelled_at,
                    requested_by,
                    reason,
                }
            }
            _ => return Err(Error::InvalidState),
        };

        Ok(())
    }

    fn validate_configuration(
        &self,
        configuration: &ElectionConfiguration,
    ) -> Result<(), ElectionTransitionError> {
        if configuration.modality() != self.modality {
            return Err(ElectionTransitionError::ModalityMismatch);
        }
        if let ElectionConfiguration::Confidential(verified) = configuration {
            let manifest = verified.manifest();
            if manifest.election_id() != self.id
                || manifest.academic_center_id() != self.academic_center_id
            {
                return Err(ElectionTransitionError::InvalidConfiguration);
            }
        }

        Ok(())
    }

    fn validate_opening(&self, opened_at: DateTime<Utc>) -> Result<(), ElectionTransitionError> {
        if !self.period.contains(&opened_at) {
            return Err(ElectionTransitionError::OpeningOutsidePeriod {
                attempted_at: opened_at,
                starts_at: *self.period.starts_at(),
                ends_at: *self.period.ends_at(),
            });
        }

        Ok(())
    }

    fn validate_closing(&self, closed_at: DateTime<Utc>) -> Result<(), ElectionTransitionError> {
        if closed_at < *self.period.ends_at() {
            return Err(ElectionTransitionError::ClosingBeforeDeadline);
        }

        Ok(())
    }

    fn has_required_contributions(
        configuration: &ElectionConfiguration,
        contributing_authorities: &[AuthorityId],
    ) -> bool {
        match configuration {
            ElectionConfiguration::Identified(_) => contributing_authorities.is_empty(),
            ElectionConfiguration::Confidential(verified) => {
                let expected = verified.manifest().authorities();
                if contributing_authorities.len() != expected.len() {
                    return false;
                }

                let received: HashSet<_> = contributing_authorities.iter().copied().collect();
                expected
                    .iter()
                    .all(|authority| received.contains(&authority.id()))
            }
        }
    }

    fn validate_cancellation(
        state: &ElectionState,
        cancelled_at: DateTime<Utc>,
        requested_by: Uuid,
        reason: &str,
    ) -> Result<(), ElectionTransitionError> {
        use ElectionTransitionError as Error;

        if requested_by.is_nil() {
            return Err(Error::InvalidCancellationRequester);
        }
        if !(1..=2000).contains(&reason.chars().count()) {
            return Err(Error::InvalidCancellationReasonLength);
        }

        match state {
            ElectionState::Cancelled { .. } => return Ok(()),
            ElectionState::Finalized { .. } => return Err(Error::InvalidState),
            ElectionState::Open(voting) if cancelled_at < voting.opened_at => {
                return Err(Error::CancellationBeforeOpening);
            }
            ElectionState::Closed { closed_at, .. }
            | ElectionState::CountingVotes { closed_at, .. }
                if cancelled_at < *closed_at =>
            {
                return Err(Error::CancellationBeforeClosure);
            }
            _ => {}
        }

        Ok(())
    }

    /// Retorna a configuração pronta enquanto ela integra o estado da eleição.
    pub const fn configuration(&self) -> Option<&ElectionConfiguration> {
        match &self.state {
            ElectionState::WaitingStart(configuration) => Some(configuration),
            ElectionState::Open(voting)
            | ElectionState::Closed { voting, .. }
            | ElectionState::CountingVotes { voting, .. }
            | ElectionState::Finalized { voting, .. } => Some(&voting.configuration),
            ElectionState::Preparing | ElectionState::Cancelled { .. } => None,
        }
    }

    /// Altera o período vigente mantendo as restrições do calendário.
    ///
    /// Em preparação ou aguardando início, o início pode ser adiado. Uma
    /// votação aberta só admite prorrogação antes do fim vigente e preserva o
    /// início. Em todos os casos, o novo fim deve ser posterior ao anterior.
    /// A aplicação grava a alteração e o histórico na mesma transação.
    pub fn change_schedule(
        &mut self,
        new_period: ElectionPeriod,
        changed_at: DateTime<Utc>,
    ) -> Result<(), ElectionTransitionError> {
        let opened_at = match &self.state {
            ElectionState::Preparing | ElectionState::WaitingStart(_) => None,
            ElectionState::Open(voting) => Some(voting.opened_at),
            _ => return Err(ElectionTransitionError::InvalidState),
        };
        self.period
            .validate_revision(&new_period, opened_at, changed_at)?;

        self.period = new_period;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
