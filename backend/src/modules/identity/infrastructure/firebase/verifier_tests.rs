//! Testes do adaptador REAL, com JWT assinado por chave RSA descartável e
//! JWKS hospedado em servidor HTTP local Wiremock (sem Google externo).
//!
//! Chaves descartáveis de teste e endpoint local; nenhuma credencial de produção.
use super::*;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde_json::{Value, json};
use tokio::time::timeout;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TEST_PROJECT_ID: &str = "firebase-project-tests";
const TEST_PRIVATE_PEM: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/jwt/test-only-rsa-private.pem"
));
const TEST_JWKS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/jwt/jwks.json"
));

/// Cria claims temporais válidas para um projeto fictício.
fn jwt_claims() -> Value {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    json!({
        "sub":"firebase-uid-123",
        "aud": TEST_PROJECT_ID,
        "iss": format!("https://securetoken.google.com/{TEST_PROJECT_ID}"),
        "iat": now - 60,
        "auth_time": now - 120,
        "exp": now + 3600,
        "firebase": {"identities": {"microsoft.com": [
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
        ]}}
    })
}

/// Assina claims controladas com a chave RSA descartável dos testes.
fn signed_token(claims: Value, kid: &str) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_owned());
    encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(TEST_PRIVATE_PEM.as_bytes()).unwrap(),
    )
    .unwrap()
}

/// Direciona JWKS ao Wiremock, mantendo a validação criptográfica real.
fn new_test_verifier(server: &MockServer) -> FirebaseTokenVerifier {
    FirebaseTokenVerifier::new_with_endpoint(
        TEST_PROJECT_ID.to_owned(),
        Client::new(),
        format!("{}/keys", server.uri()),
    )
    .unwrap()
}

/// Publica a chave de teste e exige a quantidade indicada de downloads.
async fn mock_jwks(server: &MockServer, max_age: u64, expected_calls: u64) {
    Mock::given(method("GET"))
        .and(path("/keys"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Cache-Control", format!("public, max-age={max_age}"))
                .set_body_raw(TEST_JWKS, "application/json"),
        )
        .expect(expected_calls)
        .mount(server)
        .await;
}

/// Valida RS256 e reutiliza as chaves sem novos downloads.
#[tokio::test]
async fn valid_rs256_token_verifies_offline_after_first_jwks_fetch() {
    let server = MockServer::start().await;
    mock_jwks(&server, 300, 1).await;
    let verifier = new_test_verifier(&server);
    let token = signed_token(jwt_claims(), "test-kid");

    for _ in 0..3 {
        let result = verifier.verify(&token).await.unwrap();
        assert_eq!(result.firebase_uid, "firebase-uid-123");
        assert_eq!(
            result.microsoft_object_id.as_deref(),
            Some("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")
        );
    }
    // Wiremock confirma que somente UMA requisição HTTPS simulada ocorreu.
}

/// Rejeita adulteração da assinatura mesmo quando a chave está em cache.
#[tokio::test]
async fn altered_signature_is_rejected_after_key_is_cached() {
    let server = MockServer::start().await;
    mock_jwks(&server, 300, 1).await;
    let verifier = new_test_verifier(&server);
    verifier
        .verify(&signed_token(jwt_claims(), "test-kid"))
        .await
        .unwrap();

    let mut bad_claims = jwt_claims();
    bad_claims["sub"] = json!("attacker");
    // Altera a assinatura sem modificar o cabeçalho nem o payload.
    let valid = signed_token(bad_claims, "test-kid");
    let mut parts: Vec<String> = valid.split('.').map(str::to_owned).collect();
    let mut bytes = parts[2].clone().into_bytes();
    bytes[0] = if bytes[0] == b'A' { b'B' } else { b'A' };
    parts[2] = String::from_utf8(bytes).unwrap();
    let modified_token = parts.join(".");
    assert!(matches!(
        verifier.verify(&modified_token).await,
        Err(AuthenticationError::InvalidToken)
    ));
}

/// Recusa tokens assinados corretamente mas emitidos para outro projeto ou emissor.
#[tokio::test]
async fn invalid_issuer_and_audience_are_rejected() {
    let server = MockServer::start().await;
    mock_jwks(&server, 300, 1).await;
    let verifier = new_test_verifier(&server);

    let mut wrong_issuer = jwt_claims();
    wrong_issuer["iss"] = json!("https://evil.invalid");
    assert!(matches!(
        verifier
            .verify(&signed_token(wrong_issuer, "test-kid"))
            .await,
        Err(AuthenticationError::InvalidToken)
    ));

    let mut wrong_audience = jwt_claims();
    wrong_audience["aud"] = json!("other-firebase-project");
    assert!(matches!(
        verifier
            .verify(&signed_token(wrong_audience, "test-kid"))
            .await,
        Err(AuthenticationError::InvalidToken)
    ));
}

/// Distingue token expirado de outras credenciais inválidas.
#[tokio::test]
async fn expired_token_is_rejected() {
    let server = MockServer::start().await;
    mock_jwks(&server, 300, 1).await;
    let verifier = new_test_verifier(&server);
    let mut expired = jwt_claims();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    expired["exp"] = json!(now - 1);
    assert!(matches!(
        verifier.verify(&signed_token(expired, "test-kid")).await,
        Err(AuthenticationError::ExpiredToken)
    ));
}

/// Recusa autenticação declarada no futuro.
#[tokio::test]
async fn auth_time_in_future_is_rejected() {
    let server = MockServer::start().await;
    mock_jwks(&server, 300, 1).await;
    let verifier = new_test_verifier(&server);
    let mut claims = jwt_claims();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    claims["auth_time"] = json!(now + 500);
    assert!(matches!(
        verifier.verify(&signed_token(claims, "test-kid")).await,
        Err(AuthenticationError::InvalidToken)
    ));
}

/// Contém downloads provocados por kid desconhecido durante o intervalo mínimo.
#[tokio::test]
async fn unknown_kid_during_fresh_cache_does_not_trigger_another_download() {
    let server = MockServer::start().await;
    mock_jwks(&server, 300, 1).await;
    let verifier = new_test_verifier(&server);
    verifier.key_for("test-kid").await.unwrap();
    assert!(matches!(
        verifier.key_for("nonexistent-kid").await,
        Err(AuthenticationError::InvalidToken)
    ));
}

/// TTL zero atende apenas a busca atual e mantém contenção nas seguintes.
#[tokio::test]
async fn zero_ttl_applies_backoff_and_does_not_requery_per_request() {
    let server = MockServer::start().await;
    mock_jwks(&server, 0, 1).await;
    let verifier = new_test_verifier(&server);
    verifier.key_for("test-kid").await.unwrap();
    assert!(matches!(
        verifier.key_for("test-kid").await,
        Err(AuthenticationError::ProviderUnavailable)
    ));
}

/// Falha HTTP do JWKS é indisponibilidade e ativa backoff.
#[tokio::test]
async fn server_failure_is_provider_unavailable() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/keys"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&server)
        .await;
    let verifier = new_test_verifier(&server);
    assert!(matches!(
        verifier.key_for("test-kid").await,
        Err(AuthenticationError::ProviderUnavailable)
    ));
    // A consulta imediata seguinte não deve causar outra requisição.
    assert!(matches!(
        verifier.key_for("test-kid").await,
        Err(AuthenticationError::ProviderUnavailable)
    ));
}

/// Requisições concorrentes compartilham uma busca inicial; contenção pode recusar leitores.
#[tokio::test]
async fn concurrent_requests_share_one_initial_jwks_fetch() {
    let server = MockServer::start().await;
    mock_jwks(&server, 300, 1).await;
    let verifier = new_test_verifier(&server);
    let token = signed_token(jwt_claims(), "test-kid");
    let mut handles = Vec::new();
    for _ in 0..12 {
        let verifier = verifier.clone();
        let token = token.clone();
        handles.push(tokio::spawn(async move { verifier.verify(&token).await }));
    }
    let mut successes = 0usize;
    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => successes += 1,
            // A implementação atual prefere falhar rapidamente enquanto
            // um JWKS inicial está em carregamento, sem aguardar o refresh.
            Err(AuthenticationError::ProviderUnavailable) => {}
            other => panic!(
                "resultado inesperado: {}",
                if other.is_err() { "erro" } else { "sucesso" }
            ),
        }
    }
    assert!(successes >= 1);
    assert!(verifier.verify(&token).await.is_ok());
}

/// Leitores de uma chave válida não aguardam o mutex de atualização.
#[tokio::test]
async fn cached_read_does_not_wait_for_refresh_mutex() {
    let server = MockServer::start().await;
    mock_jwks(&server, 300, 1).await;
    let verifier = new_test_verifier(&server);
    verifier.key_for("test-kid").await.unwrap();

    // Simula que outra tarefa está atualizando o JWKS: o mutex de atualização
    // está adquirido, mas o RwLock NÃO deve impedir leitores de chaves válidas.
    let _guard = verifier.refresh_lock.lock().await;
    let found = timeout(Duration::from_millis(500), verifier.key_for("test-kid")).await;
    assert!(
        found
            .expect("cache válido não deveria aguardar o refresh")
            .is_ok()
    );
}

/// A configuração rejeita projeto vazio ou com espaços antes de acessar a rede.
#[test]
fn invalid_project_configuration_is_rejected() {
    for project in ["", " ", "project id", "project\n"] {
        assert!(FirebaseTokenVerifier::new(project.into(), Client::new()).is_err());
    }
    assert!(FirebaseTokenVerifier::new(TEST_PROJECT_ID.into(), Client::new()).is_ok());
}

/// Cabeçalhos inválidos, algoritmo simétrico e ausência de kid dispensam JWKS.
#[tokio::test]
async fn malformed_headers_are_rejected_without_network() {
    let server = MockServer::start().await;
    let verifier = new_test_verifier(&server);
    let key = EncodingKey::from_rsa_pem(TEST_PRIVATE_PEM.as_bytes()).unwrap();
    let mut empty_kid = Header::new(Algorithm::RS256);
    empty_kid.kid = Some(String::new());
    let tokens = [
        "not-a-jwt".to_owned(),
        "invalid.base64.signature".to_owned(),
        encode(
            &Header::new(Algorithm::HS256),
            &jwt_claims(),
            &EncodingKey::from_secret(b"test-secret"),
        )
        .unwrap(),
        encode(&Header::new(Algorithm::RS256), &jwt_claims(), &key).unwrap(),
        encode(&empty_kid, &jwt_claims(), &key).unwrap(),
    ];
    for token in tokens {
        assert!(matches!(
            verifier.verify(&token).await,
            Err(AuthenticationError::InvalidToken)
        ));
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}

/// Claims obrigatórias ausentes ou de tipo incompatível nunca autenticam.
#[tokio::test]
async fn missing_or_malformed_claims_are_rejected() {
    let server = MockServer::start().await;
    mock_jwks(&server, 300, 1).await;
    let verifier = new_test_verifier(&server);
    for field in ["sub", "aud", "iss", "iat", "exp", "auth_time", "firebase"] {
        let mut claims = jwt_claims();
        claims.as_object_mut().unwrap().remove(field);
        assert!(
            matches!(
                verifier.verify(&signed_token(claims, "test-kid")).await,
                Err(AuthenticationError::InvalidToken)
            ),
            "{field}"
        );
    }
    for (field, value) in [
        ("sub", json!(42)),
        ("exp", json!("tomorrow")),
        ("firebase", json!({})),
        ("iat", json!(-1)),
    ] {
        let mut claims = jwt_claims();
        claims[field] = value;
        assert!(
            matches!(
                verifier.verify(&signed_token(claims, "test-kid")).await,
                Err(AuthenticationError::InvalidToken)
            ),
            "{field}"
        );
    }
}

/// UID vazio/maior que 128 caracteres é rejeitado; o limite conta caracteres Unicode.
#[tokio::test]
async fn uid_length_boundaries_are_enforced() {
    let server = MockServer::start().await;
    mock_jwks(&server, 300, 1).await;
    let verifier = new_test_verifier(&server);
    for uid in [String::new(), "a".repeat(129)] {
        let mut claims = jwt_claims();
        claims["sub"] = json!(uid);
        assert!(matches!(
            verifier.verify(&signed_token(claims, "test-kid")).await,
            Err(AuthenticationError::InvalidToken)
        ));
    }
    let uid = "é".repeat(128);
    let mut claims = jwt_claims();
    claims["sub"] = json!(uid);
    assert_eq!(
        verifier
            .verify(&signed_token(claims, "test-kid"))
            .await
            .unwrap()
            .firebase_uid,
        uid
    );
}

/// Datas futuras, autenticação posterior à emissão e emissão após expiração são recusadas.
#[tokio::test]
async fn inconsistent_temporal_claims_are_rejected() {
    let server = MockServer::start().await;
    mock_jwks(&server, 300, 1).await;
    let verifier = new_test_verifier(&server);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    for (field, value) in [
        ("iat", now + 600),
        ("auth_time", now - 30),
        ("nbf", now + 600),
        ("iat", now + 3600),
    ] {
        let mut claims = jwt_claims();
        claims[field] = json!(value);
        assert!(
            matches!(
                verifier.verify(&signed_token(claims, "test-kid")).await,
                Err(AuthenticationError::InvalidToken)
            ),
            "{field}"
        );
    }
    let mut claims = jwt_claims();
    claims["exp"] = json!(now);
    assert!(matches!(
        verifier.verify(&signed_token(claims, "test-kid")).await,
        Err(AuthenticationError::ExpiredToken)
    ));
}

/// Ausência de provedor Microsoft é permitida; listas vazias, ambíguas ou ID vazio não são.
#[tokio::test]
async fn microsoft_identity_cardinality_is_validated() {
    let server = MockServer::start().await;
    mock_jwks(&server, 300, 1).await;
    let verifier = new_test_verifier(&server);
    let mut claims = jwt_claims();
    claims["firebase"]["identities"] = json!({});
    assert!(
        verifier
            .verify(&signed_token(claims, "test-kid"))
            .await
            .unwrap()
            .microsoft_object_id
            .is_none()
    );
    for ids in [
        json!([]),
        json!([""]),
        json!(["a", "b"]),
        json!("not-an-array"),
    ] {
        let mut claims = jwt_claims();
        claims["firebase"]["identities"]["microsoft.com"] = ids;
        assert!(matches!(
            verifier.verify(&signed_token(claims, "test-kid")).await,
            Err(AuthenticationError::InvalidToken)
        ));
    }
}

/// Cache-Control é interpretado sem prolongar validade além da idade da resposta.
#[test]
fn cache_control_directives_and_age_are_respected() {
    for (control, age, expected) in [
        ("public, max-age=300", "20", 280),
        ("MAX-AGE=\"60\"", "0", 60),
        ("max-age=10", "100", 0),
        ("max-age=60, no-store", "0", 0),
        ("no-cache, max-age=60", "0", 0),
        ("max-age=invalid", "0", 0),
        ("max-age=-1", "0", 0),
        ("max-age=30", "invalid", 30),
        ("max-age=10, max-age=20", "0", 20),
        ("public", "0", 0),
    ] {
        let mut headers = HeaderMap::new();
        headers.insert(CACHE_CONTROL, control.parse().unwrap());
        headers.insert(AGE, age.parse().unwrap());
        assert_eq!(
            cache_ttl(&headers),
            Duration::from_secs(expected),
            "{control}, Age={age}"
        );
    }
    assert_eq!(cache_ttl(&HeaderMap::new()), Duration::ZERO);
}

/// JSON JWKS inválido ou sem chave RSA de assinatura utilizável é erro do provedor.
#[tokio::test]
async fn invalid_jwks_never_populates_cache() {
    let original: Value = serde_json::from_str(TEST_JWKS).unwrap();
    let mut bodies = vec!["not-json".to_owned(), "{}".into(), "{\"keys\":[]}".into()];
    for (field, value) in [
        ("alg", json!("RS512")),
        ("use", json!("enc")),
        ("kid", json!("")),
        ("n", json!("!")),
    ] {
        let mut body = original.clone();
        body["keys"][0][field] = value;
        bodies.push(body.to_string());
    }
    let mut no_kid = original.clone();
    no_kid["keys"][0].as_object_mut().unwrap().remove("kid");
    bodies.push(no_kid.to_string());
    bodies.push(json!({"keys":[{"kty":"oct", "k":"YWJj", "kid":"test-kid"}]}).to_string());
    for body in bodies {
        let server = MockServer::start().await;
        Mock::given(path("/keys"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .expect(1)
            .mount(&server)
            .await;
        let verifier = new_test_verifier(&server);
        assert!(matches!(
            verifier.key_for("test-kid").await,
            Err(AuthenticationError::ProviderUnavailable)
        ));
        assert!(verifier.cache.read().await.keys.is_empty());
    }
}

/// Uma chave nova pode ser obtida após o intervalo, e a chave antiga deixa o cache.
#[tokio::test]
async fn expired_cache_refreshes_rotated_keys() {
    let server = MockServer::start().await;
    mock_jwks(&server, 300, 1).await;
    let verifier = new_test_verifier(&server);
    verifier.key_for("test-kid").await.unwrap();
    server.verify().await;
    server.reset().await;
    let mut jwks: Value = serde_json::from_str(TEST_JWKS).unwrap();
    jwks["keys"][0]["kid"] = json!("rotated");
    Mock::given(path("/keys"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("cache-control", "max-age=300")
                .set_body_json(jwks),
        )
        .expect(1)
        .mount(&server)
        .await;
    {
        let mut cache = verifier.cache.write().await;
        cache.valid_until = Instant::now();
        cache.refresh_after = Instant::now();
    }
    assert!(
        verifier
            .verify(&signed_token(jwt_claims(), "rotated"))
            .await
            .is_ok()
    );
    assert!(matches!(
        verifier.key_for("test-kid").await,
        Err(AuthenticationError::InvalidToken)
    ));
}

/// Cancelar uma busca em andamento mantém o backoff e não deixa um lock adquirido.
#[tokio::test]
async fn cancelled_fetch_preserves_backoff_without_deadlock() {
    let server = MockServer::start().await;
    let started = Arc::new(tokio::sync::Notify::new());
    let notify = started.clone();
    Mock::given(path("/keys"))
        .respond_with(move |_: &wiremock::Request| {
            notify.notify_one();
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(30))
                .set_body_raw(TEST_JWKS, "application/json")
        })
        .expect(1)
        .mount(&server)
        .await;
    let verifier = new_test_verifier(&server);
    let cloned = verifier.clone();
    let task = tokio::spawn(async move { cloned.key_for("test-kid").await });
    timeout(Duration::from_secs(2), started.notified())
        .await
        .unwrap();
    task.abort();
    assert!(matches!(task.await, Err(error) if error.is_cancelled()));
    let result = timeout(Duration::from_secs(1), verifier.key_for("test-kid"))
        .await
        .unwrap();
    assert!(matches!(
        result,
        Err(AuthenticationError::ProviderUnavailable)
    ));
}

/// Um refresh malsucedido não impede leitura de uma chave antiga ainda válida.
#[tokio::test]
async fn failed_refresh_preserves_valid_known_keys() {
    let server = MockServer::start().await;
    mock_jwks(&server, 300, 1).await;
    let verifier = new_test_verifier(&server);
    verifier.key_for("test-kid").await.unwrap();
    server.verify().await;
    server.reset().await;
    Mock::given(path("/keys"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&server)
        .await;
    verifier.cache.write().await.refresh_after = Instant::now();
    assert!(matches!(
        verifier.key_for("rotated").await,
        Err(AuthenticationError::ProviderUnavailable)
    ));
    assert!(verifier.key_for("test-kid").await.is_ok());
}
