/// API de aplicação do módulo de identidade.
///
/// Esta fachada define o que o restante do aplicativo pode consumir do módulo:
/// coordenação da sessão, estados observáveis, modelos e falhas. Ela concentra
/// o contrato público sem expor o caminho de cada biblioteca interna.
///
/// ## Uso pelos consumidores
///
/// A composição constrói SessionController e o entrega à interface. O consumidor
/// observa state e states e solicita operações pelo controlador, preservando
/// a coordenação de autenticação, verificação e cache.
///
/// Conhecer a conta autenticada e conhecer uma verificação institucional válida
/// são fatos diferentes. Os tipos exportados preservam essa distinção e publicam
/// SessionAccess para orientar a apresentação, sem conceder autorização no servidor.
///
/// ## Fronteira do módulo
///
/// As exportações selecionam apenas a camada application. SDKs, DTOs HTTP,
/// preferências, portas internas e InstitutionalAuthService não são reexportados.
/// AppDependencies importa implementações e contratos explicitamente para montar
/// o grafo; outros consumidores usam esta API em vez de depender dos adaptadores.
///
/// Os widgets são expostos separadamente por identity_widgets.dart. Essa separação
/// permite consumir a aplicação sem acrescentar uma dependência de Flutter à API.
library;

export 'application/session_controller.dart'
    show SessionController, SessionClock;
export 'application/session_state.dart'
    show
        SessionState,
        SessionRestoring,
        SessionSignedOut,
        SessionSignedIn,
        SessionRestorationFailed,
        SessionAccess,
        SessionOperation,
        SessionFailure,
        SessionFailureDetails;
export 'application/session_cache_result.dart'
    show
        SessionCacheLoadResult,
        SessionCacheFound,
        SessionCacheMissing,
        SessionCacheRejected,
        SessionCacheRejectionReason,
        SessionCacheReadFailed;
export 'application/models/institutional_affiliation.dart'
    show InstitutionalAffiliation, StudentAffiliation, NonStudentAffiliation;
export 'application/models/institutional_profile.dart'
    show InstitutionalProfile;
export 'application/models/institutional_verification.dart'
    show InstitutionalVerification;
export 'application/models/institutional_verification_result.dart'
    show InstitutionalVerificationResult;
export 'application/errors/identity_exception.dart'
    show IdentityException, IdentityFailureKind;
export 'application/errors/session_identity_changed_exception.dart'
    show SessionIdentityChangedException;
export 'application/errors/session_cache_write_exception.dart'
    show SessionCacheWriteException;
export 'application/errors/session_cache_removal_exception.dart'
    show SessionCacheRemovalException;
