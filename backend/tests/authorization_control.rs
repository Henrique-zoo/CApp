//! Testes de integração e verificação de regras de autorização institucional (Issue #52).
//!
//! Valida o isolamento estrito por Centro Acadêmico, a implicação de homologação sobre
//! publicação direta, a recusa de homologação para publicadores diretos e a revogação
//! imediata de privilégios com preservação de histórico.

use std::{collections::HashSet, sync::Arc};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    middleware::from_fn,
    routing::{get, post},
};
use backend::{
    api::{
        auth::{
            ActiveCaContext, ApproveEventsPermission, ApproveNewsPermission,
            ManageMembersPermission, PublishEventsDirectPermission, PublishNewsDirectPermission,
            RequirePermission, require_permission_middleware,
        },
        state::AppState,
    },
    domain::{Permission, Role, Scope},
    modules::ca::application::{
        ports::{
            AuthorizationServiceError, AuthorizationServicePort, RoleAssignmentRepositoryPort,
            RoleRepositoryPort,
        },
        services::authorization_service::{
            AuthorizationService, CACC_DIDACTIC_ID, CAER_DIDACTIC_ID, DIRETOR_COMUNICACAO_USER_ID,
            DIRETOR_EVENTOS_USER_ID, PRESIDENTE_CAER_USER_ID, STUDENT_WITHOUT_ROLE_USER_ID,
        },
    },
};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
use uuid::Uuid;

/// Constrói um pool preguiçoso para compor o AppState nos testes HTTP em memória.
fn create_test_state(auth_service: Arc<dyn AuthorizationServicePort>) -> AppState {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://capp:capp@localhost:5432/capp")
        .expect("pool preguiçoso para testes");
    AppState::with_authorization(pool, auth_service)
}

/// Cria o router de testes com endpoints protegidos por diferentes extratores declarativos.
fn create_test_auth_app(state: AppState) -> Router {
    Router::new()
        .route(
            "/test/management/members",
            get(|_auth: RequirePermission<ManageMembersPermission>| async {
                "Acesso concedido à gestão de integrantes."
            }),
        )
        .route(
            "/test/news/direct",
            post(
                |_auth: RequirePermission<PublishNewsDirectPermission>| async {
                    "Notícia publicada com sucesso."
                },
            ),
        )
        .route(
            "/test/news/approve",
            post(|_auth: RequirePermission<ApproveNewsPermission>| async {
                "Notícia homologada com sucesso."
            }),
        )
        .route(
            "/test/events/direct",
            post(
                |_auth: RequirePermission<PublishEventsDirectPermission>| async {
                    "Evento publicado com sucesso."
                },
            ),
        )
        .route(
            "/test/events/approve",
            post(|_auth: RequirePermission<ApproveEventsPermission>| async {
                "Evento homologado com sucesso."
            }),
        )
        .route(
            "/test/context-only",
            get(|ctx: ActiveCaContext| async move {
                format!(
                    "Contexto ativo: user={}, ca={}",
                    ctx.user_id, ctx.active_ca_id
                )
            }),
        )
        .with_state(state)
}

// =========================================================================
// 1. Cenário: Usuário sem cargo no CA recebendo recusa (Forbidden)
// =========================================================================

#[tokio::test]
async fn test_user_without_role_in_ca_receives_forbidden() {
    let auth_service = Arc::new(AuthorizationService::with_didactic_data().await);
    let state = create_test_state(auth_service.clone());
    let app = create_test_auth_app(state);

    // Consulta de serviço direta
    let service_result = auth_service
        .check_permission(
            STUDENT_WITHOUT_ROLE_USER_ID,
            CAER_DIDACTIC_ID,
            &Permission::ManageMembers,
        )
        .await;

    assert_eq!(
        service_result.unwrap_err(),
        AuthorizationServiceError::NoActiveRoleInCa {
            user_id: STUDENT_WITHOUT_ROLE_USER_ID,
            ca_id: CAER_DIDACTIC_ID,
        }
    );

    assert!(
        !auth_service
            .has_permission(
                STUDENT_WITHOUT_ROLE_USER_ID,
                CAER_DIDACTIC_ID,
                &Permission::ManageMembers,
            )
            .await
            .unwrap()
    );

    // Requisição HTTP via Axum
    let req = Request::builder()
        .uri("/test/management/members")
        .header("X-User-ID", STUDENT_WITHOUT_ROLE_USER_ID.to_string())
        .header("X-Active-CA-ID", CAER_DIDACTIC_ID.to_string())
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body_json["error"], "forbidden");
}

// =========================================================================
// 2. Cenário: Usuário com cargo em outro CA recebendo recusa (Isolamento)
// =========================================================================

#[tokio::test]
async fn test_user_with_role_in_another_ca_receives_forbidden() {
    let auth_service = Arc::new(AuthorizationService::with_didactic_data().await);
    let state = create_test_state(auth_service.clone());
    let app = create_test_auth_app(state);

    // PRESIDENTE_CAER_USER_ID é Presidente no CAER, mas tenta acessar CACC
    let service_result = auth_service
        .check_permission(
            PRESIDENTE_CAER_USER_ID,
            CACC_DIDACTIC_ID,
            &Permission::ManageMembers,
        )
        .await;

    assert_eq!(
        service_result.unwrap_err(),
        AuthorizationServiceError::AcademicCenterMismatch {
            role_ca_id: CAER_DIDACTIC_ID,
            target_ca_id: CACC_DIDACTIC_ID,
        }
    );

    assert!(
        !auth_service
            .has_permission(
                PRESIDENTE_CAER_USER_ID,
                CACC_DIDACTIC_ID,
                &Permission::ManageMembers,
            )
            .await
            .unwrap()
    );

    // Requisição HTTP via Axum
    let req = Request::builder()
        .uri("/test/management/members")
        .header("X-User-ID", PRESIDENTE_CAER_USER_ID.to_string())
        .header("X-Active-CA-ID", CACC_DIDACTIC_ID.to_string())
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body_json["error"], "forbidden");
}

// =========================================================================
// 3. Cenário: Homologador conseguindo publicar direto (Implicação de Domínio)
// =========================================================================

#[tokio::test]
async fn test_homologator_can_publish_directly_in_same_domain() {
    let auth_service = Arc::new(AuthorizationService::with_didactic_data().await);
    let state = create_test_state(auth_service.clone());
    let app = create_test_auth_app(state);

    // DIRETOR_COMUNICACAO_USER_ID possui ApproveNews no CAER
    // Deve conseguir publicar notícias diretamente
    let can_publish = auth_service
        .has_permission(
            DIRETOR_COMUNICACAO_USER_ID,
            CAER_DIDACTIC_ID,
            &Permission::PublishNewsDirect,
        )
        .await
        .unwrap();
    assert!(can_publish);

    auth_service
        .check_permission(
            DIRETOR_COMUNICACAO_USER_ID,
            CAER_DIDACTIC_ID,
            &Permission::PublishNewsDirect,
        )
        .await
        .expect("homologador deve ter permissão de publicação direta");

    // Requisição HTTP via Axum
    let req = Request::builder()
        .method("POST")
        .uri("/test/news/direct")
        .header("X-User-ID", DIRETOR_COMUNICACAO_USER_ID.to_string())
        .header("X-Active-CA-ID", CAER_DIDACTIC_ID.to_string())
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

// =========================================================================
// 4. Cenário: Publicador direto recebendo recusa ao tentar homologar
// =========================================================================

#[tokio::test]
async fn test_direct_publisher_refused_when_trying_to_homologate() {
    let auth_service = Arc::new(AuthorizationService::with_didactic_data().await);
    let state = create_test_state(auth_service.clone());
    let app = create_test_auth_app(state);

    // DIRETOR_EVENTOS_USER_ID possui PublishEventsDirect, mas NÃO ApproveEvents
    // Pode publicar eventos diretamente
    let can_publish = auth_service
        .has_permission(
            DIRETOR_EVENTOS_USER_ID,
            CAER_DIDACTIC_ID,
            &Permission::PublishEventsDirect,
        )
        .await
        .unwrap();
    assert!(can_publish);

    // NÃO pode homologar eventos
    let can_approve = auth_service
        .has_permission(
            DIRETOR_EVENTOS_USER_ID,
            CAER_DIDACTIC_ID,
            &Permission::ApproveEvents,
        )
        .await
        .unwrap();
    assert!(!can_approve);

    let check_err = auth_service
        .check_permission(
            DIRETOR_EVENTOS_USER_ID,
            CAER_DIDACTIC_ID,
            &Permission::ApproveEvents,
        )
        .await
        .unwrap_err();
    assert!(matches!(check_err, AuthorizationServiceError::Forbidden(_)));

    // Requisição HTTP via Axum
    let req = Request::builder()
        .method("POST")
        .uri("/test/events/approve")
        .header("X-User-ID", DIRETOR_EVENTOS_USER_ID.to_string())
        .header("X-Active-CA-ID", CAER_DIDACTIC_ID.to_string())
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// =========================================================================
// 5. Cenário: Membro desvinculado perdendo autorização imediatamente
// =========================================================================

#[tokio::test]
async fn test_delinked_member_loses_authorization_immediately() {
    let user_id = Uuid::new_v4();
    let ca_id = CAER_DIDACTIC_ID;

    // Configura cargo de Diretor de Materiais
    let role = Role::new(
        Uuid::new_v4(),
        ca_id,
        "Diretor de Materiais",
        HashSet::from([Permission::ManageMaterials]),
    )
    .unwrap();

    let role_repo =
        Arc::new(backend::modules::ca::infrastructure::repository::InMemoryRoleRepository::new());
    role_repo.save_role(&role).await.unwrap();

    let assignment_repo = Arc::new(
        backend::modules::ca::infrastructure::repository::InMemoryRoleAssignmentRepository::new(),
    );
    let service = AuthorizationService::new(role_repo, assignment_repo.clone());

    // Vincula o usuário ao cargo
    let assignment = service.assign_role(user_id, ca_id, role.id).await.unwrap();

    // Usuário possui permissão ativa
    assert!(
        service
            .has_permission(user_id, ca_id, &Permission::ManageMaterials)
            .await
            .unwrap()
    );

    // Desvincula o usuário
    service.revoke_assignment(assignment.id).await.unwrap();

    // Permissão deixa de ser concedida imediatamente
    assert!(
        !service
            .has_permission(user_id, ca_id, &Permission::ManageMaterials)
            .await
            .unwrap()
    );

    let err = service
        .check_permission(user_id, ca_id, &Permission::ManageMaterials)
        .await
        .unwrap_err();
    assert!(matches!(err, AuthorizationServiceError::Forbidden(_)));

    // Histórico administrativo é preservado e registra a revogação
    let history = assignment_repo
        .find_history_by_user_and_ca(user_id, ca_id)
        .await
        .unwrap();
    assert_eq!(history.len(), 1);
    assert!(!history[0].is_active);
    assert!(history[0].revoked_at.is_some());
}

// =========================================================================
// 6. Cenário: Middleware require_permission funcional no Axum
// =========================================================================

#[tokio::test]
async fn test_axum_require_permission_middleware() {
    let auth_service = Arc::new(AuthorizationService::with_didactic_data().await);
    let auth_service_clone = auth_service.clone();

    // Rota protegida via middleware funcional em tempo de execução
    let app = Router::new()
        .route(
            "/protected/news",
            post(|| async { "Acesso autorizado por middleware." }),
        )
        .layer(from_fn(move |req, next| {
            let auth = auth_service_clone.clone();
            async move {
                require_permission_middleware(auth, Permission::PublishNewsDirect, req, next).await
            }
        }));

    // 1. Homologador pode publicar direto através do middleware
    let req_authorized = Request::builder()
        .method("POST")
        .uri("/protected/news")
        .header("X-User-ID", DIRETOR_COMUNICACAO_USER_ID.to_string())
        .header("X-Active-CA-ID", CAER_DIDACTIC_ID.to_string())
        .body(Body::empty())
        .unwrap();

    let res_authorized = app.clone().oneshot(req_authorized).await.unwrap();
    assert_eq!(res_authorized.status(), StatusCode::OK);

    // 2. Estudante sem cargo é bloqueado pelo middleware
    let req_unauthorized = Request::builder()
        .method("POST")
        .uri("/protected/news")
        .header("X-User-ID", STUDENT_WITHOUT_ROLE_USER_ID.to_string())
        .header("X-Active-CA-ID", CAER_DIDACTIC_ID.to_string())
        .body(Body::empty())
        .unwrap();

    let res_unauthorized = app.oneshot(req_unauthorized).await.unwrap();
    assert_eq!(res_unauthorized.status(), StatusCode::FORBIDDEN);
}

// =========================================================================
// 7. Cenário: Validação de cabeçalhos HTTP e códigos de erro adequados
// =========================================================================

#[tokio::test]
async fn test_axum_header_validations() {
    let auth_service = Arc::new(AuthorizationService::with_didactic_data().await);
    let state = create_test_state(auth_service);
    let app = create_test_auth_app(state);

    // Falta cabeçalho de usuário -> 401 Unauthorized
    let req_missing_user = Request::builder()
        .uri("/test/context-only")
        .header("X-Active-CA-ID", CAER_DIDACTIC_ID.to_string())
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req_missing_user).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Falta cabeçalho de CA ativo -> 400 Bad Request
    let req_missing_ca = Request::builder()
        .uri("/test/context-only")
        .header("X-User-ID", PRESIDENTE_CAER_USER_ID.to_string())
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req_missing_ca).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // UUID inválido -> 400 Bad Request
    let req_invalid_uuid = Request::builder()
        .uri("/test/context-only")
        .header("X-User-ID", "nao-e-um-uuid")
        .header("X-Active-CA-ID", CAER_DIDACTIC_ID.to_string())
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req_invalid_uuid).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

// =========================================================================
// 8. Cenário: Segregação estrita de escopos (Gestão vs Eleitoral vs Plataforma)
// =========================================================================

#[test]
fn test_strict_scope_segregation() {
    assert_eq!(Permission::ManageMembers.scope(), Scope::CaManagement);
    assert_eq!(Permission::ApproveNews.scope(), Scope::CaManagement);
    assert_eq!(
        Permission::PublishFinancialStatements.scope(),
        Scope::CaManagement
    );

    assert_eq!(
        Permission::ConfigureElection.scope(),
        Scope::ElectoralCommission
    );
    assert_eq!(Permission::ManageSlates.scope(), Scope::ElectoralCommission);
    assert_eq!(Permission::TallyVotes.scope(), Scope::ElectoralCommission);

    assert_eq!(
        Permission::AdministerPlatform.scope(),
        Scope::PlatformAdministration
    );
    assert_eq!(
        Permission::VerifyAcademicCenter.scope(),
        Scope::PlatformAdministration
    );

    // O Presidente do CA não possui permissão de apuração eleitoral ou administração geral da plataforma
    let pres = Role::presidente(CAER_DIDACTIC_ID);
    assert!(!pres.has_permission(&Permission::TallyVotes));
    assert!(!pres.has_permission(&Permission::AdministerPlatform));
}
