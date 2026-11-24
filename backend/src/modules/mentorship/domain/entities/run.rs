//! Execução de matching solicitada para um programa.
//!
//! Guarda a estratégia usada, o snapshot de configuração e o ciclo de vida
//! `pending` → `running` → `completed`|`failed`. Não processa o matching nem
//! inclui segredos no snapshot. `model_id` é obrigatório somente na estratégia
//! neural; nas demais permanece ausente.

use crate::modules::mentorship::domain::entities::program::MentorshipProgram;
use crate::modules::mentorship::domain::{MatchingRunStatus, MatchingStrategy};
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

/// Fotografia persistida de uma execução de matching. Não é um contrato HTTP.
///
/// Campos privados expõem leitura por métodos e alteração pelas transições de
/// estado validadas. A reconstituição PostgreSQL usa o mapeamento
/// `sqlx::FromRow` adotado no projeto.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MatchingRun {
    id: Uuid,
    program_id: Uuid,
    model_id: Option<Uuid>,
    requested_by_id: Option<Uuid>,
    strategy: MatchingStrategy,
    status: MatchingRunStatus,
    configuration_snapshot: Value,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

/// Uma criação ou transição de execução que não preservaria as regras do domínio.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("Execução de matching inválida")]
pub struct InvalidRun;

impl MatchingRun {
    /// Identificador interno estável desta execução.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Programa ao qual a execução pertence.
    pub fn program_id(&self) -> Uuid {
        self.program_id
    }

    /// Modelo neural usado, quando a estratégia é neural.
    pub fn model_id(&self) -> Option<Uuid> {
        self.model_id
    }

    /// Usuário que solicitou a execução, quando informado.
    pub fn requested_by_id(&self) -> Option<Uuid> {
        self.requested_by_id
    }

    /// Método efetivamente usado nesta execução.
    ///
    /// Não muda se o programa for reconfigurado depois.
    pub fn strategy(&self) -> MatchingStrategy {
        self.strategy
    }

    /// Estado persistido do ciclo de vida.
    pub fn status(&self) -> MatchingRunStatus {
        self.status
    }

    /// Snapshot da configuração usada para explicar a execução.
    ///
    /// Não deve conter segredos dos participantes; a entidade armazena o valor
    /// recebido e não processa o matching.
    pub fn configuration_snapshot(&self) -> &Value {
        &self.configuration_snapshot
    }

    /// Instante em que a execução foi registrada (estado inicial `pending`).
    pub fn started_at(&self) -> DateTime<Utc> {
        self.started_at
    }

    /// Instante do término, presente somente em `completed` ou `failed`.
    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.completed_at
    }

    /// Indica se a execução ainda não terminou.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            MatchingRunStatus::Completed | MatchingRunStatus::Failed
        )
    }

    /// Cria a execução em `pending` para o programa aberto e dentro das datas.
    ///
    /// # Parâmetros e pré-condições
    ///
    /// - `program` precisa aceitar nova execução em `started_at`.
    /// - A estratégia gravada é a ativa do programa no momento da solicitação.
    /// - `model_id` é obrigatório somente quando a estratégia é
    ///   [`MatchingStrategy::Neural`]; nas demais deve estar ausente.
    /// - `configuration_snapshot` explica a configuração usada e não inclui
    ///   segredos dos participantes.
    ///
    /// Não processa o matching e não grava no banco.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidRun`] se o programa não aceita execução ou se `model_id`
    /// for incompatível com a estratégia ativa do programa.
    pub fn request(
        id: Uuid,
        program: &MentorshipProgram,
        model_id: Option<Uuid>,
        requested_by_id: Option<Uuid>,
        configuration_snapshot: Value,
        started_at: DateTime<Utc>,
    ) -> Result<Self, InvalidRun> {
        if !program.accepts_new_run(started_at) {
            return Err(InvalidRun);
        }

        let strategy = program.matching_strategy();
        validate_model_id(strategy, model_id)?;

        Ok(Self {
            id,
            program_id: program.id(),
            model_id,
            requested_by_id,
            strategy,
            status: MatchingRunStatus::Pending,
            configuration_snapshot,
            started_at,
            completed_at: None,
        })
    }

    /// Inicia o processamento: somente `pending` → `running`.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidRun`] se a execução não estiver pendente. Em erro, o
    /// objeto fica intacto.
    pub fn start(&mut self) -> Result<(), InvalidRun> {
        if self.status != MatchingRunStatus::Pending {
            return Err(InvalidRun);
        }
        self.status = MatchingRunStatus::Running;
        Ok(())
    }

    /// Conclui com sucesso: somente `running` → `completed`.
    ///
    /// Preenche `completed_at`. Pode não ter produzido sugestões; ausência de
    /// par compatível não é falha operacional desta transição.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidRun`] se a execução não estiver em andamento. Em erro, o
    /// objeto fica intacto.
    pub fn complete(&mut self, completed_at: DateTime<Utc>) -> Result<(), InvalidRun> {
        self.finish(MatchingRunStatus::Completed, completed_at)
    }

    /// Encerra com falha: somente `running` → `failed`.
    ///
    /// Preenche `completed_at`. A política de recuperação fica no fluxo.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidRun`] se a execução não estiver em andamento. Em erro, o
    /// objeto fica intacto.
    pub fn fail(&mut self, completed_at: DateTime<Utc>) -> Result<(), InvalidRun> {
        self.finish(MatchingRunStatus::Failed, completed_at)
    }

    fn finish(
        &mut self,
        terminal: MatchingRunStatus,
        completed_at: DateTime<Utc>,
    ) -> Result<(), InvalidRun> {
        if self.status != MatchingRunStatus::Running {
            return Err(InvalidRun);
        }
        self.status = terminal;
        self.completed_at = Some(completed_at);
        Ok(())
    }
}

fn validate_model_id(
    strategy: MatchingStrategy,
    model_id: Option<Uuid>,
) -> Result<(), InvalidRun> {
    match (strategy, model_id) {
        (MatchingStrategy::Neural, Some(_)) => Ok(()),
        (MatchingStrategy::Neural, None) => Err(InvalidRun),
        (_, None) => Ok(()),
        (_, Some(_)) => Err(InvalidRun),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::mentorship::domain::entities::program::ProgramConfig;
    use chrono::TimeZone;
    use serde_json::json;

    fn at(year: i32, month: u32, day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap()
    }

    fn program_with(strategy: MatchingStrategy) -> MentorshipProgram {
        let mut program = MentorshipProgram::draft(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ProgramConfig {
                name: "Apadrinhamento 2026.1".into(),
                matching_strategy: strategy,
                mentors_per_mentee: 2,
                allow_other_courses: false,
                starts_at: at(2026, 3, 1),
                ends_at: Some(at(2026, 7, 1)),
            },
        )
        .unwrap();
        program.open().unwrap();
        program
    }

    fn snapshot_without_secrets() -> Value {
        json!({
            "criteria": [{"code": "affinity", "weight": 1.0}],
            "mentors_per_mentee": 2
        })
    }

    #[test]
    fn request_requires_open_program_within_window() {
        let mut program = MentorshipProgram::draft(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ProgramConfig {
                name: "Apadrinhamento 2026.1".into(),
                matching_strategy: MatchingStrategy::Criteria,
                mentors_per_mentee: 2,
                allow_other_courses: false,
                starts_at: at(2026, 3, 1),
                ends_at: Some(at(2026, 7, 1)),
            },
        )
        .unwrap();

        let draft = MatchingRun::request(
            Uuid::new_v4(),
            &program,
            None,
            Some(Uuid::new_v4()),
            snapshot_without_secrets(),
            at(2026, 4, 1),
        );
        assert_eq!(draft.err(), Some(InvalidRun));

        program.open().unwrap();

        let before = MatchingRun::request(
            Uuid::new_v4(),
            &program,
            None,
            None,
            snapshot_without_secrets(),
            at(2026, 2, 28),
        );
        assert_eq!(before.err(), Some(InvalidRun));

        let run = MatchingRun::request(
            Uuid::new_v4(),
            &program,
            None,
            Some(Uuid::new_v4()),
            snapshot_without_secrets(),
            at(2026, 4, 1),
        )
        .unwrap();
        assert_eq!(run.status(), MatchingRunStatus::Pending);
        assert_eq!(run.strategy(), MatchingStrategy::Criteria);
        assert_eq!(run.program_id(), program.id());
        assert!(run.completed_at().is_none());
        assert!(!run.is_terminal());

        let after_end = MatchingRun::request(
            Uuid::new_v4(),
            &program,
            None,
            None,
            snapshot_without_secrets(),
            at(2026, 7, 1),
        );
        assert_eq!(after_end.err(), Some(InvalidRun));
    }

    #[test]
    fn model_id_required_only_for_neural_strategy() {
        let criteria = program_with(MatchingStrategy::Criteria);
        assert_eq!(
            MatchingRun::request(
                Uuid::new_v4(),
                &criteria,
                Some(Uuid::new_v4()),
                None,
                snapshot_without_secrets(),
                at(2026, 4, 1),
            )
            .err(),
            Some(InvalidRun)
        );

        let without_model = MatchingRun::request(
            Uuid::new_v4(),
            &criteria,
            None,
            None,
            snapshot_without_secrets(),
            at(2026, 4, 1),
        )
        .unwrap();
        assert!(without_model.model_id().is_none());

        let neural = program_with(MatchingStrategy::Neural);
        assert_eq!(
            MatchingRun::request(
                Uuid::new_v4(),
                &neural,
                None,
                None,
                snapshot_without_secrets(),
                at(2026, 4, 1),
            )
            .err(),
            Some(InvalidRun)
        );

        let model_id = Uuid::new_v4();
        let with_model = MatchingRun::request(
            Uuid::new_v4(),
            &neural,
            Some(model_id),
            None,
            snapshot_without_secrets(),
            at(2026, 4, 1),
        )
        .unwrap();
        assert_eq!(with_model.model_id(), Some(model_id));
        assert_eq!(with_model.strategy(), MatchingStrategy::Neural);
    }

    #[test]
    fn strategy_is_frozen_from_program_at_request_time() {
        let mut program = program_with(MatchingStrategy::Secrets);
        let run = MatchingRun::request(
            Uuid::new_v4(),
            &program,
            None,
            None,
            snapshot_without_secrets(),
            at(2026, 4, 1),
        )
        .unwrap();

        program
            .reconfigure(ProgramConfig {
                name: "Apadrinhamento 2026.1".into(),
                matching_strategy: MatchingStrategy::Manual,
                mentors_per_mentee: 2,
                allow_other_courses: false,
                starts_at: at(2026, 3, 1),
                ends_at: Some(at(2026, 7, 1)),
            })
            .unwrap();

        assert_eq!(program.matching_strategy(), MatchingStrategy::Manual);
        assert_eq!(run.strategy(), MatchingStrategy::Secrets);
    }

    #[test]
    fn snapshot_stores_configuration_without_secrets() {
        let program = program_with(MatchingStrategy::Criteria);
        let snapshot = snapshot_without_secrets();
        let run = MatchingRun::request(
            Uuid::new_v4(),
            &program,
            None,
            None,
            snapshot.clone(),
            at(2026, 4, 1),
        )
        .unwrap();

        assert_eq!(run.configuration_snapshot(), &snapshot);
        assert!(run.configuration_snapshot().get("encrypted_value").is_none());
        assert!(run.configuration_snapshot().get("secrets").is_none());
    }

    #[test]
    fn lifecycle_pending_running_completed_or_failed() {
        let program = program_with(MatchingStrategy::Manual);
        let mut run = MatchingRun::request(
            Uuid::new_v4(),
            &program,
            None,
            None,
            snapshot_without_secrets(),
            at(2026, 4, 1),
        )
        .unwrap();

        assert!(run.complete(at(2026, 4, 2)).is_err());
        assert!(run.fail(at(2026, 4, 2)).is_err());
        assert_eq!(run.status(), MatchingRunStatus::Pending);

        run.start().unwrap();
        assert_eq!(run.status(), MatchingRunStatus::Running);
        assert!(run.start().is_err());

        run.complete(at(2026, 4, 2)).unwrap();
        assert_eq!(run.status(), MatchingRunStatus::Completed);
        assert_eq!(run.completed_at(), Some(at(2026, 4, 2)));
        assert!(run.is_terminal());
        assert!(run.start().is_err());
        assert!(run.complete(at(2026, 4, 3)).is_err());
        assert!(run.fail(at(2026, 4, 3)).is_err());

        let mut failed = MatchingRun::request(
            Uuid::new_v4(),
            &program,
            None,
            None,
            snapshot_without_secrets(),
            at(2026, 4, 1),
        )
        .unwrap();
        failed.start().unwrap();
        failed.fail(at(2026, 4, 3)).unwrap();
        assert_eq!(failed.status(), MatchingRunStatus::Failed);
        assert_eq!(failed.completed_at(), Some(at(2026, 4, 3)));
        assert!(failed.is_terminal());
    }

    #[test]
    fn completed_may_produce_zero_suggestions() {
        let program = program_with(MatchingStrategy::Criteria);
        let mut run = MatchingRun::request(
            Uuid::new_v4(),
            &program,
            None,
            None,
            snapshot_without_secrets(),
            at(2026, 4, 1),
        )
        .unwrap();
        run.start().unwrap();
        run.complete(at(2026, 4, 2)).unwrap();
        assert_eq!(run.status(), MatchingRunStatus::Completed);
    }
}
