//! Entidades do módulo de apadrinhamento.
//!
//! [`MentorshipProgram`] modela o programa com estado, estratégia, limite de
//! padrinhos e alcance entre cursos. [`MentorshipParticipant`] modela a inscrição
//! com papel, situação e preferência de outros cursos. [`MatchingCriterion`]
//! guarda critérios ponderados e [`MatchingSecret`] guarda valores já cifrados
//! do participante. [`MatchingRun`] registra a execução de matching e
//! [`MatchingSuggestion`] o par sugerido. [`MentorshipMatch`] confirma o vínculo
//! por atribuição manual ou aceitação de sugestão. Os derives SQLx mapeiam linhas
//! para esses tipos; não aplicam migrations nem consultam o banco.

pub mod criterion;
pub mod pairing;
pub mod participant;
pub mod program;
pub mod run;
pub mod secret;
pub mod suggestion;

pub use criterion::MatchingCriterion;
pub use pairing::MentorshipMatch;
pub use participant::MentorshipParticipant;
pub use program::MentorshipProgram;
pub use run::MatchingRun;
pub use secret::MatchingSecret;
pub use suggestion::MatchingSuggestion;
