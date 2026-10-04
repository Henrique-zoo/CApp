//! Registro dos steps do backend para as features de produto do monorepo.
//!
//! Submódulos implementam a tradução dos passos em linguagem de domínio
//! para ações e verificações sobre [`AppWorld`](crate::support::AppWorld).
//!
//! Steps técnicos de prontidão e isolamento pertencem ao alvo
//! `bdd_infrastructure`, sem serem registrados nesta suíte.

mod prestacao_de_contas;
