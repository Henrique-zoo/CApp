//! Camada de API prevista para o módulo de Centros Acadêmicos.
//!
//! Contém os extratores, middlewares e adaptadores HTTP para controle de acesso ao CA.

pub mod dto;
pub mod handlers;

pub use crate::api::auth::{
    ActiveCaContext, AuthorizationErrorResponse, RequirePermission, require_permission_middleware,
};
