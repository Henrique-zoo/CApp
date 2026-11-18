//! Programa de apadrinhamento definido por um CA.
//!
//! Guarda a configuração e aplica as invariantes de estado, datas, limite de
//! padrinhos por calouro e alcance entre cursos. Não cria participantes, vínculos
//! nem execuções de matching; esses registros consultam os predicados desta
//! entidade. CA e usuário criador chegam como `Uuid` — não são criados aqui.

use crate::modules::mentorship::domain::{MatchingStrategy, MentorshipProgramStatus};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Fotografia persistida do programa de apadrinhamento. Não é um contrato HTTP.
///
/// Campos privados expõem leitura por métodos e alteração pelos caminhos
/// validados de criação, abertura, encerramento e reconfiguração. A reconstituição
/// PostgreSQL usa o mapeamento `sqlx::FromRow` adotado no projeto.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MentorshipProgram {
    id: Uuid,
    ca_id: Uuid,
    created_by_id: Uuid,
    name: String,
    status: MentorshipProgramStatus,
    matching_strategy: MatchingStrategy,
    mentors_per_mentee: i32,
    allow_other_courses: bool,
    starts_at: DateTime<Utc>,
    ends_at: Option<DateTime<Utc>>,
}

/// Dados exigidos para criar ou reconfigurar um programa.
///
/// Criar este agrupamento, inclusive com `Default`, não comprova validade.
/// A existência do CA e a permissão de gestão pertencem ao caso de uso.
#[derive(Debug, Clone)]
pub struct ProgramConfig {
    /// Nome apresentado do programa; será aparado antes da persistência.
    pub name: String,
    /// Método ativo de formação de sugestões ou pares.
    pub matching_strategy: MatchingStrategy,
    /// Quantidade máxima de padrinhos vigentes por calouro; deve ser positiva.
    pub mentors_per_mentee: i32,
    /// Se o programa admite veteranos de outros cursos.
    pub allow_other_courses: bool,
    /// Início da vigência operacional do programa.
    pub starts_at: DateTime<Utc>,
    /// Término opcional; quando presente, deve ser estritamente posterior a `starts_at`.
    pub ends_at: Option<DateTime<Utc>>,
}

/// Uma criação, transição ou reconfiguração que não preservaria as regras do programa.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("Configuração ou transição de programa inválida")]
pub struct InvalidProgram;

impl MentorshipProgram {
    /// Identificador interno estável deste programa.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// CA que define o contexto do programa.
    pub fn ca_id(&self) -> Uuid {
        self.ca_id
    }

    /// Usuário que criou o registro.
    pub fn created_by_id(&self) -> Uuid {
        self.created_by_id
    }

    /// Nome normalizado usado para identificar e apresentar o programa.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Estado persistido do ciclo de vida.
    pub fn status(&self) -> MentorshipProgramStatus {
        self.status
    }

    /// Método ativo de matching neste momento.
    pub fn matching_strategy(&self) -> MatchingStrategy {
        self.matching_strategy
    }

    /// Limite de padrinhos vigentes por calouro.
    pub fn mentors_per_mentee(&self) -> i32 {
        self.mentors_per_mentee
    }

    /// Indica se veteranos de outros cursos podem participar.
    pub fn allow_other_courses(&self) -> bool {
        self.allow_other_courses
    }

    /// Início da vigência operacional.
    pub fn starts_at(&self) -> DateTime<Utc> {
        self.starts_at
    }

    /// Término opcional da vigência operacional.
    pub fn ends_at(&self) -> Option<DateTime<Utc>> {
        self.ends_at
    }

    /// Cria o programa em rascunho com a configuração inicial validada.
    ///
    /// # Parâmetros e pré-condições
    ///
    /// - `id`, `ca_id` e `created_by_id` são identidades escolhidas pelo chamador.
    /// - `config`: nome não vazio após `trim`, `mentors_per_mentee` positivo e
    ///   `ends_at` estritamente posterior a `starts_at` quando informado.
    ///
    /// Não consulta o CA, não verifica permissão e não grava no banco.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidProgram`] para nome vazio, limite não positivo ou janela
    /// de datas inválida. A validação precede a construção.
    pub fn draft(
        id: Uuid,
        ca_id: Uuid,
        created_by_id: Uuid,
        config: ProgramConfig,
    ) -> Result<Self, InvalidProgram> {
        let (name, mentors_per_mentee, starts_at, ends_at) = validate_config(&config)?;

        Ok(Self {
            id,
            ca_id,
            created_by_id,
            name,
            status: MentorshipProgramStatus::Draft,
            matching_strategy: config.matching_strategy,
            mentors_per_mentee,
            allow_other_courses: config.allow_other_courses,
            starts_at,
            ends_at,
        })
    }

    /// Abre o programa: somente `draft` → `open`.
    ///
    /// Em `Ok(())`, muda o estado para [`MentorshipProgramStatus::Open`]. A
    /// configuração e as identidades não mudam. Abrir não implica aceitar
    /// inscrição imediatamente: a vigência das datas continua sendo conferida.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidProgram`] se o programa não estiver em rascunho.
    pub fn open(&mut self) -> Result<(), InvalidProgram> {
        if self.status != MentorshipProgramStatus::Draft {
            return Err(InvalidProgram);
        }
        self.status = MentorshipProgramStatus::Open;
        Ok(())
    }

    /// Encerra o programa: somente `open` → `closed`.
    ///
    /// Em `Ok(())`, muda o estado para [`MentorshipProgramStatus::Closed`].
    /// Vínculos vigentes não são encerrados por esta transição; o tratamento
    /// deles pertence à política do fluxo.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidProgram`] se o programa não estiver aberto.
    pub fn close(&mut self) -> Result<(), InvalidProgram> {
        if self.status != MentorshipProgramStatus::Open {
            return Err(InvalidProgram);
        }
        self.status = MentorshipProgramStatus::Closed;
        Ok(())
    }

    /// Atualiza a configuração enquanto o programa não estiver encerrado.
    ///
    /// Reduzir `mentors_per_mentee` não encerra vínculos já vigentes: apenas
    /// altera o limite usado nas próximas checagens. Estado e identidades
    /// permanecem intactos.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidProgram`] para programa encerrado, nome vazio, limite
    /// não positivo ou janela de datas inválida. Em erro, o objeto fica intacto.
    pub fn reconfigure(&mut self, config: ProgramConfig) -> Result<(), InvalidProgram> {
        if self.status == MentorshipProgramStatus::Closed {
            return Err(InvalidProgram);
        }

        let (name, mentors_per_mentee, starts_at, ends_at) = validate_config(&config)?;

        self.name = name;
        self.matching_strategy = config.matching_strategy;
        self.mentors_per_mentee = mentors_per_mentee;
        self.allow_other_courses = config.allow_other_courses;
        self.starts_at = starts_at;
        self.ends_at = ends_at;
        Ok(())
    }

    /// Indica se novas inscrições são aceitas neste instante.
    ///
    /// Exige status [`MentorshipProgramStatus::Open`] e instante dentro da
    /// vigência: a partir de `starts_at` e, quando houver `ends_at`, antes dele.
    pub fn accepts_enrollment(&self, now: DateTime<Utc>) -> bool {
        self.accepts_activity(now)
    }

    /// Indica se novos vínculos são aceitos neste instante.
    ///
    /// Mesmas condições de [`Self::accepts_enrollment`]: programa aberto e
    /// dentro das datas. Rascunho e encerrado recusam.
    pub fn accepts_match(&self, now: DateTime<Utc>) -> bool {
        self.accepts_activity(now)
    }

    /// Indica se uma nova execução de matching é aceita neste instante.
    ///
    /// Mesmas condições de [`Self::accepts_enrollment`]: programa aberto e
    /// dentro das datas. Rascunho e encerrado recusam.
    pub fn accepts_new_run(&self, now: DateTime<Utc>) -> bool {
        self.accepts_activity(now)
    }

    /// Programa aberto e instante dentro da vigência operacional.
    fn accepts_activity(&self, now: DateTime<Utc>) -> bool {
        self.status == MentorshipProgramStatus::Open && self.is_within_window(now)
    }

    /// `now >= starts_at` e, se houver término, `now < ends_at`.
    fn is_within_window(&self, now: DateTime<Utc>) -> bool {
        now >= self.starts_at && self.ends_at.is_none_or(|ends_at| now < ends_at)
    }
}

/// Valida e normaliza a configuração compartilhada por criação e reconfiguração.
fn validate_config(
    config: &ProgramConfig,
) -> Result<(String, i32, DateTime<Utc>, Option<DateTime<Utc>>), InvalidProgram> {
    let name = config.name.trim();
    if name.is_empty() || config.mentors_per_mentee < 1 {
        return Err(InvalidProgram);
    }
    if config
        .ends_at
        .is_some_and(|ends_at| ends_at <= config.starts_at)
    {
        return Err(InvalidProgram);
    }

    Ok((
        name.to_owned(),
        config.mentors_per_mentee,
        config.starts_at,
        config.ends_at,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(year: i32, month: u32, day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap()
    }

    fn config() -> ProgramConfig {
        ProgramConfig {
            name: " Apadrinhamento 2026.1 ".into(),
            matching_strategy: MatchingStrategy::Manual,
            mentors_per_mentee: 2,
            allow_other_courses: false,
            starts_at: at(2026, 3, 1),
            ends_at: Some(at(2026, 7, 1)),
        }
    }

    fn draft_program() -> MentorshipProgram {
        MentorshipProgram::draft(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), config()).unwrap()
    }

    #[test]
    fn draft_normalizes_name_and_rejects_invalid_config() {
        let program = draft_program();
        assert_eq!(program.name(), "Apadrinhamento 2026.1");
        assert_eq!(program.status(), MentorshipProgramStatus::Draft);
        assert_eq!(program.matching_strategy(), MatchingStrategy::Manual);
        assert_eq!(program.mentors_per_mentee(), 2);
        assert!(!program.allow_other_courses());

        for (name, mentors, ends_at) in [
            (" ", 2, Some(at(2026, 7, 1))),
            ("ok", 0, Some(at(2026, 7, 1))),
            ("ok", -1, Some(at(2026, 7, 1))),
            ("ok", 1, Some(at(2026, 3, 1))),
            ("ok", 1, Some(at(2026, 2, 1))),
        ] {
            let result = MentorshipProgram::draft(
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                ProgramConfig {
                    name: name.into(),
                    mentors_per_mentee: mentors,
                    ends_at,
                    ..config()
                },
            );
            assert_eq!(result.err(), Some(InvalidProgram));
        }
    }

    #[test]
    fn open_and_close_follow_draft_open_closed() {
        let mut program = draft_program();
        let now = at(2026, 4, 1);

        assert!(!program.accepts_enrollment(now));
        assert!(!program.accepts_match(now));
        assert!(!program.accepts_new_run(now));

        assert!(program.close().is_err());
        assert_eq!(program.status(), MentorshipProgramStatus::Draft);

        program.open().unwrap();
        assert_eq!(program.status(), MentorshipProgramStatus::Open);
        assert!(program.accepts_enrollment(now));
        assert!(program.accepts_match(now));
        assert!(program.accepts_new_run(now));

        assert!(program.open().is_err());

        program.close().unwrap();
        assert_eq!(program.status(), MentorshipProgramStatus::Closed);
        assert!(!program.accepts_enrollment(now));
        assert!(!program.accepts_match(now));
        assert!(!program.accepts_new_run(now));
        assert!(program.close().is_err());
        assert!(program.open().is_err());
    }

    #[test]
    fn open_program_respects_activity_window() {
        let mut program = draft_program();
        program.open().unwrap();

        assert!(!program.accepts_enrollment(at(2026, 2, 28)));
        assert!(program.accepts_enrollment(at(2026, 3, 1)));
        assert!(program.accepts_match(at(2026, 6, 30)));
        assert!(!program.accepts_new_run(at(2026, 7, 1)));
        assert!(!program.accepts_enrollment(at(2026, 8, 1)));
    }

    #[test]
    fn open_program_without_end_accepts_after_start() {
        let mut program = MentorshipProgram::draft(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ProgramConfig {
                ends_at: None,
                ..config()
            },
        )
        .unwrap();
        program.open().unwrap();

        assert!(program.accepts_enrollment(at(2026, 3, 1)));
        assert!(program.accepts_match(at(2030, 1, 1)));
    }

    #[test]
    fn reconfigure_works_until_closed_and_preserves_existing_limit_semantics() {
        let mut program = draft_program();

        program
            .reconfigure(ProgramConfig {
                name: " Novo nome ".into(),
                matching_strategy: MatchingStrategy::Criteria,
                mentors_per_mentee: 1,
                allow_other_courses: true,
                starts_at: at(2026, 4, 1),
                ends_at: None,
            })
            .unwrap();

        assert_eq!(program.name(), "Novo nome");
        assert_eq!(program.matching_strategy(), MatchingStrategy::Criteria);
        assert_eq!(program.mentors_per_mentee(), 1);
        assert!(program.allow_other_courses());
        assert_eq!(program.starts_at(), at(2026, 4, 1));
        assert!(program.ends_at().is_none());
        assert_eq!(program.status(), MentorshipProgramStatus::Draft);

        program.open().unwrap();
        program
            .reconfigure(ProgramConfig {
                mentors_per_mentee: 3,
                ..config()
            })
            .unwrap();
        assert_eq!(program.mentors_per_mentee(), 3);

        program.close().unwrap();
        let before = program.clone();
        assert!(program.reconfigure(config()).is_err());
        assert_eq!(program.name(), before.name());
        assert_eq!(program.mentors_per_mentee(), before.mentors_per_mentee());
        assert_eq!(program.status(), MentorshipProgramStatus::Closed);
    }
}
