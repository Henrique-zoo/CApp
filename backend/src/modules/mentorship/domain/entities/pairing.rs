//! Vínculo confirmado entre padrinho e afilhado.
//!
//! Guarda a atribuição manual ou a aceitação de sugestão, com limite de
//! padrinhos por calouro e papéis compatíveis. Encerrar preenche `ends_at`,
//! preserva o par e libera a capacidade do afilhado. A checagem usa as
//! inscrições, sugestões e vínculos vigentes recebidos pelo método; disputa
//! concorrente fica para a migration.

use crate::modules::mentorship::domain::entities::participant::MentorshipParticipant;
use crate::modules::mentorship::domain::entities::program::MentorshipProgram;
use crate::modules::mentorship::domain::entities::run::MatchingRun;
use crate::modules::mentorship::domain::entities::suggestion::MatchingSuggestion;
use crate::modules::mentorship::domain::{MentorshipMatchStatus, MentorshipParticipantRole};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Fotografia persistida do vínculo de apadrinhamento. Não é um contrato HTTP.
///
/// Campos privados expõem leitura por métodos e alteração pelos caminhos
/// validados de atribuição, aceitação e encerramento. A reconstituição
/// PostgreSQL usa o mapeamento `sqlx::FromRow` adotado no projeto.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MentorshipMatch {
    id: Uuid,
    program_id: Uuid,
    mentor_id: Uuid,
    mentee_id: Uuid,
    suggestion_id: Option<Uuid>,
    confirmed_by_id: Uuid,
    status: MentorshipMatchStatus,
    starts_at: DateTime<Utc>,
    ends_at: Option<DateTime<Utc>>,
}

/// Uma atribuição, aceitação ou encerramento que não preservaria as regras do vínculo.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("Vínculo de apadrinhamento inválido")]
pub struct InvalidMatch;

impl MentorshipMatch {
    /// Identificador interno estável deste vínculo.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Programa ao qual o vínculo pertence.
    pub fn program_id(&self) -> Uuid {
        self.program_id
    }

    /// Participante no papel de veterano/padrinho.
    pub fn mentor_id(&self) -> Uuid {
        self.mentor_id
    }

    /// Participante no papel de calouro/afilhado.
    pub fn mentee_id(&self) -> Uuid {
        self.mentee_id
    }

    /// Sugestão que originou o vínculo, ausente em atribuição manual.
    pub fn suggestion_id(&self) -> Option<Uuid> {
        self.suggestion_id
    }

    /// Usuário que confirmou a formação do vínculo.
    pub fn confirmed_by_id(&self) -> Uuid {
        self.confirmed_by_id
    }

    /// Situação persistida do vínculo.
    pub fn status(&self) -> MentorshipMatchStatus {
        self.status
    }

    /// Instante de início da vigência.
    pub fn starts_at(&self) -> DateTime<Utc> {
        self.starts_at
    }

    /// Instante de encerramento, quando o vínculo já terminou.
    pub fn ends_at(&self) -> Option<DateTime<Utc>> {
        self.ends_at
    }

    /// Indica se o vínculo ainda está vigente.
    pub fn is_active(&self) -> bool {
        self.status == MentorshipMatchStatus::Active
    }

    /// Atribui o par manualmente, sem sugestão.
    ///
    /// # Parâmetros e pré-condições
    ///
    /// - `program` precisa aceitar vínculo em `starts_at` (aberto e dentro das
    ///   datas).
    /// - `mentor` e `mentee` são participantes diferentes, ativos, com papéis
    ///   de padrinho e afilhado, no mesmo programa.
    /// - `active_matches` traz vínculos vigentes conhecidos para impedir o
    ///   mesmo par ativo e aplicar `mentors_per_mentee` ao calouro.
    ///
    /// Não há limite de afilhados por veterano. `suggestion_id` permanece
    /// ausente. Não grava no banco.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidMatch`] se o programa ou os participantes forem
    /// inelegíveis, se o par já estiver vigente ou se o calouro já tiver
    /// atingido o limite de padrinhos.
    pub fn assign(
        id: Uuid,
        program: &MentorshipProgram,
        mentor: &MentorshipParticipant,
        mentee: &MentorshipParticipant,
        confirmed_by_id: Uuid,
        starts_at: DateTime<Utc>,
        active_matches: &[Self],
    ) -> Result<Self, InvalidMatch> {
        validate_new_match(program, mentor, mentee, starts_at, active_matches)?;

        Ok(Self {
            id,
            program_id: program.id(),
            mentor_id: mentor.id(),
            mentee_id: mentee.id(),
            suggestion_id: None,
            confirmed_by_id,
            status: MentorshipMatchStatus::Active,
            starts_at,
            ends_at: None,
        })
    }

    /// Aceita a sugestão pendente e cria o vínculo correspondente.
    ///
    /// Grava `suggestion_id`, exige o mesmo programa e o mesmo par da
    /// sugestão, e marca a sugestão como `accepted` somente após revalidar
    /// elegibilidade e limite. Em erro, a sugestão permanece intacta.
    ///
    /// # Parâmetros e pré-condições
    ///
    /// - `program` e `run` precisam ser do mesmo programa; a execução deve ser
    ///   a da sugestão.
    /// - `mentor` e `mentee` batem com a sugestão e passam pelas mesmas regras
    ///   de [`Self::assign`].
    /// - `active_matches` traz vínculos vigentes para unicidade e capacidade.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidMatch`] se a sugestão não puder ser aceita, se o
    /// programa/participantes forem inelegíveis, se o par já estiver vigente
    /// ou se o limite de padrinhos for excedido.
    pub fn accept_suggestion(
        id: Uuid,
        program: &MentorshipProgram,
        run: &MatchingRun,
        suggestion: &mut MatchingSuggestion,
        mentor: &MentorshipParticipant,
        mentee: &MentorshipParticipant,
        confirmed_by_id: Uuid,
        starts_at: DateTime<Utc>,
        active_matches: &[Self],
    ) -> Result<Self, InvalidMatch> {
        if run.program_id() != program.id() || suggestion.run_id() != run.id() {
            return Err(InvalidMatch);
        }
        if suggestion.mentor_id() != mentor.id() || suggestion.mentee_id() != mentee.id() {
            return Err(InvalidMatch);
        }

        validate_new_match(program, mentor, mentee, starts_at, active_matches)?;

        suggestion
            .mark_accepted(run, mentor, mentee)
            .map_err(|_| InvalidMatch)?;

        Ok(Self {
            id,
            program_id: program.id(),
            mentor_id: mentor.id(),
            mentee_id: mentee.id(),
            suggestion_id: Some(suggestion.id()),
            confirmed_by_id,
            status: MentorshipMatchStatus::Active,
            starts_at,
            ends_at: None,
        })
    }

    /// Encerra o vínculo: preenche `ends_at`, passa a `ended` e libera a capacidade do calouro.
    ///
    /// Preserva o histórico do par. Em `Ok(())`, o vínculo deixa de contar no
    /// limite de padrinhos vigentes.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidMatch`] se o vínculo já estiver encerrado.
    pub fn end(&mut self, ends_at: DateTime<Utc>) -> Result<(), InvalidMatch> {
        if !self.is_active() {
            return Err(InvalidMatch);
        }

        self.status = MentorshipMatchStatus::Ended;
        self.ends_at = Some(ends_at);
        Ok(())
    }
}

fn validate_new_match(
    program: &MentorshipProgram,
    mentor: &MentorshipParticipant,
    mentee: &MentorshipParticipant,
    starts_at: DateTime<Utc>,
    active_matches: &[MentorshipMatch],
) -> Result<(), InvalidMatch> {
    if !program.accepts_match(starts_at) {
        return Err(InvalidMatch);
    }
    if mentor.program_id() != program.id() || mentee.program_id() != program.id() {
        return Err(InvalidMatch);
    }
    if mentor.id() == mentee.id() || mentor.user_id() == mentee.user_id() {
        return Err(InvalidMatch);
    }
    if mentor.role() != MentorshipParticipantRole::Veterano
        || mentee.role() != MentorshipParticipantRole::Calouro
    {
        return Err(InvalidMatch);
    }
    if !mentor.can_enter_new_match() || !mentee.can_enter_new_match() {
        return Err(InvalidMatch);
    }

    if active_matches
        .iter()
        .any(|m| m.is_active() && m.mentor_id == mentor.id() && m.mentee_id == mentee.id())
    {
        return Err(InvalidMatch);
    }

    let active_for_mentee = active_matches
        .iter()
        .filter(|m| m.is_active() && m.mentee_id == mentee.id())
        .count();
    if active_for_mentee >= program.mentors_per_mentee() as usize {
        return Err(InvalidMatch);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::mentorship::domain::entities::program::ProgramConfig;
    use crate::modules::mentorship::domain::entities::run::MatchingRun;
    use crate::modules::mentorship::domain::{
        MatchingStrategy, MatchingSuggestionStatus, MentorshipParticipantRole,
    };
    use chrono::TimeZone;
    use serde_json::json;

    fn at(year: i32, month: u32, day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap()
    }

    fn open_program(mentors_per_mentee: i32) -> MentorshipProgram {
        let mut program = MentorshipProgram::draft(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ProgramConfig {
                name: "Apadrinhamento 2026.1".into(),
                matching_strategy: MatchingStrategy::Criteria,
                mentors_per_mentee,
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

    fn propose(
        run: &MatchingRun,
        mentor: &MentorshipParticipant,
        mentee: &MentorshipParticipant,
    ) -> MatchingSuggestion {
        MatchingSuggestion::propose(Uuid::new_v4(), run, mentor, mentee, Some(0.8), 1, &[]).unwrap()
    }

    #[test]
    fn assign_creates_active_manual_match_without_suggestion() {
        let program = open_program(2);
        let mentor = enroll(&program, MentorshipParticipantRole::Veterano);
        let mentee = enroll(&program, MentorshipParticipantRole::Calouro);
        let confirmed_by = Uuid::new_v4();

        let pairing = MentorshipMatch::assign(
            Uuid::new_v4(),
            &program,
            &mentor,
            &mentee,
            confirmed_by,
            at(2026, 4, 1),
            &[],
        )
        .unwrap();

        assert!(pairing.is_active());
        assert_eq!(pairing.status(), MentorshipMatchStatus::Active);
        assert_eq!(pairing.program_id(), program.id());
        assert_eq!(pairing.mentor_id(), mentor.id());
        assert_eq!(pairing.mentee_id(), mentee.id());
        assert!(pairing.suggestion_id().is_none());
        assert_eq!(pairing.confirmed_by_id(), confirmed_by);
        assert_eq!(pairing.starts_at(), at(2026, 4, 1));
        assert!(pairing.ends_at().is_none());
    }

    #[test]
    fn assign_rejects_ineligible_program_roles_and_inactive_participants() {
        let mut program = open_program(2);
        let other = open_program(2);
        let mentor = enroll(&program, MentorshipParticipantRole::Veterano);
        let mentee = enroll(&program, MentorshipParticipantRole::Calouro);
        let foreign_mentee = enroll(&other, MentorshipParticipantRole::Calouro);
        let mut inactive = enroll(&program, MentorshipParticipantRole::Calouro);
        inactive.leave(at(2026, 5, 1)).unwrap();
        let confirmed_by = Uuid::new_v4();

        assert_eq!(
            MentorshipMatch::assign(
                Uuid::new_v4(),
                &program,
                &mentor,
                &mentor,
                confirmed_by,
                at(2026, 4, 1),
                &[],
            )
            .err(),
            Some(InvalidMatch)
        );
        assert_eq!(
            MentorshipMatch::assign(
                Uuid::new_v4(),
                &program,
                &mentee,
                &mentor,
                confirmed_by,
                at(2026, 4, 1),
                &[],
            )
            .err(),
            Some(InvalidMatch)
        );
        assert_eq!(
            MentorshipMatch::assign(
                Uuid::new_v4(),
                &program,
                &mentor,
                &foreign_mentee,
                confirmed_by,
                at(2026, 4, 1),
                &[],
            )
            .err(),
            Some(InvalidMatch)
        );
        assert_eq!(
            MentorshipMatch::assign(
                Uuid::new_v4(),
                &program,
                &mentor,
                &inactive,
                confirmed_by,
                at(2026, 4, 1),
                &[],
            )
            .err(),
            Some(InvalidMatch)
        );
        assert_eq!(
            MentorshipMatch::assign(
                Uuid::new_v4(),
                &program,
                &mentor,
                &mentee,
                confirmed_by,
                at(2026, 2, 1),
                &[],
            )
            .err(),
            Some(InvalidMatch)
        );

        program.close().unwrap();
        assert_eq!(
            MentorshipMatch::assign(
                Uuid::new_v4(),
                &program,
                &mentor,
                &mentee,
                confirmed_by,
                at(2026, 4, 1),
                &[],
            )
            .err(),
            Some(InvalidMatch)
        );
    }

    #[test]
    fn assign_enforces_unique_active_pair_and_mentors_per_mentee() {
        let program = open_program(2);
        let mentor_a = enroll(&program, MentorshipParticipantRole::Veterano);
        let mentor_b = enroll(&program, MentorshipParticipantRole::Veterano);
        let mentor_c = enroll(&program, MentorshipParticipantRole::Veterano);
        let mentee = enroll(&program, MentorshipParticipantRole::Calouro);
        let confirmed_by = Uuid::new_v4();

        let first = MentorshipMatch::assign(
            Uuid::new_v4(),
            &program,
            &mentor_a,
            &mentee,
            confirmed_by,
            at(2026, 4, 1),
            &[],
        )
        .unwrap();

        assert_eq!(
            MentorshipMatch::assign(
                Uuid::new_v4(),
                &program,
                &mentor_a,
                &mentee,
                confirmed_by,
                at(2026, 4, 2),
                &[first.clone()],
            )
            .err(),
            Some(InvalidMatch)
        );

        let second = MentorshipMatch::assign(
            Uuid::new_v4(),
            &program,
            &mentor_b,
            &mentee,
            confirmed_by,
            at(2026, 4, 2),
            &[first.clone()],
        )
        .unwrap();

        assert_eq!(
            MentorshipMatch::assign(
                Uuid::new_v4(),
                &program,
                &mentor_c,
                &mentee,
                confirmed_by,
                at(2026, 4, 3),
                &[first.clone(), second.clone()],
            )
            .err(),
            Some(InvalidMatch)
        );

        // Não há limite de afilhados por veterano.
        let other_mentee = enroll(&program, MentorshipParticipantRole::Calouro);
        let third = MentorshipMatch::assign(
            Uuid::new_v4(),
            &program,
            &mentor_a,
            &other_mentee,
            confirmed_by,
            at(2026, 4, 3),
            &[first, second],
        );
        assert!(third.is_ok());
    }

    #[test]
    fn ending_frees_mentee_capacity_and_allows_same_pair_again() {
        let program = open_program(1);
        let mentor = enroll(&program, MentorshipParticipantRole::Veterano);
        let mentee = enroll(&program, MentorshipParticipantRole::Calouro);
        let confirmed_by = Uuid::new_v4();

        let mut pairing = MentorshipMatch::assign(
            Uuid::new_v4(),
            &program,
            &mentor,
            &mentee,
            confirmed_by,
            at(2026, 4, 1),
            &[],
        )
        .unwrap();

        assert_eq!(
            MentorshipMatch::assign(
                Uuid::new_v4(),
                &program,
                &mentor,
                &mentee,
                confirmed_by,
                at(2026, 4, 2),
                &[pairing.clone()],
            )
            .err(),
            Some(InvalidMatch)
        );

        pairing.end(at(2026, 5, 1)).unwrap();
        assert_eq!(pairing.status(), MentorshipMatchStatus::Ended);
        assert_eq!(pairing.ends_at(), Some(at(2026, 5, 1)));
        assert!(!pairing.is_active());
        assert!(pairing.end(at(2026, 5, 2)).is_err());

        let renewed = MentorshipMatch::assign(
            Uuid::new_v4(),
            &program,
            &mentor,
            &mentee,
            confirmed_by,
            at(2026, 5, 2),
            &[pairing],
        )
        .unwrap();
        assert!(renewed.is_active());
        assert!(renewed.suggestion_id().is_none());
    }

    #[test]
    fn accept_suggestion_creates_match_and_marks_suggestion() {
        let program = open_program(2);
        let run = completed_run(&program);
        let mentor = enroll(&program, MentorshipParticipantRole::Veterano);
        let mentee = enroll(&program, MentorshipParticipantRole::Calouro);
        let mut suggestion = propose(&run, &mentor, &mentee);
        let confirmed_by = Uuid::new_v4();

        let pairing = MentorshipMatch::accept_suggestion(
            Uuid::new_v4(),
            &program,
            &run,
            &mut suggestion,
            &mentor,
            &mentee,
            confirmed_by,
            at(2026, 4, 3),
            &[],
        )
        .unwrap();

        assert_eq!(suggestion.status(), MatchingSuggestionStatus::Accepted);
        assert_eq!(pairing.suggestion_id(), Some(suggestion.id()));
        assert_eq!(pairing.mentor_id(), mentor.id());
        assert_eq!(pairing.mentee_id(), mentee.id());
        assert!(pairing.is_active());
    }

    #[test]
    fn accept_suggestion_rejects_mismatch_and_keeps_suggestion_pending() {
        let program = open_program(1);
        let other = open_program(1);
        let run = completed_run(&program);
        let other_run = completed_run(&other);
        let mentor = enroll(&program, MentorshipParticipantRole::Veterano);
        let mentee = enroll(&program, MentorshipParticipantRole::Calouro);
        let other_mentee = enroll(&program, MentorshipParticipantRole::Calouro);
        let mut suggestion = propose(&run, &mentor, &mentee);
        let confirmed_by = Uuid::new_v4();

        assert_eq!(
            MentorshipMatch::accept_suggestion(
                Uuid::new_v4(),
                &program,
                &other_run,
                &mut suggestion,
                &mentor,
                &mentee,
                confirmed_by,
                at(2026, 4, 3),
                &[],
            )
            .err(),
            Some(InvalidMatch)
        );
        assert_eq!(suggestion.status(), MatchingSuggestionStatus::Pending);

        assert_eq!(
            MentorshipMatch::accept_suggestion(
                Uuid::new_v4(),
                &program,
                &run,
                &mut suggestion,
                &mentor,
                &other_mentee,
                confirmed_by,
                at(2026, 4, 3),
                &[],
            )
            .err(),
            Some(InvalidMatch)
        );
        assert_eq!(suggestion.status(), MatchingSuggestionStatus::Pending);

        suggestion.reject().unwrap();
        assert_eq!(
            MentorshipMatch::accept_suggestion(
                Uuid::new_v4(),
                &program,
                &run,
                &mut suggestion,
                &mentor,
                &mentee,
                confirmed_by,
                at(2026, 4, 3),
                &[],
            )
            .err(),
            Some(InvalidMatch)
        );
        assert_eq!(suggestion.status(), MatchingSuggestionStatus::Rejected);
    }

    #[test]
    fn accept_suggestion_respects_capacity_before_marking() {
        let program = open_program(1);
        let run = completed_run(&program);
        let mentor_a = enroll(&program, MentorshipParticipantRole::Veterano);
        let mentor_b = enroll(&program, MentorshipParticipantRole::Veterano);
        let mentee = enroll(&program, MentorshipParticipantRole::Calouro);
        let existing = MentorshipMatch::assign(
            Uuid::new_v4(),
            &program,
            &mentor_a,
            &mentee,
            Uuid::new_v4(),
            at(2026, 4, 1),
            &[],
        )
        .unwrap();

        let mut suggestion =
            MatchingSuggestion::propose(Uuid::new_v4(), &run, &mentor_b, &mentee, None, 1, &[])
                .unwrap();

        assert_eq!(
            MentorshipMatch::accept_suggestion(
                Uuid::new_v4(),
                &program,
                &run,
                &mut suggestion,
                &mentor_b,
                &mentee,
                Uuid::new_v4(),
                at(2026, 4, 3),
                &[existing],
            )
            .err(),
            Some(InvalidMatch)
        );
        assert_eq!(suggestion.status(), MatchingSuggestionStatus::Pending);
    }
}
