//! Organização dos domínios de negócio do CApp.
//!
//! Cada domínio possui camadas de API, aplicação, domínio e infraestrutura.
//! As declarações atuais estabelecem essa organização; ainda não implementam
//! casos de uso, regras, endpoints de negócio ou persistência.
//!
//! - [`identity`]: identidade institucional e dados acadêmicos.
//! - [`ca`]: Centros Acadêmicos e perfis administrativos.
//! - [`news`]: publicação e consulta de notícias.
//! - [`events`]: eventos, inscrições e vagas.
//! - [`demands`]: demandas institucionais e do espaço físico.
//! - [`materials`]: materiais acadêmicos, arquivos e links.
//! - [`elections`]: eleições e composição de gestões.
//! - [`faq`]: perguntas frequentes e base de conhecimento.
//! - [`mentorship`]: apadrinhamento e pareamento de participantes.
//! - [`governance`]: transparência e participação estudantil.

pub mod ca;
pub mod demands;
pub mod elections;
pub mod events;
pub mod faq;
pub mod governance;
pub mod identity;
pub mod materials;
pub mod mentorship;
pub mod news;
