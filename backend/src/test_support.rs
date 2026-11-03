//! Acesso restrito pela feature `test-support` para testes externos à biblioteca.
//!
//! Constrói clientes com origens locais e um router de teste que exercita o extractor
//! real. Não altera o router de produção nem os construtores públicos dos clientes.

use axum::{Json, Router, routing::get};
use serde_json::{Value, json};

use crate::{
    api::state::{AppState, Clients},
    modules::identity::{
        api::extractors::AuthenticatedInstitutionalIdentity,
        infrastructure::{firebase::FirebaseTokenVerifier, microsoft::MicrosoftGraphClient},
    },
};

/// Constrói clientes reais apontando para os servidores HTTP locais da suíte.
///
/// As URLs são fornecidas pelo teste, nunca pela requisição do usuário. A validação
/// criptográfica e o cache Firebase permanecem ativos. O cliente HTTP deve recusar
/// redirecionamentos para não encaminhar credenciais a outras origens.
///
/// # Erros
///
/// Propaga a rejeição de um identificador de projeto Firebase inválido.
pub fn local_identity_clients(
    project_id: String,
    http: reqwest::Client,
    jwks_url: String,
    graph_url: String,
) -> anyhow::Result<Clients> {
    Ok(Clients::new(
        FirebaseTokenVerifier::new_with_endpoint(project_id, http.clone(), jwks_url)?,
        MicrosoftGraphClient::new_with_base_url(http, graph_url),
    ))
}

/// Compõe as rotas reais de identidade com uma rota protegida exclusiva dos testes.
///
/// `GET /protected` devolve UID e instituição obtidos pelo extractor real. O router
/// ainda precisa de `AppState` e não abre conexões durante sua construção. Essa rota
/// não é adicionada por `api::router::create_router` nem pelo servidor de produção.
pub fn identity_router() -> Router<AppState> {
    crate::modules::identity::api::router().route("/protected", get(protected_identity))
}

/// Expõe o resultado do extractor para que o teste confira a identidade autenticada.
async fn protected_identity(identity: AuthenticatedInstitutionalIdentity) -> Json<Value> {
    Json(json!({
        "uid": identity.0.firebase_uid,
        "institution": identity.0.institution_id,
    }))
}
