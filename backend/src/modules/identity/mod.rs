//! Identidade institucional e evidência temporária de vínculo.
//!
//! [`application`] coordena Firebase, Microsoft Graph e persistência por traits;
//! [`infrastructure`] implementa esses contratos; [`api`] adapta HTTP e erros;
//! [`domain`] representa instituições e evidências persistidas.
//!
//! # Fluxo e limites
//!
//! [`verify_institutional_identity`](application::services::authentication::verify_institutional_identity)
//! valida Firebase, consulta o tenant Microsoft, localiza a instituição e verifica
//! o grupo estudantil configurado. Obtém então o perfil com seu vínculo e confronta
//! o objeto Microsoft com a identidade vinculada ao Firebase antes de persistir.
//! Estudantes e não estudantes podem concluir a verificação. O grupo determina
//! [`InstitutionalAffiliation`](application::ports::InstitutionalAffiliation);
//! `jobTitle` só é interpretado como curso para estudantes. Nome, e-mail e curso
//! ausentes não impedem a operação; `userPrincipalName` não integra o perfil.
//!
//! A instituição é resolvida pelo tenant, sem restrição fixa à UnB. Seu cadastro
//! deve existir antes do login; `institution.status` não é uma regra de acesso
//! aplicada pelo serviço atual. A configuração de tenant e grupo vem do banco.
//!
//! A evidência tem validade de 24 horas a partir do início das consultas externas.
//! [`authenticate_institutional_request`](application::services::authentication::authenticate_institutional_request)
//! permite reutilizá-la, inclusive para não estudantes, sem consultar o Graph.
//! O extractor em [`api::extractors`] só protege handlers que o recebem; nenhuma
//! rota de produção atual o usa. A rota de verificação aceita o primeiro acesso
//! e a renovação, sem exigir evidência anterior.
//!
//! Não provisiona usuários internos, associa cursos ou administra sessões Firebase.
//! O curso é um dado externo opcional, sem associação ao catálogo interno.
//! A identidade institucional não representa pertencimento a um Centro Acadêmico
//! nem concede permissões de curso ou administrativas.
//!
//! # Onde consultar os detalhes
//!
//! | Assunto | Documentação responsável |
//! | --- | --- |
//! | Ordem das operações, TTL, concorrência e erros dos casos de uso | [`application::services::authentication`] |
//! | Contratos dos provedores e repositórios | [`application::ports`] |
//! | JWT, cache de chaves e limites da validação Firebase | [`infrastructure::firebase`] |
//! | Consultas de tenant, grupo e perfil Microsoft | [`infrastructure::microsoft`] |
//! | Filtros de validade e atualização da evidência no PostgreSQL | [`infrastructure::repository`] |
//! | Composição HTTP, DTOs, timeouts e conversão de erros | [`api`] |
//!
//! O contrato HTTP de referência, com campos, códigos e exemplos, está em
//! `packages/api-contracts/openapi.yaml`, na raiz do monorepo. A configuração do
//! ambiente e os comandos gerais estão em `backend/README.md`.
//!
//! # Estratégia de testes
//!
//! Os testes usam dados fictícios, sem credenciais Firebase ou Microsoft reais.
//! Os caminhos abaixo são relativos a `backend/`:
//!
//! | Local | Escopo |
//! | --- | --- |
//! | `src/modules/identity/application/services/authentication/tests/` | Decisões dos serviços, chamadas às portas, TTL e vínculos, com mocks. |
//! | `src/modules/identity/infrastructure/firebase/verifier_tests.rs` | JWT, claims, JWKS, cache, rotação, concorrência e cancelamento, com HTTP local. |
//! | `src/modules/identity/infrastructure/microsoft/client_tests.rs` | Requisições Graph, falhas do provedor e interpretação de curso por vínculo. |
//! | `src/modules/identity/api/tests.rs` | JSON, Bearer, erros públicos e rejeições de entrada sem PostgreSQL. |
//! | `tests/integration/repositories/identity.rs` | Migrations, constraints, isolamento, validade e UPSERT concorrente com banco real. |
//! | `tests/integration/http/identity.rs` | Router, JWT assinado, Graph local, banco real e extractor protegido juntos. |
//!
//! Cada teste de integração reutiliza um PostgreSQL descartável entre seus casos,
//! com migrations reais e dados isolados dos outros testes. Fixtures de domínio
//! ficam junto aos testes; as chaves RSA/JWKS em `tests/fixtures/jwt/` são públicas,
//! exclusivas dos testes e compartilhadas pelas suítes que validam JWT.
//! O módulo HTTP exige a feature `test-support` para compor clientes locais e uma
//! rota protegida de teste. Essa rota não faz parte do router de produção.
//!
//! No diretório `backend`, filtre os testes de identidade:
//!
//! ```bash
//! cargo test --lib --locked modules::identity::
//! cargo test --test integration --locked repositories::identity::
//! cargo test --test integration --features test-support --locked http::identity::
//! ```
//!
//! Os dois últimos comandos exigem Docker. Nenhuma dessas suítes carrega `.env`
//! ou usa `DATABASE_URL`. Os testes unitários de provedores abrem servidores HTTP
//! locais, mas não dependem de serviços hospedados.
//!
//! Para medir a cobertura das suítes Rust, com `cargo-llvm-cov` e
//! `llvm-tools-preview` instalados:
//!
//! ```bash
//! cargo llvm-cov --lib --test integration --features test-support --locked --summary-only --ignore-filename-regex '(/tests/|_tests\.rs|/tests\.rs|/test_support\.rs)'
//! ```
//!
//! Esse comando mede a biblioteca e as integrações, excluindo o código dos testes;
//! consulte os caminhos `src/modules/identity/` para avaliar o módulo. Cobertura
//! local não comprova integração com os serviços hospedados da Microsoft/Google.
//! A automação dos cenários Gherkin é uma suíte separada; cenários marcados como
//! pendentes não validam autenticação por terem sido compilados.

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
