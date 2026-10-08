//! Runner das verificações técnicas exclusivas do backend.
//!
//! Lê `tests/features/` e registra os steps de `infrastructure_steps`.
//! Usa o mesmo suporte da suíte funcional, mas cria seu próprio container
//! PostgreSQL por execução. O caminho dos cenários é relativo ao manifesto.
//!
//! # Execução
//!
//! Requer um daemon Docker acessível. Execute no diretório `backend`:
//!
//! ```sh
//! cargo test --test bdd_infrastructure --locked
//! cargo test --test bdd_infrastructure --locked -- --tags @ready
//! ```
//!
//! As verificações de prontidão e isolamento não substituem os cenários de
//! negócio compartilhados na raiz do monorepo.

mod infrastructure_steps;
#[allow(dead_code)]
mod support;

/// Resolve as features técnicas e executa a suíte no runtime Tokio.
///
/// # Panics
///
/// Propaga as falhas de preparação, execução e encerramento descritas em
/// [`support::run`], fazendo o alvo Cargo terminar com falha.
#[tokio::main]
async fn main() {
    let features = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/features");
    support::run(features, None).await;
}
