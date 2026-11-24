//! Camada de domínio do módulo de apadrinhamento.
//!
//! [`entities`] exporta [`MentorshipProgram`](entities::MentorshipProgram),
//! [`MentorshipParticipant`](entities::MentorshipParticipant),
//! [`MatchingCriterion`](entities::MatchingCriterion),
//! [`MatchingSecret`](entities::MatchingSecret),
//! [`MatchingRun`](entities::MatchingRun),
//! [`MatchingSuggestion`](entities::MatchingSuggestion) e
//! [`MentorshipMatch`](entities::MentorshipMatch).
//! [`MentorshipProgramStatus`] e [`MatchingStrategy`] representam o ciclo de
//! vida e o método ativo do programa; [`MentorshipParticipantRole`] e
//! [`MentorshipParticipantStatus`] cobrem papel e situação da inscrição;
//! [`MatchingRunStatus`] e [`MatchingSuggestionStatus`] cobrem execução e
//! decisão sobre a sugestão; [`MentorshipMatchStatus`] cobre a vigência do
//! vínculo. [`rules`] e [`events`] permanecem reservados.
//!
//! Os tipos não executam SQL nem conhecem HTTP. Os derives `sqlx::FromRow` e
//! `sqlx::Type` constituem o acoplamento de mapeamento adotado pelo projeto;
//! as consultas e o gerenciamento de conexões continuam na infraestrutura.

pub mod entities;
pub mod events;
pub mod rules;
mod status;

pub use status::{
    MatchingRunStatus, MatchingStrategy, MatchingSuggestionStatus, MentorshipMatchStatus,
    MentorshipParticipantRole, MentorshipParticipantStatus, MentorshipProgramStatus,
};
