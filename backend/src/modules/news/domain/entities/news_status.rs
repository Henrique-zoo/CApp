//! Estado e ciclo de vida de uma notícia no CApp.
//!
//! Representa as fases editoriais pelas quais um artigo de notícia transita,
//! desde o rascunho inicial até a publicação ou arquivamento permanente.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::modules::news::domain::rules::NewsDomainError;

/// Estados editoriais possíveis para uma notícia ([`super::NewsArticle`]).
///
/// O ciclo de vida segue a especificação institucional:
/// - [`NewsStatus::Draft`]: notícia em preparação, visível apenas para o autor e gestores.
/// - [`NewsStatus::PendingReview`]: proposta submetida, aguardando homologação da diretoria.
/// - [`NewsStatus::Published`]: notícia homologada e disponível nos feeds autorizados.
/// - [`NewsStatus::Rejected`]: proposta analisada e recusada na revisão.
/// - [`NewsStatus::Archived`]: notícia retirada de circulação e mantida para fins históricos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NewsStatus {
    /// Rascunho inicial em elaboração.
    Draft,
    /// Proposta submetida para homologação pela gestão do Centro Acadêmico.
    PendingReview,
    /// Notícia aprovada e divulgada nos murais e feeds.
    Published,
    /// Proposta que não foi aceita no processo de homologação.
    Rejected,
    /// Notícia arquivada e retirada da listagem corrente.
    Archived,
}

impl NewsStatus {
    /// Retorna a representação textual padronizada em `snake_case`.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::PendingReview => "pending_review",
            Self::Published => "published",
            Self::Rejected => "rejected",
            Self::Archived => "archived",
        }
    }

    /// Avalia se a transição para o estado de destino é válida segundo as invariantes de negócio.
    ///
    /// # Regras de Transição
    ///
    /// - `Draft` -> `PendingReview`, `Published`, `Archived`
    /// - `PendingReview` -> `Published`, `Rejected`, `Draft`, `Archived`
    /// - `Rejected` -> `Draft`, `PendingReview`, `Archived`
    /// - `Published` -> `Archived`
    /// - `Archived` é terminal: nenhuma transição a partir deste estado é permitida.
    /// - Transição para o mesmo estado não é autorizada.
    #[must_use]
    pub fn can_transition_to(&self, target: Self) -> bool {
        if *self == target {
            return false;
        }

        match self {
            Self::Draft => matches!(
                target,
                Self::PendingReview | Self::Published | Self::Archived
            ),
            Self::PendingReview => matches!(
                target,
                Self::Published | Self::Rejected | Self::Draft | Self::Archived
            ),
            Self::Rejected => matches!(target, Self::Draft | Self::PendingReview | Self::Archived),
            Self::Published => matches!(target, Self::Archived),
            Self::Archived => false,
        }
    }

    /// Valida e autoriza formalmente a transição para o estado especificado.
    ///
    /// # Erros
    ///
    /// Retorna [`NewsDomainError::InvalidStatusTransition`] se a transição não for permitida.
    pub fn validate_transition(&self, target: Self) -> Result<(), NewsDomainError> {
        if self.can_transition_to(target) {
            Ok(())
        } else {
            Err(NewsDomainError::InvalidStatusTransition {
                current: *self,
                target,
            })
        }
    }

    /// Indica se o status atual representa um registro arquivado.
    #[must_use]
    pub fn is_archived(&self) -> bool {
        matches!(self, Self::Archived)
    }

    /// Indica se o status atual representa uma notícia pública e disponível.
    #[must_use]
    pub fn is_published(&self) -> bool {
        matches!(self, Self::Published)
    }

    /// Indica se o status atual é um rascunho.
    #[must_use]
    pub fn is_draft(&self) -> bool {
        matches!(self, Self::Draft)
    }

    /// Indica se o status atual está pendente de revisão.
    #[must_use]
    pub fn is_pending_review(&self) -> bool {
        matches!(self, Self::PendingReview)
    }
}

impl fmt::Display for NewsStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for NewsStatus {
    type Err = NewsDomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(Self::Draft),
            "pending_review" => Ok(Self::PendingReview),
            "published" => Ok(Self::Published),
            "rejected" => Ok(Self::Rejected),
            "archived" => Ok(Self::Archived),
            _ => Err(NewsDomainError::UnknownStatus(s.to_string())),
        }
    }
}

impl TryFrom<&str> for NewsStatus {
    type Error = NewsDomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deve_permitir_transicoes_validas_a_partir_de_draft() {
        assert!(NewsStatus::Draft.can_transition_to(NewsStatus::PendingReview));
        assert!(NewsStatus::Draft.can_transition_to(NewsStatus::Published));
        assert!(NewsStatus::Draft.can_transition_to(NewsStatus::Archived));

        assert!(!NewsStatus::Draft.can_transition_to(NewsStatus::Draft));
        assert!(!NewsStatus::Draft.can_transition_to(NewsStatus::Rejected));
    }

    #[test]
    fn deve_permitir_transicoes_validas_a_partir_de_pending_review() {
        assert!(NewsStatus::PendingReview.can_transition_to(NewsStatus::Published));
        assert!(NewsStatus::PendingReview.can_transition_to(NewsStatus::Rejected));
        assert!(NewsStatus::PendingReview.can_transition_to(NewsStatus::Draft));
        assert!(NewsStatus::PendingReview.can_transition_to(NewsStatus::Archived));

        assert!(!NewsStatus::PendingReview.can_transition_to(NewsStatus::PendingReview));
    }

    #[test]
    fn deve_permitir_apenas_arquivamento_a_partir_de_published() {
        assert!(NewsStatus::Published.can_transition_to(NewsStatus::Archived));

        assert!(!NewsStatus::Published.can_transition_to(NewsStatus::Draft));
        assert!(!NewsStatus::Published.can_transition_to(NewsStatus::PendingReview));
        assert!(!NewsStatus::Published.can_transition_to(NewsStatus::Rejected));
        assert!(!NewsStatus::Published.can_transition_to(NewsStatus::Published));
    }

    #[test]
    fn deve_permitir_transicoes_validas_a_partir_de_rejected() {
        assert!(NewsStatus::Rejected.can_transition_to(NewsStatus::Draft));
        assert!(NewsStatus::Rejected.can_transition_to(NewsStatus::PendingReview));
        assert!(NewsStatus::Rejected.can_transition_to(NewsStatus::Archived));

        assert!(!NewsStatus::Rejected.can_transition_to(NewsStatus::Published));
        assert!(!NewsStatus::Rejected.can_transition_to(NewsStatus::Rejected));
    }

    #[test]
    fn deve_bloquear_qualquer_transicao_a_partir_de_archived() {
        assert!(!NewsStatus::Archived.can_transition_to(NewsStatus::Draft));
        assert!(!NewsStatus::Archived.can_transition_to(NewsStatus::PendingReview));
        assert!(!NewsStatus::Archived.can_transition_to(NewsStatus::Published));
        assert!(!NewsStatus::Archived.can_transition_to(NewsStatus::Rejected));
        assert!(!NewsStatus::Archived.can_transition_to(NewsStatus::Archived));
    }

    #[test]
    fn deve_converter_corretamente_entre_string_e_status() {
        assert_eq!(NewsStatus::Draft.as_str(), "draft");
        assert_eq!(NewsStatus::PendingReview.as_str(), "pending_review");
        assert_eq!(NewsStatus::Published.as_str(), "published");
        assert_eq!(NewsStatus::Rejected.as_str(), "rejected");
        assert_eq!(NewsStatus::Archived.as_str(), "archived");

        assert_eq!("draft".parse::<NewsStatus>().unwrap(), NewsStatus::Draft);
        assert_eq!(
            "published".parse::<NewsStatus>().unwrap(),
            NewsStatus::Published
        );
        assert!("status_invalido".parse::<NewsStatus>().is_err());
    }

    #[test]
    fn deve_identificar_estados_corretamente_com_predicados() {
        assert!(NewsStatus::Draft.is_draft());
        assert!(!NewsStatus::Draft.is_published());

        assert!(NewsStatus::PendingReview.is_pending_review());
        assert!(!NewsStatus::PendingReview.is_archived());

        assert!(NewsStatus::Published.is_published());
        assert!(!NewsStatus::Published.is_draft());

        assert!(NewsStatus::Archived.is_archived());
        assert!(!NewsStatus::Archived.is_published());
    }
}
