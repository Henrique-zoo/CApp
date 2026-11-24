//! Sugestão de par produzida por uma execução de matching.
//!
//! Guarda padrinho, afilhado, pontuação opcional, posição e decisão. Aplica
//! unicidade `(run_id, mentor_id, mentee_id)` e papéis compatíveis. Recusar ou
//! expirar não cria vínculo; aceitar só marca a sugestão e deixa a formação do
//! vínculo para a entidade correspondente.

use crate::modules::mentorship::domain::entities::participant::MentorshipParticipant;
use crate::modules::mentorship::domain::entities::run::MatchingRun;
use crate::modules::mentorship::domain::{MatchingSuggestionStatus, MentorshipParticipantRole};
use uuid::Uuid;

/// Fotografia persistida de uma sugestão de par. Não é um contrato HTTP.
///
/// Campos privados expõem leitura por métodos e alteração pelas decisões
/// validadas. A reconstituição PostgreSQL usa o mapeamento `sqlx::FromRow`
/// adotado no projeto.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MatchingSuggestion {
    id: Uuid,
    run_id: Uuid,
    mentor_id: Uuid,
    mentee_id: Uuid,
    score: Option<f64>,
    rank: i32,
    status: MatchingSuggestionStatus,
}

/// Um registro ou decisão de sugestão que não preservaria as regras do domínio.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("Sugestão de matching inválida")]
pub struct InvalidSuggestion;

impl MatchingSuggestion {
    /// Identificador interno estável desta sugestão.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Execução que produziu a sugestão.
    pub fn run_id(&self) -> Uuid {
        self.run_id
    }

    /// Participante no papel de veterano/padrinho.
    pub fn mentor_id(&self) -> Uuid {
        self.mentor_id
    }

    /// Participante no papel de calouro/afilhado.
    pub fn mentee_id(&self) -> Uuid {
        self.mentee_id
    }

    /// Medida de compatibilidade, quando o método a atribui.
    pub fn score(&self) -> Option<f64> {
        self.score
    }

    /// Posição positiva da sugestão para o calouro nesta execução.
    pub fn rank(&self) -> i32 {
        self.rank
    }

    /// Situação da decisão sobre a sugestão.
    pub fn status(&self) -> MatchingSuggestionStatus {
        self.status
    }

    /// Indica se a sugestão ainda aguarda decisão.
    pub fn is_pending(&self) -> bool {
        self.status == MatchingSuggestionStatus::Pending
    }

    /// Registra a sugestão em `pending`, sem criar vínculo.
    ///
    /// # Parâmetros e pré-condições
    ///
    /// - `mentor` e `mentee` são participantes diferentes, ativos, com papéis
    ///   de padrinho e afilhado, no mesmo programa da execução.
    /// - `rank` é positivo.
    /// - `score`, quando presente, é finito.
    /// - `existing` traz sugestões já conhecidas da execução para conferir a
    ///   unicidade `(run_id, mentor_id, mentee_id)`.
    ///
    /// Não transforma a sugestão em vínculo e não processa o matching.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidSuggestion`] se os participantes forem incompatíveis,
    /// se `rank`/`score` forem inválidos ou se o par já existir na execução.
    pub fn propose(
        id: Uuid,
        run: &MatchingRun,
        mentor: &MentorshipParticipant,
        mentee: &MentorshipParticipant,
        score: Option<f64>,
        rank: i32,
        existing: &[Self],
    ) -> Result<Self, InvalidSuggestion> {
        validate_pair(run, mentor, mentee)?;
        validate_rank(rank)?;
        validate_score(score)?;

        if existing.iter().any(|s| {
            s.run_id == run.id() && s.mentor_id == mentor.id() && s.mentee_id == mentee.id()
        }) {
            return Err(InvalidSuggestion);
        }

        Ok(Self {
            id,
            run_id: run.id(),
            mentor_id: mentor.id(),
            mentee_id: mentee.id(),
            score,
            rank,
            status: MatchingSuggestionStatus::Pending,
        })
    }

    /// Recusa a sugestão: somente `pending` → `rejected`.
    ///
    /// Não cria vínculo. Em erro, o objeto fica intacto.
    pub fn reject(&mut self) -> Result<(), InvalidSuggestion> {
        self.decide(MatchingSuggestionStatus::Rejected)
    }

    /// Expira a sugestão: somente `pending` → `expired`.
    ///
    /// Não cria vínculo. Em erro, o objeto fica intacto.
    pub fn expire(&mut self) -> Result<(), InvalidSuggestion> {
        self.decide(MatchingSuggestionStatus::Expired)
    }

    /// Marca a sugestão como aceita: somente `pending` → `accepted`.
    ///
    /// Revalida elegibilidade dos participantes recebidos. Não cria o vínculo;
    /// a formação do par correspondente fica na entidade de vínculo, que deve
    /// gravar este `id` como origem.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidSuggestion`] se a sugestão não estiver pendente ou se o
    /// par deixou de ser elegível. Em erro, o objeto fica intacto.
    pub fn mark_accepted(
        &mut self,
        run: &MatchingRun,
        mentor: &MentorshipParticipant,
        mentee: &MentorshipParticipant,
    ) -> Result<(), InvalidSuggestion> {
        if !self.is_pending() {
            return Err(InvalidSuggestion);
        }
        if self.run_id != run.id() || self.mentor_id != mentor.id() || self.mentee_id != mentee.id()
        {
            return Err(InvalidSuggestion);
        }

        validate_pair(run, mentor, mentee)?;
        self.status = MatchingSuggestionStatus::Accepted;
        Ok(())
    }

    fn decide(&mut self, next: MatchingSuggestionStatus) -> Result<(), InvalidSuggestion> {
        if !self.is_pending() {
            return Err(InvalidSuggestion);
        }
        self.status = next;
        Ok(())
    }
}

fn validate_pair(
    run: &MatchingRun,
    mentor: &MentorshipParticipant,
    mentee: &MentorshipParticipant,
) -> Result<(), InvalidSuggestion> {
    if mentor.id() == mentee.id() || mentor.user_id() == mentee.user_id() {
        return Err(InvalidSuggestion);
    }
    if mentor.program_id() != run.program_id() || mentee.program_id() != run.program_id() {
        return Err(InvalidSuggestion);
    }
    if mentor.role() != MentorshipParticipantRole::Veterano
        || mentee.role() != MentorshipParticipantRole::Calouro
    {
        return Err(InvalidSuggestion);
    }
    if !mentor.can_enter_new_match() || !mentee.can_enter_new_match() {
        return Err(InvalidSuggestion);
    }
    Ok(())
}

fn validate_rank(rank: i32) -> Result<(), InvalidSuggestion> {
    if rank < 1 {
        return Err(InvalidSuggestion);
    }
    Ok(())
}

fn validate_score(score: Option<f64>) -> Result<(), InvalidSuggestion> {
    if score.is_some_and(|value| !value.is_finite()) {
        return Err(InvalidSuggestion);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::mentorship::domain::MatchingStrategy;
    use crate::modules::mentorship::domain::entities::program::{MentorshipProgram, ProgramConfig};
    use crate::modules::mentorship::domain::entities::run::MatchingRun;
    use chrono::{DateTime, TimeZone, Utc};
    use serde_json::json;

    fn at(year: i32, month: u32, day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap()
    }

    fn open_program() -> MentorshipProgram {
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
        program.open().unwrap();
        program
    }

    fn enroll(
        program: &MentorshipProgram,
        role: MentorshipParticipantRole,
    ) -> MentorshipParticipant {
        MentorshipParticipant::enroll(
            Uuid::new_v4(),
            program,
            Uuid::new_v4(),
            Uuid::new_v4(),
            role,
            false,
            at(2026, 4, 1),
            &[],
        )
        .unwrap()
    }

    fn completed_run(program: &MentorshipProgram) -> MatchingRun {
        let mut run = MatchingRun::request(
            Uuid::new_v4(),
            program,
            None,
            None,
            json!({"criteria": []}),
            at(2026, 4, 1),
        )
        .unwrap();
        run.start().unwrap();
        run.complete(at(2026, 4, 2)).unwrap();
        run
    }

    #[test]
    fn propose_requires_compatible_active_pair_and_unique_triple() {
        let program = open_program();
        let run = completed_run(&program);
        let mentor = enroll(&program, MentorshipParticipantRole::Veterano);
        let mentee = enroll(&program, MentorshipParticipantRole::Calouro);

        let suggestion =
            MatchingSuggestion::propose(Uuid::new_v4(), &run, &mentor, &mentee, Some(0.85), 1, &[])
                .unwrap();
        assert_eq!(suggestion.status(), MatchingSuggestionStatus::Pending);
        assert_eq!(suggestion.run_id(), run.id());
        assert_eq!(suggestion.mentor_id(), mentor.id());
        assert_eq!(suggestion.mentee_id(), mentee.id());
        assert_eq!(suggestion.score(), Some(0.85));
        assert_eq!(suggestion.rank(), 1);
        assert!(suggestion.is_pending());

        let duplicate = MatchingSuggestion::propose(
            Uuid::new_v4(),
            &run,
            &mentor,
            &mentee,
            None,
            2,
            &[suggestion.clone()],
        );
        assert_eq!(duplicate.err(), Some(InvalidSuggestion));

        let swapped = MatchingSuggestion::propose(
            Uuid::new_v4(),
            &run,
            &mentee,
            &mentor,
            None,
            1,
            &[suggestion],
        );
        assert_eq!(swapped.err(), Some(InvalidSuggestion));
    }

    #[test]
    fn propose_rejects_same_person_inactive_wrong_program_rank_and_score() {
        let program = open_program();
        let other = open_program();
        let run = completed_run(&program);
        let mentor = enroll(&program, MentorshipParticipantRole::Veterano);
        let mentee = enroll(&program, MentorshipParticipantRole::Calouro);
        let mut inactive_mentee = enroll(&program, MentorshipParticipantRole::Calouro);
        inactive_mentee.leave(at(2026, 5, 1)).unwrap();
        let foreign_mentee = enroll(&other, MentorshipParticipantRole::Calouro);

        assert_eq!(
            MatchingSuggestion::propose(Uuid::new_v4(), &run, &mentor, &mentor, None, 1, &[],)
                .err(),
            Some(InvalidSuggestion)
        );
        assert_eq!(
            MatchingSuggestion::propose(
                Uuid::new_v4(),
                &run,
                &mentor,
                &inactive_mentee,
                None,
                1,
                &[],
            )
            .err(),
            Some(InvalidSuggestion)
        );
        assert_eq!(
            MatchingSuggestion::propose(
                Uuid::new_v4(),
                &run,
                &mentor,
                &foreign_mentee,
                None,
                1,
                &[],
            )
            .err(),
            Some(InvalidSuggestion)
        );

        for (score, rank) in [
            (None, 0),
            (None, -1),
            (Some(f64::NAN), 1),
            (Some(f64::INFINITY), 1),
            (Some(f64::NEG_INFINITY), 1),
        ] {
            assert_eq!(
                MatchingSuggestion::propose(
                    Uuid::new_v4(),
                    &run,
                    &mentor,
                    &mentee,
                    score,
                    rank,
                    &[],
                )
                .err(),
                Some(InvalidSuggestion)
            );
        }
    }

    #[test]
    fn reject_and_expire_do_not_create_a_match() {
        let program = open_program();
        let run = completed_run(&program);
        let mentor = enroll(&program, MentorshipParticipantRole::Veterano);
        let mentee = enroll(&program, MentorshipParticipantRole::Calouro);

        let mut rejected =
            MatchingSuggestion::propose(Uuid::new_v4(), &run, &mentor, &mentee, Some(0.5), 1, &[])
                .unwrap();
        rejected.reject().unwrap();
        assert_eq!(rejected.status(), MatchingSuggestionStatus::Rejected);
        assert!(rejected.reject().is_err());
        assert!(rejected.expire().is_err());
        assert!(rejected.mark_accepted(&run, &mentor, &mentee).is_err());

        let other_mentee = enroll(&program, MentorshipParticipantRole::Calouro);
        let mut expired = MatchingSuggestion::propose(
            Uuid::new_v4(),
            &run,
            &mentor,
            &other_mentee,
            None,
            1,
            &[rejected],
        )
        .unwrap();
        expired.expire().unwrap();
        assert_eq!(expired.status(), MatchingSuggestionStatus::Expired);
        assert!(expired.expire().is_err());
        assert!(expired.reject().is_err());

        // Recusar ou expirar só altera o status da sugestão; não há entidade de
        // vínculo produzida por esses caminhos.
        assert_ne!(expired.status(), MatchingSuggestionStatus::Accepted);
        assert_ne!(expired.status(), MatchingSuggestionStatus::Pending);
    }

    #[test]
    fn mark_accepted_revalidates_pair_without_creating_a_match() {
        let program = open_program();
        let run = completed_run(&program);
        let mentor = enroll(&program, MentorshipParticipantRole::Veterano);
        let mut mentee = enroll(&program, MentorshipParticipantRole::Calouro);

        let mut suggestion =
            MatchingSuggestion::propose(Uuid::new_v4(), &run, &mentor, &mentee, Some(0.9), 1, &[])
                .unwrap();

        mentee.leave(at(2026, 5, 1)).unwrap();
        assert!(suggestion.mark_accepted(&run, &mentor, &mentee).is_err());
        assert_eq!(suggestion.status(), MatchingSuggestionStatus::Pending);

        let active_mentee = enroll(&program, MentorshipParticipantRole::Calouro);
        let mut suggestion = MatchingSuggestion::propose(
            Uuid::new_v4(),
            &run,
            &mentor,
            &active_mentee,
            None,
            2,
            &[],
        )
        .unwrap();
        suggestion
            .mark_accepted(&run, &mentor, &active_mentee)
            .unwrap();
        assert_eq!(suggestion.status(), MatchingSuggestionStatus::Accepted);
        assert!(
            suggestion
                .mark_accepted(&run, &mentor, &active_mentee)
                .is_err()
        );

        // Aceitar marca a sugestão; a criação do vínculo correspondente é
        // responsabilidade de outra entidade e não ocorre aqui.
        assert_eq!(suggestion.mentor_id(), mentor.id());
        assert_eq!(suggestion.mentee_id(), active_mentee.id());
    }
}
