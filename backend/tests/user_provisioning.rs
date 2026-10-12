//! Testes de integração para o provisionamento automático de usuários (Issue #7 / I01).
//!
//! Valida os cenários de primeiro acesso autenticado, idempotência em acessos
//! subsequentes, inicialização de preferências com `favorite_ca_id = None`,
//! preservação do isolamento de privilégios e tratamento de erros de domínio da UnB.

use backend::modules::identity::{
    AppUserError, AppUserStatus, ExternalIdentityError, ProvisionUserCommand,
    UserProvisioningError, create_in_memory_user_provisioning_service,
};

#[tokio::test]
async fn should_provision_user_on_first_authenticated_access() {
    let service = create_in_memory_user_provisioning_service();

    let cmd = ProvisionUserCommand::new(
        "firebase-calouro-test",
        "ms-subject-calouro-test",
        "262001234@aluno.unb.br",
    )
    .with_display_name("Estudante Calouro UnB");

    let result = service
        .provision(cmd)
        .await
        .expect("provisionamento inicial");

    // Valida primeiro acesso e perfil interno
    assert!(
        result.is_new_user,
        "deve ser identificado como novo usuário"
    );
    assert_eq!(result.user.status, AppUserStatus::Active);
    assert_eq!(result.user.institutional_email, "262001234@aluno.unb.br");
    assert_eq!(result.user.display_name, "Estudante Calouro UnB");

    // Valida associação unívoca com a identidade externa
    assert_eq!(result.external_identity.user_id, result.user.id);
    assert_eq!(
        result.external_identity.firebase_uid,
        "firebase-calouro-test"
    );
    assert_eq!(
        result.external_identity.microsoft_subject,
        "ms-subject-calouro-test"
    );
    assert!(result.external_identity.last_login_at.is_some());

    // Valida inicialização de preferências com favorite_ca_id = None
    assert_eq!(result.preference.user_id, result.user.id);
    assert_eq!(result.preference.favorite_ca_id, None);

    // Valida isolamento de privilégios
    assert!(
        !result.has_administrative_privileges,
        "o provisionamento não pode conceder privilégios administrativos"
    );
    assert!(
        !service.has_administrative_privileges(result.user.id).await,
        "usuário provisionado deve permanecer sem privilégios administrativos"
    );
}

#[tokio::test]
async fn should_reuse_profile_idempotently_on_subsequent_accesses() {
    let service = create_in_memory_user_provisioning_service();

    let cmd = ProvisionUserCommand::new(
        "firebase-veterano-test",
        "ms-subject-veterano-test",
        "231012345@aluno.unb.br",
    )
    .with_display_name("Estudante Veterano UnB");

    // Primeiro acesso
    let first = service
        .provision(cmd.clone())
        .await
        .expect("primeiro login");
    assert!(first.is_new_user);
    let original_user_id = first.user.id;
    let first_login_time = first.external_identity.last_login_at;

    // Segundo acesso com a mesma identidade externa
    let second = service.provision(cmd.clone()).await.expect("segundo login");
    assert!(!second.is_new_user, "não deve recriar o usuário");
    assert_eq!(
        second.user.id, original_user_id,
        "o identificador interno deve ser preservado de forma estrita"
    );
    assert_eq!(second.user.institutional_email, "231012345@aluno.unb.br");
    assert!(
        second.external_identity.last_login_at >= first_login_time,
        "o instante do último login deve ser atualizado"
    );

    // Terceiro acesso consecutivo (múltiplas reautenticações consecutivas)
    let third = service.provision(cmd).await.expect("terceiro login");
    assert!(!third.is_new_user);
    assert_eq!(third.user.id, original_user_id);
}

#[tokio::test]
async fn should_strictly_reject_prohibited_matricula_242003979() {
    let service = create_in_memory_user_provisioning_service();

    let cmd = ProvisionUserCommand::new(
        "firebase-proibido",
        "ms-subject-proibido",
        "242003979@aluno.unb.br",
    );

    let error = service.provision(cmd).await.unwrap_err();
    match error {
        UserProvisioningError::User(AppUserError::ProhibitedRegistration(mat)) => {
            assert_eq!(mat, "242003979");
        }
        other => panic!("erro inesperado para matrícula proibida: {other:?}"),
    }
}

#[tokio::test]
async fn should_reject_non_institutional_email() {
    let service = create_in_memory_user_provisioning_service();

    let cmd =
        ProvisionUserCommand::new("firebase-external", "ms-external", "estudante@outlook.com");

    let error = service.provision(cmd).await.unwrap_err();
    assert!(matches!(
        error,
        UserProvisioningError::InvalidInstitutionalDomain { .. }
    ));
}

#[tokio::test]
async fn should_reject_empty_credentials() {
    let service = create_in_memory_user_provisioning_service();

    let cmd = ProvisionUserCommand::new("", "ms-subject", "262001234@aluno.unb.br");
    let error = service.provision(cmd).await.unwrap_err();
    assert!(matches!(
        error,
        UserProvisioningError::ExternalIdentity(ExternalIdentityError::EmptyFirebaseUid)
    ));
}
