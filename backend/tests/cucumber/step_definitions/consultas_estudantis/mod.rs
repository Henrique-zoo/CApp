//! Registro dos steps pendentes das consultas estudantis (#302).
//!
//! Os submódulos acompanham as features de configuração, votação, discussão e
//! resultados. `common` registra uma única vez os passos usados por mais de uma
//! funcionalidade. Cada step chama `pending_step`, sem mocks ou aprovação simulada.
//! Todas as features mantêm `@pending_backend` até existir automação real.

use super::pending_step;

mod common;
mod configuracao;
mod discussao;
mod resultados;
mod votacao;
