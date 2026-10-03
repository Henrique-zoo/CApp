//! Recompilação das migrations embutidas no backend e nas suítes de testes.
//!
//! O Cargo deve observar o diretório inteiro para detectar novos arquivos SQL,
//! mesmo quando nenhum arquivo Rust for alterado.

/// Registra o diretório de migrations como entrada da compilação.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
