//! API e organização dos domínios do backend CApp.
//!
//! A biblioteca expõe o estado e o router utilizados tanto pelo servidor HTTP
//! quanto pelos testes em memória. A inicialização do processo, a leitura de
//! `DATABASE_URL` e a abertura do pool ficam no executável `backend`.
//!
//! # Organização
//!
//! - [`api`]: composição das rotas e recursos compartilhados pelas requisições.
//! - [`modules`]: estrutura dos módulos de negócio e suas camadas.
//! - [`infrastructure`]: espaços reservados para adaptadores externos comuns.
//! - [`config`]: espaço reservado para centralizar configurações.
//! - [`shared`]: espaço reservado para tipos comuns aos domínios.
//!
//! Neste estágio, a API implementa somente `/health` e `/ready`. Os módulos de
//! negócio ainda não implementam os comportamentos descritos no README.
//! Consulte [`api::router::create_router`] para compor a aplicação e
//! [`api::state::AppState`] para fornecer o pool PostgreSQL.
//!
//! # Testes
//!
//! Os alvos `bdd` e `bdd_infrastructure` utilizam esta biblioteca com PostgreSQL
//! em containers descartáveis. Seus módulos de suporte pertencem aos testes,
//! não à API pública da biblioteca. Os comandos estão no README do backend.

pub mod api;
pub mod config;
pub mod infrastructure;
pub mod modules;
pub mod shared;
