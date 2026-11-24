//! Inscrição de um usuário em um programa de apadrinhamento.
//!
//! Guarda o papel, a situação e a preferência de alcance entre cursos. Aplica as
//! invariantes de inscrição, unicidade por programa, saída e elegibilidade para
//! vínculo novo. Não cria o usuário, o vínculo acadêmico nem o programa; esses
//! chegam como identidade ou referência já validada pelo chamador.

use crate::modules::mentorship::domain::entities::program::MentorshipProgram;
use crate::modules::mentorship::domain::{
    MentorshipParticipantRole, MentorshipParticipantStatus,
};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Fotografia persistida da inscrição no programa. Não é um contrato HTTP.
///
/// Campos privados expõem leitura por métodos e alteração pelos caminhos
/// validados de inscrição e saída. A reconstituição PostgreSQL usa o mapeamento
/// `sqlx::FromRow` adotado no projeto.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MentorshipParticipant {
    id: Uuid,
    program_id: Uuid,
    user_id: Uuid,
    enrollment_id: Uuid,
    role: MentorshipParticipantRole,
    status: MentorshipParticipantStatus,
    include_other_courses_in_quiz: bool,
    joined_at: DateTime<Utc>,
    left_at: Option<DateTime<Utc>>,
}

/// Uma inscrição, saída ou preferência que não preservaria as regras do participante.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("Inscrição ou transição de participante inválida")]
pub struct InvalidParticipant;

impl MentorshipParticipant {
    /// Identificador interno estável desta inscrição.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Programa ao qual a inscrição pertence.
    pub fn program_id(&self) -> Uuid {
        self.program_id
    }

    /// Usuário inscrito.
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    /// Vínculo acadêmico usado como base da participação.
    pub fn enrollment_id(&self) -> Uuid {
        self.enrollment_id
    }

    /// Papel no programa: veterano/padrinho ou calouro/afilhado.
    pub fn role(&self) -> MentorshipParticipantRole {
        self.role
    }

    /// Situação persistida da inscrição.
    pub fn status(&self) -> MentorshipParticipantStatus {
        self.status
    }

    /// Preferência do veterano por incluir outros cursos no quiz.
    ///
    /// Calouro permanece em `false`. Veterano só fica em `true` quando o
    /// programa admite outros cursos.
    pub fn include_other_courses_in_quiz(&self) -> bool {
        self.include_other_courses_in_quiz
    }

    /// Instante de entrada no programa.
    pub fn joined_at(&self) -> DateTime<Utc> {
        self.joined_at
    }

    /// Instante de saída, quando a inscrição já foi encerrada.
    pub fn left_at(&self) -> Option<DateTime<Utc>> {
        self.left_at
    }

    /// Indica se a inscrição está habilitada.
    pub fn is_active(&self) -> bool {
        self.status == MentorshipParticipantStatus::Active
    }

    /// Indica se o participante pode entrar em um vínculo novo.
    ///
    /// Somente inscrição ativa; a elegibilidade do programa e o limite de
    /// padrinhos pertencem à formação do vínculo.
    pub fn can_enter_new_match(&self) -> bool {
        self.is_active()
    }

    /// Cria a inscrição ativa no programa, com papel e preferência validados.
    ///
    /// # Parâmetros e pré-condições
    ///
    /// - `program` precisa aceitar inscrição em `joined_at` (aberto e dentro das
    ///   datas).
    /// - `role` já é um papel reconhecido; a interpretação textual fica em
    ///   [`MentorshipParticipantRole::parse`].
    /// - `include_other_courses_in_quiz`: calouro não marca verdadeiro; veterano
    ///   só marca verdadeiro se `program.allow_other_courses()`. O padrão do
    ///   próprio curso é `false`.
    /// - `existing` traz inscrições já conhecidas (qualquer status) para conferir
    ///   a unicidade `(program_id, user_id)`, inclusive após saída.
    ///
    /// Não consulta usuário nem vínculo acadêmico e não grava no banco.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidParticipant`] se o programa não aceita inscrição, se a
    /// preferência viola o papel ou o alcance do programa, ou se já existe
    /// inscrição do mesmo usuário neste programa.
    pub fn enroll(
        id: Uuid,
        program: &MentorshipProgram,
        user_id: Uuid,
        enrollment_id: Uuid,
        role: MentorshipParticipantRole,
        include_other_courses_in_quiz: bool,
        joined_at: DateTime<Utc>,
        existing: &[Self],
    ) -> Result<Self, InvalidParticipant> {
        if !program.accepts_enrollment(joined_at) {
            return Err(InvalidParticipant);
        }

        if existing
            .iter()
            .any(|p| p.program_id == program.id() && p.user_id == user_id)
        {
            return Err(InvalidParticipant);
        }

        validate_course_preference(role, include_other_courses_in_quiz, program.allow_other_courses())?;

        Ok(Self {
            id,
            program_id: program.id(),
            user_id,
            enrollment_id,
            role,
            status: MentorshipParticipantStatus::Active,
            include_other_courses_in_quiz,
            joined_at,
            left_at: None,
        })
    }

    /// Atualiza a preferência de alcance entre cursos enquanto a inscrição está ativa.
    ///
    /// As mesmas regras de [`Self::enroll`] valem: calouro não marca verdadeiro;
    /// veterano só marca verdadeiro se o programa admite outros cursos.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidParticipant`] se a inscrição não estiver ativa ou se a
    /// preferência for incompatível. Em erro, o objeto fica intacto.
    pub fn set_include_other_courses_in_quiz(
        &mut self,
        include_other_courses_in_quiz: bool,
        allow_other_courses: bool,
    ) -> Result<(), InvalidParticipant> {
        if !self.is_active() {
            return Err(InvalidParticipant);
        }

        validate_course_preference(self.role, include_other_courses_in_quiz, allow_other_courses)?;
        self.include_other_courses_in_quiz = include_other_courses_in_quiz;
        Ok(())
    }

    /// Encerra a inscrição: preenche `left_at`, passa a `inactive` e preserva o histórico.
    ///
    /// Em `Ok(())`, a inscrição deixa de aceitar vínculo novo. A unicidade
    /// `(program_id, user_id)` permanece; não há reinscrição neste programa.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidParticipant`] se a inscrição já estiver inativa.
    pub fn leave(&mut self, left_at: DateTime<Utc>) -> Result<(), InvalidParticipant> {
        if !self.is_active() {
            return Err(InvalidParticipant);
        }

        self.status = MentorshipParticipantStatus::Inactive;
        self.left_at = Some(left_at);
        Ok(())
    }
}

/// Preferência de outros cursos só é verdadeira para veterano em programa que a admite.
fn validate_course_preference(
    role: MentorshipParticipantRole,
    include_other_courses_in_quiz: bool,
    allow_other_courses: bool,
) -> Result<(), InvalidParticipant> {
    if !include_other_courses_in_quiz {
        return Ok(());
    }

    if role != MentorshipParticipantRole::Veterano || !allow_other_courses {
        return Err(InvalidParticipant);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::mentorship::domain::entities::program::ProgramConfig;
    use crate::modules::mentorship::domain::MatchingStrategy;
    use chrono::TimeZone;

    fn at(year: i32, month: u32, day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap()
    }

    fn config(allow_other_courses: bool) -> ProgramConfig {
        ProgramConfig {
            name: "Apadrinhamento 2026.1".into(),
            matching_strategy: MatchingStrategy::Manual,
            mentors_per_mentee: 2,
            allow_other_courses,
            starts_at: at(2026, 3, 1),
            ends_at: Some(at(2026, 7, 1)),
        }
    }

    fn open_program(allow_other_courses: bool) -> MentorshipProgram {
        let mut program = MentorshipProgram::draft(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            config(allow_other_courses),
        )
        .unwrap();
        program.open().unwrap();
        program
    }

    fn enroll_ok(
        program: &MentorshipProgram,
        user_id: Uuid,
        role: MentorshipParticipantRole,
        include_other_courses_in_quiz: bool,
        existing: &[MentorshipParticipant],
    ) -> MentorshipParticipant {
        MentorshipParticipant::enroll(
            Uuid::new_v4(),
            program,
            user_id,
            Uuid::new_v4(),
            role,
            include_other_courses_in_quiz,
            at(2026, 4, 1),
            existing,
        )
        .unwrap()
    }

    #[test]
    fn parse_role_accepts_only_veterano_and_calouro() {
        assert_eq!(
            MentorshipParticipantRole::parse(Some("veterano")),
            Some(MentorshipParticipantRole::Veterano)
        );
        assert_eq!(
            MentorshipParticipantRole::parse(Some("  calouro  ")),
            Some(MentorshipParticipantRole::Calouro)
        );
        assert_eq!(MentorshipParticipantRole::Veterano.as_str(), "veterano");
        assert_eq!(MentorshipParticipantRole::Calouro.as_str(), "calouro");
        assert!(MentorshipParticipantRole::Veterano.is_mentor());
        assert!(MentorshipParticipantRole::Calouro.is_mentee());

        for label in [None, Some(""), Some(" "), Some("padrinho"), Some("Veterano")] {
            assert_eq!(MentorshipParticipantRole::parse(label), None);
        }
    }

    #[test]
    fn enroll_requires_open_program_within_window() {
        let mut program = MentorshipProgram::draft(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            config(false),
        )
        .unwrap();
        let user_id = Uuid::new_v4();

        let draft_result = MentorshipParticipant::enroll(
            Uuid::new_v4(),
            &program,
            user_id,
            Uuid::new_v4(),
            MentorshipParticipantRole::Calouro,
            false,
            at(2026, 4, 1),
            &[],
        );
        assert_eq!(draft_result.err(), Some(InvalidParticipant));

        program.open().unwrap();

        let before = MentorshipParticipant::enroll(
            Uuid::new_v4(),
            &program,
            user_id,
            Uuid::new_v4(),
            MentorshipParticipantRole::Calouro,
            false,
            at(2026, 2, 28),
            &[],
        );
        assert_eq!(before.err(), Some(InvalidParticipant));

        let participant = MentorshipParticipant::enroll(
            Uuid::new_v4(),
            &program,
            user_id,
            Uuid::new_v4(),
            MentorshipParticipantRole::Calouro,
            false,
            at(2026, 4, 1),
            &[],
        )
        .unwrap();
        assert_eq!(participant.status(), MentorshipParticipantStatus::Active);
        assert_eq!(participant.program_id(), program.id());
        assert!(!participant.include_other_courses_in_quiz());
        assert!(participant.can_enter_new_match());
        assert!(participant.left_at().is_none());

        let after_end = MentorshipParticipant::enroll(
            Uuid::new_v4(),
            &program,
            Uuid::new_v4(),
            Uuid::new_v4(),
            MentorshipParticipantRole::Veterano,
            false,
            at(2026, 7, 1),
            &[],
        );
        assert_eq!(after_end.err(), Some(InvalidParticipant));
    }

    #[test]
    fn uniqueness_is_per_program_including_after_leave() {
        let program_a = open_program(false);
        let program_b = open_program(false);
        let user_id = Uuid::new_v4();

        let first = enroll_ok(
            &program_a,
            user_id,
            MentorshipParticipantRole::Calouro,
            false,
            &[],
        );

        let duplicate = MentorshipParticipant::enroll(
            Uuid::new_v4(),
            &program_a,
            user_id,
            Uuid::new_v4(),
            MentorshipParticipantRole::Veterano,
            false,
            at(2026, 4, 1),
            &[first.clone()],
        );
        assert_eq!(duplicate.err(), Some(InvalidParticipant));

        let mut left = first.clone();
        left.leave(at(2026, 5, 1)).unwrap();
        assert_eq!(left.status(), MentorshipParticipantStatus::Inactive);
        assert_eq!(left.left_at(), Some(at(2026, 5, 1)));
        assert!(!left.can_enter_new_match());
        assert!(left.leave(at(2026, 5, 2)).is_err());

        let after_leave = MentorshipParticipant::enroll(
            Uuid::new_v4(),
            &program_a,
            user_id,
            Uuid::new_v4(),
            MentorshipParticipantRole::Veterano,
            false,
            at(2026, 5, 2),
            &[left],
        );
        assert_eq!(after_leave.err(), Some(InvalidParticipant));

        let other_program = enroll_ok(
            &program_b,
            user_id,
            MentorshipParticipantRole::Veterano,
            false,
            &[],
        );
        assert_eq!(other_program.role(), MentorshipParticipantRole::Veterano);
        assert_eq!(other_program.program_id(), program_b.id());
    }

    #[test]
    fn course_preference_respects_role_and_program() {
        let restricted = open_program(false);
        let open_courses = open_program(true);
        let user_id = Uuid::new_v4();

        assert_eq!(
            MentorshipParticipant::enroll(
                Uuid::new_v4(),
                &restricted,
                user_id,
                Uuid::new_v4(),
                MentorshipParticipantRole::Calouro,
                true,
                at(2026, 4, 1),
                &[],
            )
            .err(),
            Some(InvalidParticipant)
        );

        assert_eq!(
            MentorshipParticipant::enroll(
                Uuid::new_v4(),
                &restricted,
                user_id,
                Uuid::new_v4(),
                MentorshipParticipantRole::Veterano,
                true,
                at(2026, 4, 1),
                &[],
            )
            .err(),
            Some(InvalidParticipant)
        );

        let mentee = enroll_ok(
            &open_courses,
            user_id,
            MentorshipParticipantRole::Calouro,
            false,
            &[],
        );
        let mut mentee = mentee;
        assert!(mentee
            .set_include_other_courses_in_quiz(true, true)
            .is_err());
        assert!(!mentee.include_other_courses_in_quiz());

        let mentor = enroll_ok(
            &open_courses,
            Uuid::new_v4(),
            MentorshipParticipantRole::Veterano,
            true,
            &[],
        );
        assert!(mentor.include_other_courses_in_quiz());

        let mut mentor_own_course = enroll_ok(
            &open_courses,
            Uuid::new_v4(),
            MentorshipParticipantRole::Veterano,
            false,
            &[],
        );
        mentor_own_course
            .set_include_other_courses_in_quiz(true, true)
            .unwrap();
        assert!(mentor_own_course.include_other_courses_in_quiz());
        mentor_own_course
            .set_include_other_courses_in_quiz(false, false)
            .unwrap();
        assert!(!mentor_own_course.include_other_courses_in_quiz());

        mentor_own_course.leave(at(2026, 5, 1)).unwrap();
        assert!(mentor_own_course
            .set_include_other_courses_in_quiz(false, true)
            .is_err());
    }
}
