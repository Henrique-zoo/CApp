//! Critério configurável de matching ponderado de um programa.
//!
//! Guarda código, peso, parâmetros e habilitação. Aplica unicidade por programa
//! e só aceita registro enquanto o programa não estiver encerrado. Não executa o
//! cálculo de compatibilidade nem valida o contrato semântico de `parameters`.

use crate::modules::mentorship::domain::entities::program::MentorshipProgram;
use crate::modules::mentorship::domain::MentorshipProgramStatus;
use serde_json::Value;
use uuid::Uuid;

/// Fotografia persistida de um critério de matching. Não é um contrato HTTP.
///
/// Campos privados expõem leitura por métodos e alteração pelos caminhos
/// validados de registro e habilitação. A reconstituição PostgreSQL usa o
/// mapeamento `sqlx::FromRow` adotado no projeto.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MatchingCriterion {
    id: Uuid,
    program_id: Uuid,
    code: String,
    weight: f64,
    parameters: Value,
    enabled: bool,
}

/// Um registro ou alteração de critério que não preservaria as regras do domínio.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("Configuração de critério inválida")]
pub struct InvalidCriterion;

impl MatchingCriterion {
    /// Identificador interno estável deste critério.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Programa ao qual o critério pertence.
    pub fn program_id(&self) -> Uuid {
        self.program_id
    }

    /// Código normalizado do critério dentro do programa.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Peso numérico finito usado pelo algoritmo de matching.
    pub fn weight(&self) -> f64 {
        self.weight
    }

    /// JSON de configuração específica do critério; o contrato semântico é externo.
    pub fn parameters(&self) -> &Value {
        &self.parameters
    }

    /// Indica se o critério está habilitado para uso.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Registra o critério no programa, com unicidade `(program_id, code)`.
    ///
    /// # Parâmetros e pré-condições
    ///
    /// - `program` não pode estar encerrado.
    /// - `code` não vazio após `trim`.
    /// - `weight` finito (`f64::is_finite`).
    /// - `existing` traz critérios já conhecidos do programa para conferir a
    ///   unicidade do código.
    ///
    /// Não valida o significado de `parameters` nem executa o matching.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidCriterion`] se o programa estiver encerrado, o código for
    /// vazio, o peso não for finito ou o código já existir no programa.
    pub fn register(
        id: Uuid,
        program: &MentorshipProgram,
        code: String,
        weight: f64,
        parameters: Value,
        enabled: bool,
        existing: &[Self],
    ) -> Result<Self, InvalidCriterion> {
        if program.status() == MentorshipProgramStatus::Closed {
            return Err(InvalidCriterion);
        }

        let code = normalize_code(&code)?;
        validate_weight(weight)?;

        if existing
            .iter()
            .any(|c| c.program_id == program.id() && c.code == code)
        {
            return Err(InvalidCriterion);
        }

        Ok(Self {
            id,
            program_id: program.id(),
            code,
            weight,
            parameters,
            enabled,
        })
    }

    /// Liga ou desliga o critério sem removê-lo.
    ///
    /// Não exige revalidar o programa: o registro já existe e permanece no
    /// histórico configurável. Em sucesso, apenas `enabled` muda.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

fn normalize_code(code: &str) -> Result<String, InvalidCriterion> {
    let code = code.trim();
    if code.is_empty() {
        return Err(InvalidCriterion);
    }
    Ok(code.to_owned())
}

fn validate_weight(weight: f64) -> Result<(), InvalidCriterion> {
    if !weight.is_finite() {
        return Err(InvalidCriterion);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::mentorship::domain::entities::program::ProgramConfig;
    use crate::modules::mentorship::domain::MatchingStrategy;
    use chrono::{DateTime, TimeZone, Utc};
    use serde_json::json;

    fn at(year: i32, month: u32, day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap()
    }

    fn draft_program() -> MentorshipProgram {
        MentorshipProgram::draft(
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
        .unwrap()
    }

    fn open_program() -> MentorshipProgram {
        let mut program = draft_program();
        program.open().unwrap();
        program
    }

    #[test]
    fn register_normalizes_code_and_rejects_invalid_inputs() {
        let program = open_program();
        let criterion = MatchingCriterion::register(
            Uuid::new_v4(),
            &program,
            "  same_course  ".into(),
            1.5,
            json!({"radius": 2}),
            true,
            &[],
        )
        .unwrap();

        assert_eq!(criterion.code(), "same_course");
        assert_eq!(criterion.weight(), 1.5);
        assert_eq!(criterion.parameters(), &json!({"radius": 2}));
        assert!(criterion.enabled());
        assert_eq!(criterion.program_id(), program.id());

        for (code, weight) in [
            ("", 1.0),
            (" ", 1.0),
            ("ok", f64::NAN),
            ("ok", f64::INFINITY),
            ("ok", f64::NEG_INFINITY),
        ] {
            let result = MatchingCriterion::register(
                Uuid::new_v4(),
                &program,
                code.into(),
                weight,
                Value::Null,
                true,
                &[],
            );
            assert_eq!(result.err(), Some(InvalidCriterion));
        }
    }

    #[test]
    fn register_requires_program_not_closed_and_unique_code() {
        let mut program = draft_program();
        let first = MatchingCriterion::register(
            Uuid::new_v4(),
            &program,
            "affinity".into(),
            1.0,
            json!({}),
            true,
            &[],
        )
        .unwrap();

        let duplicate = MatchingCriterion::register(
            Uuid::new_v4(),
            &program,
            " affinity ".into(),
            2.0,
            json!({}),
            false,
            &[first.clone()],
        );
        assert_eq!(duplicate.err(), Some(InvalidCriterion));

        let other_code = MatchingCriterion::register(
            Uuid::new_v4(),
            &program,
            "semester".into(),
            0.0,
            json!({}),
            false,
            &[first],
        )
        .unwrap();
        assert!(!other_code.enabled());
        assert_eq!(other_code.weight(), 0.0);

        program.open().unwrap();
        program.close().unwrap();
        let closed = MatchingCriterion::register(
            Uuid::new_v4(),
            &program,
            "new".into(),
            1.0,
            json!({}),
            true,
            &[],
        );
        assert_eq!(closed.err(), Some(InvalidCriterion));
    }

    #[test]
    fn set_enabled_toggles_without_removing() {
        let program = open_program();
        let mut criterion = MatchingCriterion::register(
            Uuid::new_v4(),
            &program,
            "affinity".into(),
            1.0,
            json!({"k": 1}),
            true,
            &[],
        )
        .unwrap();

        criterion.set_enabled(false);
        assert!(!criterion.enabled());
        assert_eq!(criterion.code(), "affinity");
        assert_eq!(criterion.parameters(), &json!({"k": 1}));

        criterion.set_enabled(true);
        assert!(criterion.enabled());
    }
}
