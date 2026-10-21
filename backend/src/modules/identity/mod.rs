//! Identidade institucional e confirmação temporária de elegibilidade.
//!
//! [`application`] coordena Firebase, Microsoft Graph e persistência por traits;
//! [`infrastructure`] implementa esses contratos; [`api`] adapta HTTP e erros;
//! [`domain`] representa instituições e evidências persistidas.
//!
//! A verificação confronta as identidades Firebase e Microsoft, identifica a
//! instituição pelo tenant e consulta o grupo de estudantes. A evidência permite
//! reutilizar a confirmação durante seu prazo de validade, sem consultar o Graph
//! em cada requisição protegida.
//!
//! Não provisiona usuários internos, associa cursos ou administra sessões Firebase.
//! O curso obtido de `jobTitle` é apenas um dado externo opcional. A identidade
//! institucional também não representa pertencimento a um Centro Acadêmico.

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
