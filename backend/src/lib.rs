//! API e organização dos domínios do backend CApp.
//!
//! A biblioteca expõe o estado e o router utilizados tanto pelo servidor HTTP
//! quanto pelos testes em memória. A inicialização do processo, a leitura de
//! `DATABASE_URL`, a aplicação de migrations e a abertura do pool ficam no executável `backend`.
//!
//! # Organização
//!
//! - [`api`]: composição das rotas, recursos compartilhados e encerramento controlado.
//! - [`modules`]: estrutura dos módulos de negócio e suas camadas.
//! - [`infrastructure`]: migrations compartilhadas e estrutura para integrações externas.
//! - [`config`]: parâmetros operacionais, ciclo de vida e configurações externas da aplicação.
//! - [`shared`]: espaço reservado para tipos comuns aos domínios.
//!
//! Neste estágio, a API implementa `/health` e `/ready` e suporte a encerramento
//! gracioso com drenagem de conexões. Os módulos de negócio ainda não implementam
//! os comportamentos descritos no README.
//! Consulte [`api::router::create_router`] para compor a aplicação,
//! [`api::state::AppState`] para fornecer o pool PostgreSQL e
//! [`api::shutdown::serve_with_graceful_shutdown`] para executar o servidor com
//! ciclo de vida gerenciado.
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
