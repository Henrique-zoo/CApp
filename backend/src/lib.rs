//! API e casos de uso do backend CApp.
//!
//! A biblioteca expõe o router e os adaptadores usados pelo servidor e pelos testes.
//! O executável lê `DATABASE_URL`, `FIREBASE_PROJECT_ID` e `RUST_LOG`, configura os
//! clientes externos, conecta ao PostgreSQL e aplica as migrations antes de servir.
//!
//! # Organização
//!
//! - [`api`]: estado, contrato de erros e composição HTTP.
//! - [`modules::identity`]: verificação institucional e evidência de elegibilidade.
//! - [`infrastructure::database`]: adaptador PostgreSQL e migrations compartilhadas.
//! - [`config`] e [`shared`]: espaços reservados para configuração e tipos comuns.
//!
//! Estão registradas as rotas `/health`, `/ready` e
//! `POST /auth/institutional/verify`. A infraestrutura de extração de identidade
//! para handlers protegidos está implementada no módulo de identidade, mas nenhuma
//! rota atual a utiliza. Os demais módulos mantêm sua estrutura prevista.
//! Provisionamento, login no mobile e encerramento de sessão não são implementados
//! por esta biblioteca.
//!
//! # Testes
//!
//! Os alvos `bdd` e `bdd_infrastructure` usam PostgreSQL em containers descartáveis
//! e exigem `FIREBASE_PROJECT_ID` no ambiente para construir os clientes reais.
//! Os exemplos locais desta documentação usam um projeto fictício somente quando
//! não realizam autenticação externa. Consulte o README para executar as suítes.

pub mod api;
pub mod config;
pub mod infrastructure;
pub mod modules;
pub mod shared;
