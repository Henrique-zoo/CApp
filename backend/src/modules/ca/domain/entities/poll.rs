//! Entidades de domínio para consultas e enquetes estudantis do CA.
//!
//! Representa as entidades [`StudentPoll`], [`PollOption`], [`PollVote`] e os resultados
//! apurados ([`PollResults`]), garantindo integridade de regras puras de negócio.

use std::{collections::HashMap, error::Error, fmt, str::FromStr};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::ca::domain::rules::{
    validate_poll_description, validate_poll_options, validate_poll_period, validate_poll_title,
};

/// Situação do ciclo de vida da consulta/enquete estudantil.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PollStatus {
    /// A consulta está aberta para participação e votação dentro do período válido.
    #[default]
    Open,
    /// A consulta foi encerrada e não aceita mais votos.
    Closed,
}

impl fmt::Display for PollStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Closed => write!(f, "closed"),
        }
    }
}

impl FromStr for PollStatus {
    type Err = PollError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "open" | "aberta" => Ok(Self::Open),
            "closed" | "encerrada" => Ok(Self::Closed),
            other => Err(PollError::InvalidStatus(other.to_string())),
        }
    }
}

/// Erros decorrentes de violações de regras de negócio de consultas e enquetes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PollError {
    /// O título da consulta não pode ser vazio.
    EmptyTitle,
    /// O título da consulta ultrapassou o comprimento máximo permitido.
    TitleTooLong {
        /// Limite máximo em caracteres.
        max: usize,
    },
    /// A descrição da consulta ultrapassou o comprimento máximo permitido.
    DescriptionTooLong {
        /// Limite máximo em caracteres.
        max: usize,
    },
    /// O texto da opção de voto não pode ser vazio.
    EmptyOptionTitle,
    /// O texto da opção de voto ultrapassou o comprimento máximo permitido.
    OptionTitleTooLong {
        /// Limite máximo em caracteres.
        max: usize,
    },
    /// A enquete deve conter um número mínimo de opções distintas.
    TooFewOptions {
        /// Quantidade mínima exigida.
        min: usize,
        /// Quantidade encontrada.
        found: usize,
    },
    /// A enquete não permite opções com títulos duplicados.
    DuplicateOptionTitles(String),
    /// A opção de voto indicada não pertence a esta consulta.
    OptionNotFound(Uuid),
    /// As datas de início e encerramento violam a coerência temporal.
    InvalidPeriod {
        /// Instante de início.
        starts_at: DateTime<Utc>,
        /// Instante de término.
        ends_at: DateTime<Utc>,
    },
    /// A votação da consulta ainda não foi iniciada.
    VotingNotStarted {
        /// Instante configurado para início da votação.
        starts_at: DateTime<Utc>,
    },
    /// O período de votação da consulta já expirou.
    VotingEnded {
        /// Instante configurado para término da votação.
        ends_at: DateTime<Utc>,
    },
    /// A consulta já se encontra encerrada para votação.
    PollClosed,
    /// O estudante já registrou seu voto nesta consulta.
    DuplicateVote {
        /// Identificador da consulta.
        poll_id: Uuid,
        /// Identificador do usuário votante.
        user_id: Uuid,
    },
    /// Status de consulta desconhecido ou inválido.
    InvalidStatus(String),
    /// Inconsistência nos timestamps da entidade.
    InconsistentTimestamps,
}

impl fmt::Display for PollError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTitle => write!(f, "o título da consulta não pode ser vazio"),
            Self::TitleTooLong { max } => {
                write!(
                    f,
                    "o título da consulta excede o limite de {max} caracteres"
                )
            }
            Self::DescriptionTooLong { max } => {
                write!(
                    f,
                    "a descrição da consulta excede o limite de {max} caracteres"
                )
            }
            Self::EmptyOptionTitle => write!(f, "o texto da opção de voto não pode ser vazio"),
            Self::OptionTitleTooLong { max } => {
                write!(
                    f,
                    "o texto da opção de voto excede o limite de {max} caracteres"
                )
            }
            Self::TooFewOptions { min, found } => {
                write!(
                    f,
                    "a consulta exige ao menos {min} opções de voto, mas foram informadas {found}"
                )
            }
            Self::DuplicateOptionTitles(opt) => {
                write!(f, "a consulta contém opções de voto duplicadas: '{opt}'")
            }
            Self::OptionNotFound(id) => {
                write!(
                    f,
                    "opção de voto '{id}' não foi encontrada nesta consulta estudantil"
                )
            }
            Self::InvalidPeriod { starts_at, ends_at } => {
                write!(
                    f,
                    "período de votação inválido: encerramento ({ends_at}) deve ser posterior ao início ({starts_at})"
                )
            }
            Self::VotingNotStarted { starts_at } => {
                write!(
                    f,
                    "o período de votação desta consulta ainda não começou (início em: {starts_at})"
                )
            }
            Self::VotingEnded { ends_at } => {
                write!(
                    f,
                    "o período de votação desta consulta já terminou (término em: {ends_at})"
                )
            }
            Self::PollClosed => write!(f, "esta consulta estudantil encontra-se encerrada"),
            Self::DuplicateVote { poll_id, user_id } => {
                write!(
                    f,
                    "o estudante '{user_id}' já votou na consulta '{poll_id}'"
                )
            }
            Self::InvalidStatus(status) => {
                write!(f, "status de consulta estudantil inválido: '{status}'")
            }
            Self::InconsistentTimestamps => {
                write!(f, "timestamps de consulta violam coerência temporal")
            }
        }
    }
}

impl Error for PollError {}

/// Opção de voto pertencente a uma consulta estudantil.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PollOption {
    id: Uuid,
    poll_id: Uuid,
    title: String,
    order: i32,
    created_at: DateTime<Utc>,
}

impl PollOption {
    /// Constrói uma nova opção validada para uma consulta.
    pub fn new(id: Uuid, poll_id: Uuid, title: String, order: i32) -> Result<Self, PollError> {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return Err(PollError::EmptyOptionTitle);
        }
        if trimmed.chars().count() > 255 {
            return Err(PollError::OptionTitleTooLong { max: 255 });
        }
        Ok(Self {
            id,
            poll_id,
            title: trimmed.to_string(),
            order,
            created_at: Utc::now(),
        })
    }

    /// Reconstitui uma opção a partir da persistência.
    pub fn reconstitute(
        id: Uuid,
        poll_id: Uuid,
        title: String,
        order: i32,
        created_at: DateTime<Utc>,
    ) -> Result<Self, PollError> {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return Err(PollError::EmptyOptionTitle);
        }
        Ok(Self {
            id,
            poll_id,
            title: trimmed.to_string(),
            order,
            created_at,
        })
    }

    /// Identificador único da opção de voto.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Identificador da consulta à qual a opção pertence.
    pub fn poll_id(&self) -> Uuid {
        self.poll_id
    }

    /// Texto descritivo da opção de voto.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Ordem de apresentação da opção.
    pub fn order(&self) -> i32 {
        self.order
    }

    /// Instante de criação da opção.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

/// Registro de voto/participação de um estudante na consulta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PollVote {
    id: Uuid,
    poll_id: Uuid,
    option_id: Uuid,
    user_id: Uuid,
    voted_at: DateTime<Utc>,
}

impl PollVote {
    /// Cria um novo registro de voto.
    pub fn new(poll_id: Uuid, option_id: Uuid, user_id: Uuid, voted_at: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            poll_id,
            option_id,
            user_id,
            voted_at,
        }
    }

    /// Reconstitui um voto existente a partir da persistência.
    pub fn reconstitute(
        id: Uuid,
        poll_id: Uuid,
        option_id: Uuid,
        user_id: Uuid,
        voted_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            poll_id,
            option_id,
            user_id,
            voted_at,
        }
    }

    /// Identificador único do voto.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Identificador da consulta votada.
    pub fn poll_id(&self) -> Uuid {
        self.poll_id
    }

    /// Identificador da opção escolhida.
    pub fn option_id(&self) -> Uuid {
        self.option_id
    }

    /// Identificador do estudante votante.
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    /// Instante em que o voto foi depositado.
    pub fn voted_at(&self) -> DateTime<Utc> {
        self.voted_at
    }
}

/// Resultado apurado de uma opção de voto.
#[derive(Debug, Clone, PartialEq)]
pub struct PollOptionResult {
    /// Identificador da opção.
    pub id: Uuid,
    /// Texto da opção.
    pub title: String,
    /// Ordem de exibição.
    pub order: i32,
    /// Quantidade absoluta de votos recebidos.
    pub vote_count: i64,
    /// Percentual apurado em relação ao total de votos (0.0 a 100.0).
    pub percentage: f64,
}

/// Resultado consolidado da consulta/enquete estudantil.
#[derive(Debug, Clone, PartialEq)]
pub struct PollResults {
    /// Identificador da consulta.
    pub poll_id: Uuid,
    /// Total de votos válidos depositados.
    pub total_votes: i64,
    /// Resultados por opção de voto.
    pub options: Vec<PollOptionResult>,
}

/// Entidade raiz de domínio representando uma Consulta/Enquete Estudantil do CA.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudentPoll {
    id: Uuid,
    academic_center_id: Uuid,
    created_by_id: Uuid,
    title: String,
    description: Option<String>,
    target_audience: String,
    vote_modality: String,
    starts_at: DateTime<Utc>,
    ends_at: Option<DateTime<Utc>>,
    status: PollStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    closed_at: Option<DateTime<Utc>>,
    closed_by_id: Option<Uuid>,
    options: Vec<PollOption>,
}

impl StudentPoll {
    /// Cria uma nova consulta estudantil com suas opções de voto validadas.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        academic_center_id: Uuid,
        created_by_id: Uuid,
        title: String,
        description: Option<String>,
        raw_options: Vec<String>,
        starts_at: DateTime<Utc>,
        ends_at: Option<DateTime<Utc>>,
        target_audience: Option<String>,
        vote_modality: Option<String>,
    ) -> Result<Self, PollError> {
        validate_poll_title(&title)?;
        validate_poll_description(description.as_deref())?;
        validate_poll_options(&raw_options)?;
        validate_poll_period(starts_at, ends_at)?;

        let poll_id = Uuid::new_v4();
        let now = Utc::now();

        let mut options = Vec::with_capacity(raw_options.len());
        for (idx, opt_title) in raw_options.into_iter().enumerate() {
            let option = PollOption::new(Uuid::new_v4(), poll_id, opt_title, (idx + 1) as i32)?;
            options.push(option);
        }

        let audience = target_audience
            .map(|a| a.trim().to_string())
            .filter(|a| !a.is_empty())
            .unwrap_or_else(|| "all_students".to_string());

        let modality = vote_modality
            .map(|m| m.trim().to_lowercase())
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| "identified".to_string());

        Ok(Self {
            id: poll_id,
            academic_center_id,
            created_by_id,
            title: title.trim().to_string(),
            description: description
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty()),
            target_audience: audience,
            vote_modality: modality,
            starts_at,
            ends_at,
            status: PollStatus::Open,
            created_at: now,
            updated_at: now,
            closed_at: None,
            closed_by_id: None,
            options,
        })
    }

    /// Reconstitui uma consulta estudantil e suas opções a partir do armazenamento.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: Uuid,
        academic_center_id: Uuid,
        created_by_id: Uuid,
        title: String,
        description: Option<String>,
        target_audience: String,
        vote_modality: String,
        starts_at: DateTime<Utc>,
        ends_at: Option<DateTime<Utc>>,
        status: PollStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        closed_at: Option<DateTime<Utc>>,
        closed_by_id: Option<Uuid>,
        options: Vec<PollOption>,
    ) -> Result<Self, PollError> {
        validate_poll_title(&title)?;
        validate_poll_description(description.as_deref())?;
        validate_poll_period(starts_at, ends_at)?;

        if updated_at < created_at {
            return Err(PollError::InconsistentTimestamps);
        }

        Ok(Self {
            id,
            academic_center_id,
            created_by_id,
            title: title.trim().to_string(),
            description: description
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty()),
            target_audience: target_audience.trim().to_string(),
            vote_modality: vote_modality.trim().to_string(),
            starts_at,
            ends_at,
            status,
            created_at,
            updated_at,
            closed_at,
            closed_by_id,
            options,
        })
    }

    /// Verifica se a consulta está aberta para votação no instante informado.
    ///
    /// # Erros
    ///
    /// Retorna [`PollError::PollClosed`] se o status for [`PollStatus::Closed`].
    /// Retorna [`PollError::VotingNotStarted`] se `now < starts_at`.
    /// Retorna [`PollError::VotingEnded`] se houver `ends_at` e `now >= ends_at`.
    pub fn can_vote(&self, now: DateTime<Utc>) -> Result<(), PollError> {
        if self.status == PollStatus::Closed {
            return Err(PollError::PollClosed);
        }
        if now < self.starts_at {
            return Err(PollError::VotingNotStarted {
                starts_at: self.starts_at,
            });
        }
        if let Some(ends_at) = self.ends_at.filter(|&end| now >= end) {
            return Err(PollError::VotingEnded { ends_at });
        }
        Ok(())
    }

    /// Encerra a consulta manualmente por um integrante autorizado da gestão.
    pub fn close(&mut self, closed_by_id: Uuid, closed_at: DateTime<Utc>) -> Result<(), PollError> {
        if self.status == PollStatus::Closed {
            return Err(PollError::PollClosed);
        }
        self.status = PollStatus::Closed;
        self.closed_at = Some(closed_at);
        self.closed_by_id = Some(closed_by_id);
        self.updated_at = closed_at;
        Ok(())
    }

    /// Recupera uma opção de voto pertencente à consulta pelo seu identificador.
    pub fn find_option(&self, option_id: Uuid) -> Option<&PollOption> {
        self.options.iter().find(|opt| opt.id == option_id)
    }

    /// Verifica se a opção indicada pertence a esta consulta.
    pub fn has_option(&self, option_id: Uuid) -> bool {
        self.find_option(option_id).is_some()
    }

    /// Apura os resultados da consulta com base no mapa de contagem de votos por opção.
    pub fn tally_results(&self, vote_counts: &HashMap<Uuid, i64>) -> PollResults {
        let total_votes: i64 = vote_counts.values().sum();

        let mut results = Vec::with_capacity(self.options.len());
        for opt in &self.options {
            let count = vote_counts.get(&opt.id).copied().unwrap_or(0);
            let percentage = if total_votes > 0 {
                let raw = ((count as f64) / (total_votes as f64)) * 100.0;
                (raw * 100.0).round() / 100.0
            } else {
                0.0
            };

            results.push(PollOptionResult {
                id: opt.id,
                title: opt.title.clone(),
                order: opt.order,
                vote_count: count,
                percentage,
            });
        }

        // Ordena por ordem de exibição das opções
        results.sort_by_key(|r| r.order);

        PollResults {
            poll_id: self.id,
            total_votes,
            options: results,
        }
    }

    // Getters

    /// Identificador único da consulta.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Identificador do Centro Acadêmico ao qual a consulta pertence.
    pub fn academic_center_id(&self) -> Uuid {
        self.academic_center_id
    }

    /// Identificador do usuário gestor que criou a consulta.
    pub fn created_by_id(&self) -> Uuid {
        self.created_by_id
    }

    /// Título oficial da consulta.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Descrição institucional ou proposta em discussão.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Público-alvo / critério de elegibilidade configurado para a consulta.
    pub fn target_audience(&self) -> &str {
        &self.target_audience
    }

    /// Modalidade da votação ('identified' ou 'secret').
    pub fn vote_modality(&self) -> &str {
        &self.vote_modality
    }

    /// Início do período de votação da consulta.
    pub fn starts_at(&self) -> DateTime<Utc> {
        self.starts_at
    }

    /// Término do período de votação da consulta (opcional).
    pub fn ends_at(&self) -> Option<DateTime<Utc>> {
        self.ends_at
    }

    /// Situação atual da consulta ([`PollStatus`]).
    pub fn status(&self) -> PollStatus {
        self.status
    }

    /// Instante de criação da consulta.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Instante da última modificação.
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Instante do encerramento manual ou programado.
    pub fn closed_at(&self) -> Option<DateTime<Utc>> {
        self.closed_at
    }

    /// Identificador do gestor que encerrou a consulta.
    pub fn closed_by_id(&self) -> Option<Uuid> {
        self.closed_by_id
    }

    /// Opções de voto disponíveis na consulta.
    pub fn options(&self) -> &[PollOption] {
        &self.options
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    #[test]
    fn should_create_valid_poll() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let poll = StudentPoll::new(
            ca_id,
            user_id,
            "Ampliação do horário dos laboratórios".to_string(),
            Some("Manter os laboratórios abertos até as 22h".to_string()),
            vec![
                "Apoio a proposta".to_string(),
                "Não apoio a proposta".to_string(),
            ],
            now,
            Some(now + Duration::days(5)),
            Some("all_students".to_string()),
            Some("identified".to_string()),
        )
        .expect("should create valid poll");

        assert_eq!(poll.academic_center_id(), ca_id);
        assert_eq!(poll.created_by_id(), user_id);
        assert_eq!(poll.title(), "Ampliação do horário dos laboratórios");
        assert_eq!(
            poll.description(),
            Some("Manter os laboratórios abertos até as 22h")
        );
        assert_eq!(poll.status(), PollStatus::Open);
        assert_eq!(poll.options().len(), 2);
        assert_eq!(poll.options()[0].title(), "Apoio a proposta");
        assert_eq!(poll.options()[1].title(), "Não apoio a proposta");
    }

    #[test]
    fn should_validate_can_vote_time_window() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        // 1. Enquete com início futuro
        let future_poll = StudentPoll::new(
            ca_id,
            user_id,
            "Enquete futura".to_string(),
            None,
            vec!["Sim".to_string(), "Não".to_string()],
            now + Duration::days(1),
            Some(now + Duration::days(3)),
            None,
            None,
        )
        .unwrap();

        assert!(matches!(
            future_poll.can_vote(now),
            Err(PollError::VotingNotStarted { .. })
        ));

        // 2. Enquete com período expirado
        let expired_poll = StudentPoll::new(
            ca_id,
            user_id,
            "Enquete expirada".to_string(),
            None,
            vec!["Sim".to_string(), "Não".to_string()],
            now - Duration::days(5),
            Some(now - Duration::days(1)),
            None,
            None,
        )
        .unwrap();

        assert!(matches!(
            expired_poll.can_vote(now),
            Err(PollError::VotingEnded { .. })
        ));

        // 3. Enquete no período válido
        let active_poll = StudentPoll::new(
            ca_id,
            user_id,
            "Enquete ativa".to_string(),
            None,
            vec!["Sim".to_string(), "Não".to_string()],
            now - Duration::hours(1),
            Some(now + Duration::days(1)),
            None,
            None,
        )
        .unwrap();

        assert!(active_poll.can_vote(now).is_ok());

        // 4. Enquete sem prazo de encerramento
        let open_indefinite_poll = StudentPoll::new(
            ca_id,
            user_id,
            "Sem prazo".to_string(),
            None,
            vec!["Sim".to_string(), "Não".to_string()],
            now - Duration::hours(1),
            None,
            None,
            None,
        )
        .unwrap();

        assert!(open_indefinite_poll.can_vote(now).is_ok());
    }

    #[test]
    fn should_close_poll_and_block_new_votes() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let mut poll = StudentPoll::new(
            ca_id,
            user_id,
            "Enquete a fechar".to_string(),
            None,
            vec!["Opção A".to_string(), "Opção B".to_string()],
            now - Duration::hours(1),
            Some(now + Duration::days(1)),
            None,
            None,
        )
        .unwrap();

        assert!(poll.can_vote(now).is_ok());

        poll.close(user_id, now).expect("should close poll");
        assert_eq!(poll.status(), PollStatus::Closed);
        assert_eq!(poll.closed_at(), Some(now));
        assert_eq!(poll.closed_by_id(), Some(user_id));

        assert_eq!(poll.can_vote(now), Err(PollError::PollClosed));

        // Fechar novamente resulta em PollClosed
        assert_eq!(poll.close(user_id, now), Err(PollError::PollClosed));
    }

    #[test]
    fn should_tally_results_correctly() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let poll = StudentPoll::new(
            ca_id,
            user_id,
            "Votação com apuração".to_string(),
            None,
            vec![
                "Apoio a proposta".to_string(),
                "Não apoio a proposta".to_string(),
                "Abstenção".to_string(),
            ],
            now,
            None,
            None,
            None,
        )
        .unwrap();

        let opt_1 = poll.options()[0].id();
        let opt_2 = poll.options()[1].id();
        let opt_3 = poll.options()[2].id();

        let mut counts = HashMap::new();
        counts.insert(opt_1, 2);
        counts.insert(opt_2, 1);
        counts.insert(opt_3, 0);

        let results = poll.tally_results(&counts);

        assert_eq!(results.total_votes, 3);
        assert_eq!(results.options.len(), 3);

        let r1 = results.options.iter().find(|o| o.id == opt_1).unwrap();
        assert_eq!(r1.vote_count, 2);
        assert_eq!(r1.percentage, 66.67);

        let r2 = results.options.iter().find(|o| o.id == opt_2).unwrap();
        assert_eq!(r2.vote_count, 1);
        assert_eq!(r2.percentage, 33.33);

        let r3 = results.options.iter().find(|o| o.id == opt_3).unwrap();
        assert_eq!(r3.vote_count, 0);
        assert_eq!(r3.percentage, 0.0);
    }
}
