//! Registro dos steps do backend para as features de produto do monorepo.
//!
//! Submódulos implementam a tradução dos passos em linguagem de domínio
//! para ações e verificações sobre [`AppWorld`](crate::support::AppWorld).
//!
//! Steps técnicos de prontidão e isolamento pertencem ao alvo
//! `bdd_infrastructure`, sem serem registrados nesta suíte.

mod associacao_dados_academicos;
mod autenticacao_institucional;
mod cargos_padrao_ca;
mod consulta_inscricao_eventos;
mod consultas_estudantis;
mod contexto_multi_ca;
mod exclusao_de_cargos;
mod integrantes_da_gestao;
mod provisionamento_usuario;
mod publicacao_institucional;
mod publicacao_noticias;
mod shared_steps;
mod sincronizacao_academica;
mod visualizacao_materiais;
mod visualizacao_noticias;

/// Sinaliza que uma step registrada ainda aguarda a implementação real.
///
/// # Panics
///
/// Sempre falha com o texto da step. O Cucumber registra a falha e a suíte
/// permanece reprovada até que o comportamento seja automatizado.
#[track_caller]
pub(super) fn pending_step(step: &str) -> ! {
    unimplemented!("Step pendente: {step}");
}
