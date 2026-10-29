//! Entidades de domínio do módulo de Centros Acadêmicos.
//!
//! Contém as definições de escopos, permissões, cargos, vinculação de integrantes
//! e contexto ativo de navegação em sessão.

pub mod active_context;
pub mod permission;
pub mod role;
pub mod role_assignment;
pub mod scope;

pub use active_context::{ActiveCaContext, ActiveContextError, ActiveContextSource};
pub use permission::{Permission, PermissionError};
pub use role::{Role, RoleError};
pub use role_assignment::{RoleAssignment, RoleAssignmentError};
pub use scope::{Scope, ScopeError};
