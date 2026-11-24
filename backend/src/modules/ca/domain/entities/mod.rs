//! Entidades do módulo de Centros Acadêmicos.
//!
//! Reúne os modelos de domínio do Centro Acadêmico, mandatos de gestão,
//! ciclo de vida, invariantes e integridade estrutural.

pub mod academic_center;
mod mandate;
pub use academic_center::*;
pub use mandate::*;
pub use poll::*;
mod poll;

pub mod active_context;
pub mod authorization_context;
pub mod context_center;
pub mod permission;
pub mod role;
pub mod role_assignment;
pub mod scope;

pub use active_context::*;
pub use permission::{Permission, PermissionError};
pub use role::{Role, RoleError};
pub use role_assignment::{RoleAssignment, RoleAssignmentError};
pub use scope::{Scope, ScopeError};
