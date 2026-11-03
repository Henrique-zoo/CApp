//! Testes unitários dos casos de uso de autenticação institucional.
//!
//! Exercitam decisões e interações por mocks, sem Firebase, Graph ou PostgreSQL reais.
//! Métodos sem expectativas configuradas rejeitam chamadas inesperadas. O fluxo
//! consulta tenant, instituição e grupo antes de obter o perfil com o vínculo
//! correspondente. Estudantes e não estudantes podem concluir a verificação.

use chrono::{Duration, Utc};
use mockall::predicate::eq;
use uuid::Uuid;

use super::{authenticate_institutional_request, verify_institutional_identity};
use crate::modules::identity::{
    application::{
        error::{
            AuthenticationError, InstitutionRepositoryError,
            InstitutionalVerificationRepositoryError,
        },
        ports::{InstitutionalAffiliation, InstitutionalProfile, VerifiedIdentity},
    },
    domain::entities::{
        institution::Institution, institutional_verification::InstitutionalVerification,
    },
};

mod fixtures;
mod mocks;

use fixtures::*;
use mocks::*;

/// Configura uma única validação do token fictício com identidade Microsoft vinculada.
fn verifier_ok() -> MockVerifier {
    let mut verifier = MockVerifier::new();
    verifier
        .expect_verify()
        .with(eq(FIREBASE_TOKEN))
        .times(1)
        .returning(|_| Ok(valid_identity()));

    verifier
}

/// Configura somente a consulta do tenant, sem permitir consulta de grupo ou perfil.
fn directory_tenant_only() -> MockDirectory {
    let mut directory = MockDirectory::new();
    directory
        .expect_get_tenant_id()
        .with(eq(MICROSOFT_TOKEN))
        .times(1)
        .returning(|_| Ok(tenant_id()));
    directory
}

/// Configura tenant e participação no grupo, deixando a obtenção do perfil ao caso.
fn directory_membership(is_student: bool) -> MockDirectory {
    let mut directory = directory_tenant_only();
    directory
        .expect_is_student_member()
        .with(eq(MICROSOFT_TOKEN), eq(student_group_id()))
        .times(1)
        .returning(move |_, _| Ok(is_student));
    directory
}

/// Configura as consultas do diretório com o identificador e o vínculo fornecidos.
///
/// Exige que o serviço repasse a confirmação do grupo ao solicitar o perfil.
fn directory_with_profile(id: &str, is_student: bool) -> MockDirectory {
    let mut directory = directory_membership(is_student);
    let profile = profile_with_id(id, is_student);
    directory
        .expect_get_profile()
        .with(eq(MICROSOFT_TOKEN), eq(is_student))
        .times(1)
        .return_once(move |_, _| Ok(profile));
    directory
}

/// Configura o diretório com identidade compatível e a participação indicada no grupo.
fn directory_success(is_student: bool) -> MockDirectory {
    directory_with_profile(MICROSOFT_OBJECT_ID, is_student)
}

/// Configura a busca da instituição fictícia pelo tenant, sem permitir gravações.
fn repositories_institution() -> MockRepositories {
    let mut repo = MockRepositories::new();
    repo.expect_find_by_microsoft_tenant_id()
        .with(eq(tenant_id()))
        .times(1)
        .returning(|_| Ok(Some(institution())));

    repo
}

/// Verifica a persistência de uma confirmação estudantil positiva e seu prazo de validade.
///
/// Configura Firebase, perfil, tenant e grupo compatíveis. Exige uma gravação com
/// todos os identificadores e TTL de 24 horas; confere o ID do perfil retornado e
/// situa sua expiração entre os instantes medidos antes e depois da operação.
#[tokio::test]
async fn first_verification_persists_positive_evidence_with_all_identifiers() {
    let verifier = verifier_ok();
    let directory = directory_success(true);
    let mut repo = repositories_institution();

    repo.expect_save()
        .times(1)
        .withf(|v| {
            v.firebase_uid == FIREBASE_UID
                && v.microsoft_object_id == microsoft_object_id()
                && v.microsoft_tenant_id == tenant_id()
                && v.microsoft_student_group_id == student_group_id()
                && v.institution_id == institution_id()
                && v.student_membership_confirmed
                && v.expires_at == v.checked_at + Duration::hours(24)
        })
        .returning(|_| Ok(true));

    let before = Utc::now();
    let result = verify_institutional_identity(
        &directory,
        &verifier,
        &repo,
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await
    .expect("estudante válido deve ser aprovado");

    let after = Utc::now();

    assert_eq!(result.profile.id, MICROSOFT_OBJECT_ID);
    assert!(matches!(
        result.profile.affiliation,
        InstitutionalAffiliation::Student { course_name: Some(ref name) }
            if name == "Curso de Teste"
    ));
    assert!(result.expires_at >= before + Duration::hours(24));
    assert!(result.expires_at <= after + Duration::hours(24));
}

/// Verifica que um token Firebase inválido interrompe a verificação institucional.
///
/// O verificador retorna `InvalidToken`, que deve ser propagado. Diretório e
/// repositórios não têm expectativas configuradas: qualquer chamada a eles falha.
#[tokio::test]
async fn invalid_firebase_token_stops_before_any_graph_or_database_calls() {
    let mut verifier = MockVerifier::new();
    let directory = MockDirectory::new();
    let repo = MockRepositories::new();

    verifier
        .expect_verify()
        .with(eq(FIREBASE_TOKEN))
        .times(1)
        .returning(|_| Err(AuthenticationError::InvalidToken));

    let result = verify_institutional_identity(
        &directory,
        &verifier,
        &repo,
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await;
    assert!(matches!(result, Err(AuthenticationError::InvalidToken)));
    // Mockall falha caso seja chamado método não configurado.
}

/// Verifica a recusa de uma identidade Firebase sem conta Microsoft vinculada.
///
/// Retorna `InvalidMicrosoftId` antes de consultar o diretório ou a persistência,
/// cujos mocks não permitem chamadas neste caso.
#[tokio::test]
async fn missing_microsoft_identity_stops_before_graph() {
    let mut verifier = MockVerifier::new();

    verifier.expect_verify().times(1).returning(|_| {
        Ok(VerifiedIdentity {
            firebase_uid: FIREBASE_UID.into(),
            microsoft_object_id: None,
        })
    });

    let result = verify_institutional_identity(
        &MockDirectory::new(),
        &verifier,
        &MockRepositories::new(),
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await;

    assert!(matches!(
        result,
        Err(AuthenticationError::InvalidMicrosoftId)
    ));
}

/// Verifica a recusa de um identificador Microsoft vinculado que não é UUID.
///
/// Mesmo com Firebase validado, o serviço deve retornar `InvalidMicrosoftId`
/// sem consultar o diretório nem os repositórios.
#[tokio::test]
async fn malformed_microsoft_identity_is_rejected() {
    let mut verifier = MockVerifier::new();

    verifier.expect_verify().times(1).returning(|_| {
        Ok(VerifiedIdentity {
            firebase_uid: FIREBASE_UID.into(),
            microsoft_object_id: Some("not-a-uuid".into()),
        })
    });

    let result = verify_institutional_identity(
        &MockDirectory::new(),
        &verifier,
        &MockRepositories::new(),
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await;

    assert!(matches!(
        result,
        Err(AuthenticationError::InvalidMicrosoftId)
    ));
}

/// Verifica que identidades divergentes impedem a gravação da evidência.
///
/// Tenant, instituição e grupo são consultados antes do perfil. Mesmo com vínculo
/// confirmado, um ID diferente do vínculo Firebase retorna `IdentityMismatch`;
/// o repositório não permite nenhuma gravação.
#[tokio::test]
async fn mismatched_graph_identity_stops_before_persistence() {
    let verifier = verifier_ok();
    let directory = directory_with_profile("11111111-2222-3333-4444-555555555555", true);
    let result = verify_institutional_identity(
        &directory,
        &verifier,
        &repositories_institution(),
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await;

    assert!(matches!(result, Err(AuthenticationError::IdentityMismatch)));
}

/// Verifica a recusa de um perfil externo cujo identificador não é UUID.
///
/// Após consultar tenant, instituição e grupo, espera `ProviderUnavailable`
/// ao interpretar o perfil. Nenhuma evidência deve ser gravada.
#[tokio::test]
async fn malformed_graph_id_means_unavailable_profile() {
    let verifier = verifier_ok();
    let directory = directory_with_profile("invalid-uuid", true);
    let result = verify_institutional_identity(
        &directory,
        &verifier,
        &repositories_institution(),
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await;

    assert!(matches!(
        result,
        Err(AuthenticationError::ProviderUnavailable)
    ));
}

/// Distingue instituição não cadastrada de uma falha técnica de consulta.
///
/// Após obter o tenant, o repositório retorna `Ok(None)`. Espera
/// `UnknownInstitution`, sem consultar grupo ou perfil e sem gravar evidência.
#[tokio::test]
async fn unknown_tenant_is_a_business_rejection_not_a_database_failure() {
    let verifier = verifier_ok();
    let directory = directory_tenant_only();
    let mut repo = MockRepositories::new();

    repo.expect_find_by_microsoft_tenant_id()
        .with(eq(tenant_id()))
        .times(1)
        .returning(|_| Ok(None));

    let result = verify_institutional_identity(
        &directory,
        &verifier,
        &repo,
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await;

    assert!(matches!(
        result,
        Err(AuthenticationError::UnknownInstitution)
    ));
}

/// Verifica que a ausência de vínculo estudantil permite concluir a verificação.
///
/// Exige que `get_profile` receba `false` e que a evidência seja gravada com
/// `student_membership_confirmed = false`. O perfil retornado é `NonStudent`;
/// este teste não concede acesso a funcionalidades restritas a cursos.
#[tokio::test]
async fn non_student_membership_is_saved_and_profile_is_returned() {
    let verifier = verifier_ok();
    let directory = directory_success(false);
    let mut repo = repositories_institution();

    repo.expect_save()
        .times(1)
        .withf(|v| !v.student_membership_confirmed)
        .returning(|_| Ok(true));

    let result = verify_institutional_identity(
        &directory,
        &verifier,
        &repo,
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await
    .expect("não estudantes também devem concluir a verificação");

    assert_eq!(result.profile.id, MICROSOFT_OBJECT_ID);
    assert!(matches!(
        result.profile.affiliation,
        InstitutionalAffiliation::NonStudent
    ));
}

/// Verifica que uma confirmação positiva não autoriza se a gravação for descartada.
///
/// Simula `save` retornando `false`, como ocorre diante de evidência igual ou mais
/// recente. Espera `InstitutionalVerificationRequired`; a ordenação real das
/// escritas pertence aos testes do repositório.
#[tokio::test]
async fn positive_membership_cannot_authorize_when_evidence_was_not_saved() {
    let verifier = verifier_ok();
    let directory = directory_success(true);
    let mut repo = repositories_institution();

    repo.expect_save().times(1).returning(|_| Ok(false));

    let result = verify_institutional_identity(
        &directory,
        &verifier,
        &repo,
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await;

    assert!(matches!(
        result,
        Err(AuthenticationError::InstitutionalVerificationRequired)
    ));
}

/// Verifica a autenticação com evidência positiva, compatível e ainda vigente.
///
/// Exige uma consulta pelo UID Firebase e confere UID e instituição retornados.
/// O caso de uso não recebe diretório Microsoft; nenhuma nova consulta ao Graph
/// ou gravação de evidência é necessária.
#[tokio::test]
async fn valid_persisted_evidence_allows_later_requests_without_graph() {
    let verifier = verifier_ok();
    let mut repo = MockRepositories::new();

    repo.expect_find_valid_by_firebase_uid()
        .with(eq(FIREBASE_UID))
        .times(1)
        .returning(|_| Ok(Some(evidence(true, Duration::hours(1)))));

    let authenticated = authenticate_institutional_request(&verifier, &repo, FIREBASE_TOKEN)
        .await
        .expect("verificação positiva válida deve liberar acesso");

    assert_eq!(authenticated.firebase_uid, FIREBASE_UID);
    assert_eq!(authenticated.institution_id, institution_id());
}

/// Verifica a autenticação de não estudantes com evidência vigente já persistida.
///
/// Simula o novo contrato da busca, que também deve devolver evidências com vínculo
/// estudantil não confirmado. Confere UID e instituição sem novas consultas ao Graph.
/// O filtro SQL real precisa ser validado separadamente nos testes do repositório.
#[tokio::test]
async fn valid_non_student_evidence_allows_later_authentication() {
    let verifier = verifier_ok();
    let mut repo = MockRepositories::new();

    repo.expect_find_valid_by_firebase_uid()
        .with(eq(FIREBASE_UID))
        .times(1)
        .returning(|_| Ok(Some(evidence(false, Duration::hours(1)))));

    let authenticated = authenticate_institutional_request(&verifier, &repo, FIREBASE_TOKEN)
        .await
        .expect("evidência vigente de não estudante deve permitir autenticação");

    assert_eq!(authenticated.firebase_uid, FIREBASE_UID);
    assert_eq!(authenticated.institution_id, institution_id());
}

/// Verifica a exigência de nova verificação quando não existe evidência reutilizável.
///
/// O repositório retorna `None` para o UID autenticado. Espera
/// `InstitutionalVerificationRequired`, sem gravar ou renovar a evidência.
#[tokio::test]
async fn missing_evidence_requires_fresh_institutional_verification() {
    let verifier = verifier_ok();
    let mut repo = MockRepositories::new();

    repo.expect_find_valid_by_firebase_uid()
        .with(eq(FIREBASE_UID))
        .times(1)
        .returning(|_| Ok(None));

    let result = authenticate_institutional_request(&verifier, &repo, FIREBASE_TOKEN).await;

    assert!(matches!(
        result,
        Err(AuthenticationError::InstitutionalVerificationRequired)
    ));
}

/// Verifica que uma evidência de outra conta Microsoft não pode ser reutilizada.
///
/// O mock retorna evidência vigente com UUID Microsoft divergente do token.
/// Espera `InstitutionalVerificationRequired`, mesmo com UID e prazo válidos.
#[tokio::test]
async fn another_microsoft_account_cannot_reuse_saved_verification() {
    let verifier = verifier_ok();
    let mut repo = MockRepositories::new();

    repo.expect_find_valid_by_firebase_uid()
        .times(1)
        .returning(|_| {
            let mut v = evidence(true, Duration::hours(1));
            v.microsoft_object_id =
                Uuid::parse_str("11111111-2222-3333-4444-555555555555").unwrap();
            Ok(Some(v))
        });

    let result = authenticate_institutional_request(&verifier, &repo, FIREBASE_TOKEN).await;

    assert!(matches!(
        result,
        Err(AuthenticationError::InstitutionalVerificationRequired)
    ));
}

/// Verifica a revalidação do prazo de uma evidência devolvida pelo repositório.
///
/// O mock fornece evidência expirada há um segundo. Espera
/// `InstitutionalVerificationRequired`; o caso exercita o bloqueio de expiração,
/// mas não simula uma consulta lenta nem o instante exato do limite.
#[tokio::test]
async fn expired_evidence_cannot_be_reused_even_if_repository_returns_it() {
    let verifier = verifier_ok();
    let mut repo = MockRepositories::new();

    repo.expect_find_valid_by_firebase_uid()
        .times(1)
        .returning(|_| Ok(Some(evidence(true, Duration::seconds(-1)))));

    let result = authenticate_institutional_request(&verifier, &repo, FIREBASE_TOKEN).await;

    assert!(matches!(
        result,
        Err(AuthenticationError::InstitutionalVerificationRequired)
    ));
}

/// Verifica a propagação de token Firebase expirado na verificação institucional.
///
/// O verificador retorna `ExpiredToken`. Diretório e repositórios não permitem
/// chamadas, comprovando a interrupção antes das demais dependências.
#[tokio::test]
async fn firebase_expiration_error_is_propagated_without_graph() {
    let mut verifier = MockVerifier::new();

    verifier
        .expect_verify()
        .times(1)
        .returning(|_| Err(AuthenticationError::ExpiredToken));

    let result = verify_institutional_identity(
        &MockDirectory::new(),
        &verifier,
        &MockRepositories::new(),
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await;

    assert!(matches!(result, Err(AuthenticationError::ExpiredToken)));
}

/// Verifica que falha ao obter o perfil impede a persistência da verificação.
///
/// Tenant, cadastro e grupo são consultados com sucesso. A obtenção do perfil
/// retorna `ProviderUnavailable`, propagado sem gravar a evidência.
#[tokio::test]
async fn graph_profile_failure_does_not_persist_anything() {
    let verifier = verifier_ok();
    let mut directory = directory_membership(true);

    directory
        .expect_get_profile()
        .with(eq(MICROSOFT_TOKEN), eq(true))
        .times(1)
        .returning(|_, _| Err(AuthenticationError::ProviderUnavailable));

    let result = verify_institutional_identity(
        &directory,
        &verifier,
        &repositories_institution(),
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await;

    assert!(matches!(
        result,
        Err(AuthenticationError::ProviderUnavailable)
    ));
}

/// Verifica que uma falha de tenant interrompe o fluxo antes de consultar o cadastro.
///
/// Espera `ProviderUnavailable` sem consultar instituição, grupo ou perfil e sem
/// gravar evidência. Uma falha de diretório não equivale a usuário não estudante.
#[tokio::test]
async fn tenant_lookup_failure_does_not_persist_anything() {
    let verifier = verifier_ok();
    let mut directory = MockDirectory::new();

    directory
        .expect_get_tenant_id()
        .times(1)
        .returning(|_| Err(AuthenticationError::ProviderUnavailable));

    let result = verify_institutional_identity(
        &directory,
        &verifier,
        &MockRepositories::new(),
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await;

    assert!(matches!(
        result,
        Err(AuthenticationError::ProviderUnavailable)
    ));
}

/// Verifica a conversão de falha do cadastro institucional em erro interno.
///
/// O repositório retorna `InstitutionRepositoryError::InternalServer`.
/// Espera `AuthenticationError::Internal`, sem interpretar a falha como
/// instituição ausente e sem consultar grupo ou perfil ou gravar evidência.
#[tokio::test]
async fn institution_repository_failure_is_not_unknown_institution() {
    let verifier = verifier_ok();
    let directory = directory_tenant_only();

    let mut repo = MockRepositories::new();

    repo.expect_find_by_microsoft_tenant_id()
        .times(1)
        .returning(|_| Err(InstitutionRepositoryError::InternalServer));

    let result = verify_institutional_identity(
        &directory,
        &verifier,
        &repo,
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await;

    assert!(matches!(result, Err(AuthenticationError::Internal)));
}

/// Verifica que uma consulta de grupo inconclusiva não produz gravação.
///
/// O diretório retorna `ProviderUnavailable`, propagado pelo serviço. O mock
/// não permite obter o perfil nem executar `save`. Uma falha não é convertida
/// em `NonStudent`; a preservação real de um registro exige teste do repositório.
#[tokio::test]
async fn inconclusive_membership_check_must_not_replace_saved_evidence() {
    let verifier = verifier_ok();
    let mut directory = directory_tenant_only();

    directory
        .expect_is_student_member()
        .with(eq(MICROSOFT_TOKEN), eq(student_group_id()))
        .times(1)
        .returning(|_, _| Err(AuthenticationError::ProviderUnavailable));

    let repo = repositories_institution(); // save NÃO está configurado: falharia se chamado
    let result = verify_institutional_identity(
        &directory,
        &verifier,
        &repo,
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await;

    assert!(matches!(
        result,
        Err(AuthenticationError::ProviderUnavailable)
    ));
}

/// Verifica que não estudantes também precisam de uma evidência persistida.
///
/// O grupo retorna `false`, mas `save` descarta a escrita por haver um registro
/// igual ou mais recente. Espera `InstitutionalVerificationRequired`, sem
/// confundir o descarte da escrita com uma proibição de acesso a não estudantes.
#[tokio::test]
async fn non_student_verification_requires_persisted_evidence() {
    let verifier = verifier_ok();
    let directory = directory_success(false);
    let mut repo = repositories_institution();

    repo.expect_save()
        .times(1)
        .withf(|v| !v.student_membership_confirmed)
        .returning(|_| Ok(false));

    let result = verify_institutional_identity(
        &directory,
        &verifier,
        &repo,
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await;

    assert!(matches!(
        result,
        Err(AuthenticationError::InstitutionalVerificationRequired)
    ));
}

/// Verifica que falha ao persistir uma confirmação positiva impede o sucesso.
///
/// Firebase e diretório aprovam a identidade e o vínculo, mas `save` retorna
/// `InstitutionalVerificationRepositoryError`. Espera `AuthenticationError::Internal`.
#[tokio::test]
async fn persistence_failure_never_returns_success() {
    let verifier = verifier_ok();
    let directory = directory_success(true);
    let mut repo = repositories_institution();

    repo.expect_save()
        .times(1)
        .returning(|_| Err(InstitutionalVerificationRepositoryError));

    let result = verify_institutional_identity(
        &directory,
        &verifier,
        &repo,
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await;

    assert!(matches!(result, Err(AuthenticationError::Internal)));
}

/// Verifica a recusa de requisição autenticada sem vínculo Microsoft no token.
///
/// Mesmo com Firebase validado, espera `InvalidMicrosoftId` antes de consultar
/// qualquer evidência persistida.
#[tokio::test]
async fn request_authentication_rejects_missing_linked_microsoft_id() {
    let mut verifier = MockVerifier::new();

    verifier.expect_verify().times(1).returning(|_| {
        Ok(VerifiedIdentity {
            firebase_uid: FIREBASE_UID.into(),
            microsoft_object_id: None,
        })
    });

    let result =
        authenticate_institutional_request(&verifier, &MockRepositories::new(), FIREBASE_TOKEN)
            .await;

    assert!(matches!(
        result,
        Err(AuthenticationError::InvalidMicrosoftId)
    ));
}

/// Verifica a propagação de token Firebase inválido ao autenticar uma requisição.
///
/// Espera `InvalidToken` e não permite chamadas ao repositório de evidências.
#[tokio::test]
async fn request_authentication_propagates_invalid_firebase_token() {
    let mut verifier = MockVerifier::new();

    verifier
        .expect_verify()
        .times(1)
        .returning(|_| Err(AuthenticationError::InvalidToken));

    let result =
        authenticate_institutional_request(&verifier, &MockRepositories::new(), FIREBASE_TOKEN)
            .await;

    assert!(matches!(result, Err(AuthenticationError::InvalidToken)));
}

/// Verifica a conversão de falha na leitura da evidência em erro interno.
///
/// Após validar a identidade, o repositório retorna
/// `InstitutionalVerificationRepositoryError`. Espera `AuthenticationError::Internal`,
/// sem confundir indisponibilidade da persistência com necessidade de novo login.
#[tokio::test]
async fn request_authentication_propagates_repository_failure() {
    let verifier = verifier_ok();
    let mut repo = MockRepositories::new();

    repo.expect_find_valid_by_firebase_uid()
        .times(1)
        .returning(|_| Err(InstitutionalVerificationRepositoryError));

    let result = authenticate_institutional_request(&verifier, &repo, FIREBASE_TOKEN).await;

    assert!(matches!(result, Err(AuthenticationError::Internal)));
}

/// Verifica que um estudante sem nome, e-mail ou curso pode concluir a verificação.
///
/// O grupo confirma o vínculo, que é repassado a `get_profile`. Exige a gravação
/// positiva e preserva a variante `Student` mesmo com `course_name: None`.
#[tokio::test]
async fn valid_profile_without_optional_fields_is_accepted() {
    let verifier = verifier_ok();
    let mut directory = MockDirectory::new();
    let mut repo = repositories_institution();

    directory
        .expect_get_profile()
        .with(eq(MICROSOFT_TOKEN), eq(true))
        .times(1)
        .returning(|_, _| {
            Ok(InstitutionalProfile {
                id: MICROSOFT_OBJECT_ID.to_owned(),
                name: None,
                mail: None,
                affiliation: InstitutionalAffiliation::Student { course_name: None },
            })
        });

    directory
        .expect_get_tenant_id()
        .with(eq(MICROSOFT_TOKEN))
        .times(1)
        .returning(|_| Ok(tenant_id()));

    directory
        .expect_is_student_member()
        .with(eq(MICROSOFT_TOKEN), eq(student_group_id()))
        .times(1)
        .returning(|_, _| Ok(true));

    repo.expect_save()
        .withf(|verification| verification.student_membership_confirmed)
        .times(1)
        .returning(|_| Ok(true));

    let result = verify_institutional_identity(
        &directory,
        &verifier,
        &repo,
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await
    .expect("atributos opcionais ausentes não devem impedir a verificação");

    assert_eq!(result.profile.id, MICROSOFT_OBJECT_ID);
    assert!(result.profile.name.is_none());
    assert!(result.profile.mail.is_none());
    assert!(matches!(
        result.profile.affiliation,
        InstitutionalAffiliation::Student { course_name: None }
    ));
}

/// Verifica que uma falha de persistência também impede a verificação de não estudantes.
///
/// O diretório retorna um perfil `NonStudent`, mas a gravação falha. Espera
/// `Internal`, pois permitir não estudantes não dispensa registrar a evidência.
#[tokio::test]
async fn non_student_persistence_failure_never_returns_success() {
    let verifier = verifier_ok();
    let directory = directory_success(false);
    let mut repo = repositories_institution();

    repo.expect_save()
        .withf(|verification| !verification.student_membership_confirmed)
        .times(1)
        .returning(|_| Err(InstitutionalVerificationRepositoryError));

    let result = verify_institutional_identity(
        &directory,
        &verifier,
        &repo,
        FIREBASE_TOKEN,
        MICROSOFT_TOKEN,
    )
    .await;

    assert!(matches!(result, Err(AuthenticationError::Internal)));
}

/// Verifica que um token expirado impede a consulta da evidência institucional.
///
/// Espera a propagação de `ExpiredToken`. O repositório sem expectativas
/// faz o teste falhar caso o serviço tente consultá-lo.
#[tokio::test]
async fn request_authentication_propagates_expired_firebase_token() {
    let mut verifier = MockVerifier::new();
    let repo = MockRepositories::new();

    verifier
        .expect_verify()
        .with(eq(FIREBASE_TOKEN))
        .times(1)
        .returning(|_| Err(AuthenticationError::ExpiredToken));

    let result = authenticate_institutional_request(&verifier, &repo, FIREBASE_TOKEN).await;

    assert!(matches!(result, Err(AuthenticationError::ExpiredToken)));
}

/// Verifica que um identificador Microsoft malformado impede a autenticação.
///
/// Firebase é validado, mas o identificador vinculado não é um UUID.
/// Espera `InvalidMicrosoftId` sem consultar evidências no repositório.
#[tokio::test]
async fn request_authentication_rejects_malformed_linked_microsoft_id() {
    let mut verifier = MockVerifier::new();
    let repo = MockRepositories::new();

    verifier
        .expect_verify()
        .with(eq(FIREBASE_TOKEN))
        .times(1)
        .returning(|_| {
            let mut identity = valid_identity();
            identity.microsoft_object_id = Some("not-a-uuid".to_owned());
            Ok(identity)
        });

    let result = authenticate_institutional_request(&verifier, &repo, FIREBASE_TOKEN).await;

    assert!(matches!(
        result,
        Err(AuthenticationError::InvalidMicrosoftId)
    ));
}
