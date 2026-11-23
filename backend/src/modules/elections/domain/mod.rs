//! Regras e primitivas do domínio de eleições.
//!
//! Escolhas, contagens, identificadores e digests são compartilhados pelas
//! entidades. As evidências do voto confidencial ficam junto do voto em
//! [`entities::vote`]. O domínio mantém as regras independentes de HTTP e
//! persistência. A codificação textual pertence à infraestrutura.
//!
//! [`error`] reúne as falhas das entidades, das regras e das operações
//! matemáticas implementadas em [`cryptography`].
//!
//! # Invariantes e validação da #130
//!
//! | Regra | Modelo e cobertura unitária |
//! | --- | --- |
//! | Configuração conforme a modalidade escolhida | `ElectionConfiguration` e `entities/election/tests.rs` |
//! | Chapa única, Nulo, IDs e limites | `ElectionOptions`, `Manifest` e seus testes; `tests/contracts.rs` confere o perfil compartilhado |
//! | Calendário, configuração e estados protegidos | `Election::advance` e `Election::change_schedule`; `entities/election/tests.rs` |
//! | Aceite e unicidade por conta/eleição | `Vote`, `VoteKey` e `entities/vote/tests.rs`; a aplicação resolve a conta a partir da autenticação |
//! | Registro confidencial, posição e origem do quadro | `BoardEntry` projeta a cédula assinada do voto para publicação |
//! | Referências do quadro em registros assinados | `rules/board_snapshot.rs` e os testes do corpus |
//! | Falta de autoridade, apuração final e cancelamento | `ElectionState`, `FinalTallyResult` e testes do ciclo eleitoral e do corpus |
//! | Limite deliberado de validade individual | Caso `compensated` do corpus: contagem consistente não prova bits válidos |
//!
//! Os doc-tests verificam que o estado de `Election` só é modificado pelos seus
//! métodos públicos e que a verificação de `VerifiedManifest` permanece sob
//! controle do crate. A leitura da eleição persistida pertence à implementação
//! futura do repositório.
//!
//! As transações concorrentes, elegibilidade real, verificação criptográfica,
//! transporte HTTPS e automação BDD de produto pertencem às issues consumidoras.
//! As portas mantêm essas responsabilidades explícitas; exemplos sintéticos de
//! domínio não comprovam integrações nem substituem a auditoria do produto.

pub mod choices;
pub mod counts;
pub mod cryptography;
pub mod entities;
pub mod error;
pub mod events;
pub mod identifiers;
pub mod profile;
pub mod rules;

/// Digest SHA-256 usado nas referências aos registros assinados.
pub type Fingerprint = [u8; 32];

#[cfg(test)]
mod tests;
