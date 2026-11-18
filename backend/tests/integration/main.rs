//! Testes de integração do backend com PostgreSQL real e provedores HTTP locais.
//!
//! Execute `cargo test --test integration --features test-support` com Docker.
//! Sem `test-support`, apenas os contratos dos repositórios são compilados.

#[cfg(feature = "test-support")]
mod http;
mod repositories;
mod support;
