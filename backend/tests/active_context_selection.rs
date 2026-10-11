//! Testes de integração para a seleção de contexto ativo de CA (Issue #45).
//!
//! Valida a integridade da entrega incremental I01 ("Acesso e meu CA"), comprovando:
//! 1. Representação do contexto ativo de navegação em sessão/memória.
//! 2. Alternância entre os CAs cadastrados da UnB (CAER e CACC).
//! 3. Regra estrita: alterar o contexto ativo não sobrescreve a preferência de CA favorito gravada.
//! 4. Isolamento estrito de permissões e manutenção de cargos distintos por CA.

use backend::{
    domain::{
        academic_center::{CACC_DIDACTIC_ID, CAER_DIDACTIC_ID},
        active_context::{
            ActiveCaContext, ActiveContextError, ActiveContextSource, ManagementRoleAssignment,
        },
        app_user::{AppUser, VETERANO_1_DIDACTIC_ID, VETERANO_2_DIDACTIC_ID},
        user_preference::UserPreference,
    },
    modules::{
        ca::{
            application::ActiveContextService,
            domain::rules::{
                assert_favorite_preference_unaltered_by_context_switch,
                assert_navigation_only_active_context_scope,
                has_academic_enrollment_via_active_context,
                has_administrative_privileges_via_active_context,
                is_eligible_to_vote_via_active_context,
            },
            infrastructure::{InMemoryAcademicCenterRepository, InMemoryActiveContextSessionStore},
        },
        identity::infrastructure::repository::{
            InMemoryUserPreferenceRepository, UserPreferenceRepository,
        },
    },
};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn should_select_and_open_app_with_favorite_ca_context() {
    let service = ActiveContextService::with_didactic_setup();
    let veterano = AppUser::veterano_didactic();

    // 1. Abertura do aplicativo: contexto ativo inicial adota o CA favorito (CAER)
    let context = service
        .open_app(veterano.id)
        .await
        .expect("abertura do app deve ser bem-sucedida");

    assert_eq!(context.user_id(), VETERANO_1_DIDACTIC_ID);
    assert_eq!(context.active_ca_id(), CAER_DIDACTIC_ID);
    assert_eq!(context.source(), ActiveContextSource::FavoritePreference);
    assert!(context.is_active(&CAER_DIDACTIC_ID));
    assert!(!context.is_active(&CACC_DIDACTIC_ID));

    // 2. Não possui privilégios administrativos apenas por tê-lo como ativo/favorito
    assert!(context.is_navigation_only());
    assert!(!context.confers_administrative_permissions());
    assert!(!context.confers_academic_enrollment());
    assert!(!context.confers_electoral_eligibility());
    assert!(assert_navigation_only_active_context_scope(&context).is_ok());
    assert!(!has_administrative_privileges_via_active_context(
        &context,
        &CAER_DIDACTIC_ID
    ));
}

#[tokio::test]
async fn should_switch_active_context_from_caer_to_cacc_without_overwriting_favorite_preference() {
    let pref_repo = Arc::new(InMemoryUserPreferenceRepository::with_didactic_data());
    let session_store = Arc::new(InMemoryActiveContextSessionStore::new());
    let ca_repo = Arc::new(InMemoryAcademicCenterRepository::with_didactic_cas());

    let service =
        ActiveContextService::new(session_store.clone(), pref_repo.clone(), ca_repo.clone());

    let veterano_2 = AppUser::veterano_2_didactic();

    // 1. Define expressamente CAER como favorito gravado do estudante veterano 2
    let pref_inicial = UserPreference::new(veterano_2.id, Some(CAER_DIDACTIC_ID))
        .expect("preferência válida com CAER");
    pref_repo
        .save(&pref_inicial)
        .await
        .expect("deve salvar preferência inicial");

    // 2. Abre o aplicativo -> contexto ativo inicial em CAER
    let initial_context = service
        .open_app(veterano_2.id)
        .await
        .expect("abertura do aplicativo");
    assert_eq!(initial_context.active_ca_id(), CAER_DIDACTIC_ID);

    // 3. Alterna o contexto ativo em sessão para CACC
    let switched_context = service
        .switch_active_ca(veterano_2.id, CACC_DIDACTIC_ID)
        .await
        .expect("deve alternar para CACC");

    assert_eq!(switched_context.active_ca_id(), CACC_DIDACTIC_ID);
    assert_eq!(
        switched_context.source(),
        ActiveContextSource::ExplicitSelection
    );

    // 4. REGRA ESTRITA: A preferência de CA favorito persistida PERMANECE CAER
    let stored_preference = pref_repo
        .find_by_user_id(veterano_2.id)
        .await
        .expect("consulta ao banco/memória")
        .expect("preferência deve continuar existindo");

    assert_eq!(stored_preference.user_id(), VETERANO_2_DIDACTIC_ID);
    assert_eq!(
        stored_preference.favorite_ca_id(),
        Some(CAER_DIDACTIC_ID),
        "a preferência gravada NÃO pode ter sido sobrescrita para CACC"
    );
    assert!(stored_preference.is_favorite(&CAER_DIDACTIC_ID));
    assert!(!stored_preference.is_favorite(&CACC_DIDACTIC_ID));

    // Validação formal da regra de domínio de integridade
    assert!(
        assert_favorite_preference_unaltered_by_context_switch(
            &stored_preference,
            &switched_context
        )
        .is_ok()
    );
}

#[tokio::test]
async fn should_enforce_strict_privilege_isolation_when_switching_context() {
    let service = ActiveContextService::with_didactic_setup();
    let veterano = AppUser::veterano_2_didactic();

    // O estudante exerce o cargo de "Diretor de Comunicação" em CAER
    let role_caer = ManagementRoleAssignment::new(
        veterano.id,
        CAER_DIDACTIC_ID,
        "Diretor de Comunicação",
        vec!["Publicar notícias diretamente".to_string()],
    )
    .expect("cargo válido em CAER");

    service.assign_management_role(role_caer).await;

    // 1. Quando está com o contexto ativo em CAER:
    service.open_app(veterano.id).await.expect("open app");

    assert!(
        service
            .has_active_permission(veterano.id, "Publicar notícias diretamente")
            .await,
        "deve ter permissão no CA ativo CAER"
    );
    assert!(
        !service.is_visitor_in_active_context(veterano.id).await,
        "não deve ser visitante em CAER"
    );

    // 2. Quando altera o contexto ativo para CACC (onde não possui cargo):
    service
        .switch_active_ca(veterano.id, CACC_DIDACTIC_ID)
        .await
        .expect("alterna para CACC");

    assert!(
        !service
            .has_active_permission(veterano.id, "Publicar notícias diretamente")
            .await,
        "permissão de CAER NÃO pode vazar para o contexto ativo CACC"
    );
    assert!(
        service.is_visitor_in_active_context(veterano.id).await,
        "deve atuar estritamente como visitante no novo contexto CACC"
    );
}

#[tokio::test]
async fn should_maintain_distinct_isolated_roles_across_multiple_cas() {
    let service = ActiveContextService::with_didactic_setup();
    let veterano = AppUser::veterano_didactic();

    // Presidente em CAER
    let cargo_caer = ManagementRoleAssignment::new(
        veterano.id,
        CAER_DIDACTIC_ID,
        "Presidente",
        vec!["Administrar integrantes da gestão".to_string()],
    )
    .expect("presidente em CAER");

    // Colaborador de Eventos em CACC
    let cargo_cacc = ManagementRoleAssignment::new(
        veterano.id,
        CACC_DIDACTIC_ID,
        "Colaborador de Eventos",
        vec!["Publicar e gerenciar eventos diretamente".to_string()],
    )
    .expect("colaborador em CACC");

    service.assign_management_role(cargo_caer).await;
    service.assign_management_role(cargo_cacc).await;

    // 1. No contexto ativo CAER:
    service.open_app(veterano.id).await.expect("open app");

    assert!(
        service
            .has_active_permission(veterano.id, "Administrar integrantes da gestão")
            .await,
        "deve ter permissão de presidente em CAER"
    );
    assert!(
        !service
            .has_active_permission(veterano.id, "Publicar e gerenciar eventos diretamente")
            .await,
        "NÃO deve ter permissão de eventos de CACC quando ativo em CAER"
    );

    // 2. No contexto ativo CACC:
    service
        .switch_active_ca(veterano.id, CACC_DIDACTIC_ID)
        .await
        .expect("alterna para CACC");

    assert!(
        service
            .has_active_permission(veterano.id, "Publicar e gerenciar eventos diretamente")
            .await,
        "deve ter permissão de eventos em CACC"
    );
    assert!(
        !service
            .has_active_permission(veterano.id, "Administrar integrantes da gestão")
            .await,
        "NÃO deve ter permissão de presidente de CAER quando ativo em CACC"
    );
}

#[tokio::test]
async fn should_reject_switching_to_unregistered_or_inactive_cas() {
    let service = ActiveContextService::with_didactic_setup();
    let veterano = AppUser::veterano_didactic();
    service.open_app(veterano.id).await.expect("open app");

    // CA inexistente
    let fake_ca_id = Uuid::new_v4();
    let err = service
        .switch_active_ca(veterano.id, fake_ca_id)
        .await
        .unwrap_err();
    assert_eq!(err, ActiveContextError::AcademicCenterNotFound(fake_ca_id));

    // UUID nulo
    let err_nil = service
        .switch_active_ca(veterano.id, Uuid::nil())
        .await
        .unwrap_err();
    assert_eq!(err_nil, ActiveContextError::NilAcademicCenterId);
}

#[test]
fn should_strictly_prohibit_academic_or_electoral_privilege_escalation_from_context_alone() {
    let context = ActiveCaContext::new(VETERANO_1_DIDACTIC_ID, CAER_DIDACTIC_ID)
        .expect("valid active context");

    assert!(context.is_navigation_only());
    assert!(!context.confers_administrative_permissions());
    assert!(!context.confers_academic_enrollment());
    assert!(!context.confers_electoral_eligibility());

    assert!(assert_navigation_only_active_context_scope(&context).is_ok());
    assert!(!has_administrative_privileges_via_active_context(
        &context,
        &CAER_DIDACTIC_ID
    ));
    assert!(!has_academic_enrollment_via_active_context(
        &context,
        &CAER_DIDACTIC_ID
    ));
    assert!(!is_eligible_to_vote_via_active_context(
        &context,
        &CAER_DIDACTIC_ID
    ));
}
