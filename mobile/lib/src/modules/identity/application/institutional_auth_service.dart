/// Casos de uso de entrada e renovação da verificação institucional.
///
/// Esta biblioteca separa a autenticação Firebase da renovação institucional.
/// Na renovação, obter uma credencial Microsoft e solicitar a verificação ao
/// backend precisam referir-se à mesma conta. Obter
/// um token, inclusive silenciosamente, não comprova por si só o vínculo atual.
/// A coordenação depende de portas da aplicação, mantendo o caso de uso utilizável
/// sem SDK Firebase, Dio, JSON ou widgets.
///
/// ## Fluxo de uma verificação
///
/// No login, MicrosoftSignIn estabelece a autenticação e AuthSession informa
/// o UID. O controlador consulta a conta e só solicita renovação se não houver
/// confirmação institucional válida. Na renovação, o UID é capturado antes de solicitar a
/// credencial da conta atual, para detectar mudanças durante essa obtenção.
/// A operação pode concluir sem interação; allowInteraction: false impede
/// a abertura do provedor quando iniciada automaticamente pelo controlador.
///
/// O serviço confere a identidade antes da chamada ao InstitutionalIdentityGateway
/// e depois de sua conclusão. A conta e a evidência são envolvidas em
/// AccountSessionResult junto ao UID correspondente, pois o contrato
/// HTTP não inclui esse identificador Firebase.
/// Essa conferência local protege a associação do resultado à sessão; a
/// correspondência entre o token Microsoft e a identidade vinculada ao Firebase
/// é validada pelo backend, e não pelo UID informado no mobile.
///
/// ## Relação com o controlador
///
/// SessionController chama o caso de uso e possui as decisões de prontidão,
/// prazo, concorrência e persistência. Este serviço não publica estado nem grava
/// cache. Sua conferência de UID complementa as revisões do controlador; sozinha
/// não detectaria todas as trocas de contexto que retornassem ao mesmo UID.
///
/// Ausência ou troca da conta sinaliza SessionIdentityChangedException.
/// As falhas das portas propagam com o diagnóstico, sem virar resposta nula.
/// Todas as implementações injetadas devem compartilhar a mesma autenticação.
library;

import 'package:capp/src/modules/identity/application/ports/institutional_identity_gateway.dart';

import 'errors/session_identity_changed_exception.dart';
import 'errors/identity_exception.dart';
import 'ports/auth_session.dart';
import 'models/account_session_result.dart';
import 'ports/microsoft_sign_in.dart';

/// Executa autenticação e verificação para uma mesma identidade.
///
/// As implementações recebidas devem operar sobre a mesma autenticação.
/// O serviço confere o UID antes e depois da verificação externa.
///
/// Persistência, prontidão da sessão e concorrência entre operações
/// pertencem ao controlador.
final class InstitutionalAuthService {
  /// Conecta os contratos necessários ao fluxo de identidade.
  ///
  /// identityGateway verifica a identidade institucional.
  /// microsoftSignIn obtém a credencial delegada.
  /// authSession permite conferir a conta autenticada.
  ///
  /// A construção não executa operações externas.
  const InstitutionalAuthService({
    required this._identityGateway,
    required this._microsoftSignIn,
    required this._authSession,
  });

  final InstitutionalIdentityGateway _identityGateway;
  final MicrosoftSignIn _microsoftSignIn;
  final AuthSession _authSession;

  /// Estabelece a identidade Firebase para a entrada Microsoft.
  ///
  /// SessionController restaura a conta autenticada antes de decidir se precisa
  /// de uma verificação. O retorno exige um UID, mas não solicita token MSAL,
  /// consulta o Graph ou determina se o perfil interno está completo.
  Future<void> signIn() async {
    await _microsoftSignIn.signIn();
    _requireCurrentUserId();
  }

  /// Renova a verificação institucional para a conta autenticada.
  ///
  /// Tenta obter a credencial Microsoft silenciosamente.
  /// [allowInteraction] permite apresentar autorização Microsoft
  /// quando a operação foi iniciada pelo usuário.
  ///
  /// Com interação desabilitada, a necessidade de participação
  /// do usuário é propagada ao chamador.
  ///
  /// Confere a permanência do UID antes e depois da verificação.
  /// Retorna o UID, o perfil interno e a nova evidência; não grava cache nem altera o estado
  /// observável. Ausência ou mudança de conta lança
  /// [SessionIdentityChangedException]. Falhas das portas propagam.
  ///
  /// Por exemplo, uma coordenação que não admite autorização interativa usa:
  /// ```dart
  /// final result = await service.renewVerification(allowInteraction: false);
  /// ```
  /// O chamador deve tratar a necessidade de interação e preservar o prazo da
  /// evidência anterior se a operação falhar.
  Future<AccountSessionResult> renewVerification({
    bool allowInteraction = true,
  }) async {
    final firebaseUid = _requireCurrentUserId();

    final accessToken = await _microsoftSignIn.getAccessTokenForCurrentUser(
      allowInteraction: allowInteraction,
    );

    return _verify(firebaseUid: firebaseUid, microsoftAccessToken: accessToken);
  }

  /// Executa a verificação entre duas conferências da identidade esperada.
  ///
  /// Não avalia prazo, grava cache ou publica estado de interface.
  /// Uma mudança de identidade não desfaz efeitos já concluídos externamente.
  ///
  /// A revisão do controlador continua necessária para detectar contextos
  /// obsoletos mesmo quando o UID volta a ser o mesmo.
  Future<AccountSessionResult> _verify({
    required String firebaseUid,
    required String microsoftAccessToken,
  }) async {
    _requireSameUser(firebaseUid);

    final session = await _identityGateway.verify(
      microsoftAccessToken: microsoftAccessToken,
    );

    _requireSameUser(firebaseUid);

    final verification = session.verification;

    if (verification == null) {
      throw IdentityException(
        kind: IdentityFailureKind.invalidResponse,
        cause: const FormatException('Verificação institucional ausente'),
        stackTrace: StackTrace.current,
      );
    }

    return AccountSessionResult(
      firebaseUid: firebaseUid,
      user: session.user,
      verification: verification,
    );
  }

  /// Obtém o identificador atual ou sinaliza ausência de identidade.
  String _requireCurrentUserId() {
    final firebaseUid = _authSession.currentUserId;

    if (firebaseUid == null) {
      throw const SessionIdentityChangedException();
    }

    return firebaseUid;
  }

  /// Exige que a autenticação continue pertencendo a [expectedFirebaseUid].
  void _requireSameUser(String expectedFirebaseUid) {
    if (_authSession.currentUserId != expectedFirebaseUid) {
      throw const SessionIdentityChangedException();
    }
  }
}
