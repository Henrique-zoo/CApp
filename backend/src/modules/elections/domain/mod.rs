//! Camada de domínio do módulo de eleições.
//!
//! Contém os modelos, objetos de valor, regras e eventos do negócio.
//! Esta camada não deve depender de HTTP, banco de dados ou mecanismos
//! de transporte.
//!
//! # Invariantes e validação da #130
//!
//! | Regra | Modelo e cobertura unitária |
//! | --- | --- |
//! | Modalidades sem dependência criptográfica indevida | `ElectionConfiguration`, `tests/lifecycle.rs` |
//! | Chapa única, Nulo, IDs e limites | `ElectionOptions`, `Manifest` e seus testes; `tests/contracts.rs` confere o perfil compartilhado |
//! | Calendário, configuração e estados protegidos | Operações de `Election` reproduzidas na reconstituição; `tests/preparation.rs` e `tests/reconstitution.rs` |
//! | Primeiro vínculo e unicidade por conta/eleição | `Participation`, `ParticipationKey` e `tests/participation.rs`; aplicação mantém o principal em `AuthenticatedElectorContext` |
//! | Registro confidencial, posição e origem do quadro | `BoardEntry`, `tests/board_entries.rs`; a aplicação vincula registro/recibo à mesma participação |
//! | Fotografia de cancelamento/encerramento compatível com o manifesto | `rules/board_snapshot.rs`, `tests/board_snapshots.rs`; inclusive construção direta e reconstituição |
//! | Falta de autoridade, inconsistência e cancelamento distintos | `VerifiedTally`, `ElectionResult`, testes de ciclo de vida e corpus |
//! | Limite deliberado de validade individual | Caso `compensated` do corpus: contagem consistente não prova bits válidos |
//!
//! Os snapshots são entradas internas para validação, sem `Deserialize` nas
//! entidades. Doc-tests também verificam a impossibilidade de escrever estado
//! privado, desserializar `Election` diretamente ou forjar `VerifiedManifest`
//! fora do crate. Os testes não precisam de banco, cloud ou criptografia pronta.
//!
//! As transações concorrentes, elegibilidade real, verificação criptográfica,
//! transporte HTTPS e automação BDD de produto pertencem às issues consumidoras.
//! As portas mantêm essas responsabilidades explícitas; exemplos sintéticos de
//! domínio não comprovam integrações nem substituem a auditoria do produto.

pub mod entities;
pub mod errors;
pub mod events;
pub mod rules;
pub mod value_objects;

#[cfg(test)]
mod tests;
