//! Entidade de domínio para o contexto ativo de navegação em Centro Acadêmico.
//!
//! O contexto ativo define qual Centro Acadêmico está selecionado na interface
//! durante a sessão do usuário. Em conformidade com o Relatório de Features § 2.1,
//! a simples alternância de contexto é um facilitador de navegação e **NÃO** confere
//! privilégios de gestão nem altera a preferência de CA favorito gravada.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Erros relacionados à validação do contexto ativo de navegação.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ActiveContextError {
    /// O identificador do usuário não pode ser o UUID nulo.
    #[error("O identificador do usuário não pode ser nulo.")]
    NilUserId,

    /// O identificador do Centro Acadêmico não pode ser o UUID nulo.
    #[error("O identificador do Centro Acadêmico não pode ser nulo.")]
    NilAcademicCenterId,
}

/// Origem da determinação do contexto ativo na sessão.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActiveContextSource {
    /// Inicializado a partir da preferência gravada de CA favorito.
    FavoritePreference,

    /// Alternado explicitamente pelo usuário durante a navegação.
    ExplicitSelection,

    /// Inicializado com o contexto padrão quando não há favorito configurado.
    DefaultNavigation,
}

/// Entidade de domínio que expressa o contexto ativo de um estudante.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveCaContext {
    /// Identificador do usuário navegando na aplicação.
    pub user_id: Uuid,

    /// Identificador do Centro Acadêmico atualmente ativo.
    pub active_ca_id: Uuid,

    /// Origem do contexto selecionado na sessão.
    pub source: ActiveContextSource,

    /// Instante da última alternância de contexto.
    pub updated_at: DateTime<Utc>,
}

impl ActiveCaContext {
    /// Cria uma nova instância de contexto ativo a partir de seleção explícita.
    ///
    /// # Erros
    ///
    /// Retorna [`ActiveContextError::NilUserId`] ou [`ActiveContextError::NilAcademicCenterId`].
    pub fn new(user_id: Uuid, active_ca_id: Uuid) -> Result<Self, ActiveContextError> {
        Self::with_source(
            user_id,
            active_ca_id,
            ActiveContextSource::ExplicitSelection,
        )
    }

    /// Cria uma nova instância de contexto ativo indicando a origem.
    pub fn with_source(
        user_id: Uuid,
        active_ca_id: Uuid,
        source: ActiveContextSource,
    ) -> Result<Self, ActiveContextError> {
        if user_id.is_nil() {
            return Err(ActiveContextError::NilUserId);
        }
        if active_ca_id.is_nil() {
            return Err(ActiveContextError::NilAcademicCenterId);
        }

        Ok(Self {
            user_id,
            active_ca_id,
            source,
            updated_at: Utc::now(),
        })
    }

    /// Retorna o identificador do usuário associado a este contexto.
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    /// Retorna o identificador do Centro Acadêmico atualmente selecionado.
    pub fn active_ca_id(&self) -> Uuid {
        self.active_ca_id
    }

    /// Verifica se o Centro Acadêmico informado corresponde ao contexto ativo atual.
    pub fn is_active(&self, ca_id: &Uuid) -> bool {
        &self.active_ca_id == ca_id
    }

    /// Confirma a invariante de negócio: o contexto ativo destina-se apenas à navegação.
    pub fn is_navigation_only(&self) -> bool {
        true
    }

    /// Confirma a invariante de negócio: o contexto ativo por si só NÃO concede poderes de gestão.
    pub fn confers_administrative_permissions(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DUMMY_USER_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);
    const DUMMY_CA_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0001);

    #[test]
    fn should_create_active_context_and_validate_invariants() {
        let ctx = ActiveCaContext::new(DUMMY_USER_ID, DUMMY_CA_ID).expect("contexto válido");
        assert_eq!(ctx.user_id(), DUMMY_USER_ID);
        assert_eq!(ctx.active_ca_id(), DUMMY_CA_ID);
        assert!(ctx.is_active(&DUMMY_CA_ID));
        assert!(ctx.is_navigation_only());
        assert!(!ctx.confers_administrative_permissions());
    }

    #[test]
    fn should_reject_nil_identifiers() {
        assert_eq!(
            ActiveCaContext::new(Uuid::nil(), DUMMY_CA_ID).unwrap_err(),
            ActiveContextError::NilUserId
        );
        assert_eq!(
            ActiveCaContext::new(DUMMY_USER_ID, Uuid::nil()).unwrap_err(),
            ActiveContextError::NilAcademicCenterId
        );
    }
}
