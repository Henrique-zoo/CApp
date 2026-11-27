//! Testes de integração do backend com PostgreSQL real e provedores HTTP locais.
//!
//! Execute `cargo test --test integration --locked` com Docker. As dependências
//! de desenvolvimento habilitam os provedores locais da feature `test-support`.

#[cfg(feature = "test-support")]
mod http;
#[path = "../fixtures/materials/mod.rs"]
mod materials_fixtures;
mod repositories;
mod support;
