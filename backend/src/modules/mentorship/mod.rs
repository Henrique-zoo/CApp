//! Módulo de apadrinhamento e formação de vínculos.
//!
//! O domínio modela o programa ([`MentorshipProgram`](domain::entities::MentorshipProgram)),
//! a inscrição ([`MentorshipParticipant`](domain::entities::MentorshipParticipant)),
//! o critério ([`MatchingCriterion`](domain::entities::MatchingCriterion)),
//! o segredo cifrado ([`MatchingSecret`](domain::entities::MatchingSecret)),
//! a execução ([`MatchingRun`](domain::entities::MatchingRun)),
//! a sugestão ([`MatchingSuggestion`](domain::entities::MatchingSuggestion))
//! e o vínculo ([`MentorshipMatch`](domain::entities::MentorshipMatch)).
//!
//! As camadas [`api`], [`application`] e [`infrastructure`] continuam
//! reservadas; este módulo ainda não expõe HTTP, casos de uso ou persistência.

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
