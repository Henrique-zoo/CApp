//! Runner dos comportamentos funcionais compartilhados com os clientes do CApp.
//!
//! Lê `features/` na raiz do monorepo e registra somente os steps de
//! `step_definitions`. A preparação do PostgreSQL, o World e os hooks são
//! reutilizados de `support`. O caminho é resolvido a partir do manifesto,
//! independentemente do diretório de trabalho do processo.
//!
//! # Execução
//!
//! Requer um daemon Docker acessível. Execute no diretório `backend`:
//!
//! ```sh
//! cargo test --test bdd --locked
//! ```
//!
//! Especificações sem cenários podem resultar em zero cenários executados;
//! esse resultado não demonstra cobertura dos comportamentos do produto.

mod step_definitions;
#[allow(dead_code)]
mod support;

/// Resolve as features do produto e executa a suíte no runtime Tokio.
///
/// # Panics
///
/// Propaga as falhas de preparação, execução e encerramento descritas em
/// [`support::run`], fazendo o alvo Cargo terminar com falha.
#[tokio::main]
async fn main() {
    let features = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../features");
    support::run(features).await;
}
