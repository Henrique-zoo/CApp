/// Caso de uso de autenticação Microsoft e verificação institucional.
///
/// Esta biblioteca reúne duas integrações que precisam referir-se à mesma conta:
/// obter uma credencial Microsoft e solicitar a verificação ao backend.
/// A coordenação depende de portas da aplicação, mantendo o caso de uso utilizável
/// sem SDK Firebase, Dio, JSON ou widgets.
///
/// ## Fluxo de uma verificação
///
/// No primeiro login, MicrosoftSignIn obtém o access token e AuthSession informa
/// o UID estabelecido. Na reautenticação, o UID é capturado antes de abrir o
/// provedor, de modo que outra conta não seja aceita como renovação.
///
/// O serviço confere a identidade antes da chamada ao InstitutionalIdentityGateway
/// e depois de sua conclusão. A resposta é envolvida em
/// InstitutionalVerificationResult junto ao UID correspondente, pois o contrato
/// HTTP não inclui esse identificador Firebase.
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
import 'ports/auth_session.dart';
import 'models/institutional_verification_result.dart';
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

  /// Autentica com Microsoft e verifica a identidade institucional.
  ///
  /// Captura o UID estabelecido pela autenticação e confere sua permanência
  /// durante a verificação externa.
  ///
  /// Retorna a verificação associada à conta correspondente.
  /// Falhas dos contratos recebidos propagam sem conversão em resultado nulo.
  ///
  /// Lança [SessionIdentityChangedException] se não houver identidade
  /// após autenticar ou se ela mudar durante a verificação.
  Future<InstitutionalVerificationResult> signInAndVerify() async {
    final accessToken = await _microsoftSignIn.signInAndGetAccessToken();
    final firebaseUid = _requireCurrentUserId();

    return _verify(firebaseUid: firebaseUid, microsoftAccessToken: accessToken);
  }

  /// Reautentica a conta atual e obtém uma nova verificação institucional.
  ///
  /// Captura o UID antes da interação Microsoft, impedindo que outra conta
  /// seja aceita como renovação da identidade anterior.
  ///
  /// Reautenticar não prorroga a evidência por si só: a verificação externa
  /// também precisa concluir.
  ///
  /// Falhas propagam com seus diagnósticos. A ausência ou mudança da conta
  /// lança [SessionIdentityChangedException].
  Future<InstitutionalVerificationResult> reauthenticateAndVerify() async {
    final firebaseUid = _requireCurrentUserId();

    final accessToken = await _microsoftSignIn
        .reauthenticateAndGetAccessToken();

    return _verify(firebaseUid: firebaseUid, microsoftAccessToken: accessToken);
  }

  /// Executa a verificação entre duas conferências da identidade esperada.
  ///
  /// Não avalia prazo, grava cache ou publica estado de interface.
  /// Uma mudança de identidade não desfaz efeitos já concluídos externamente.
  ///
  /// A revisão do controlador continua necessária para detectar contextos
  /// obsoletos mesmo quando o UID volta a ser o mesmo.
  Future<InstitutionalVerificationResult> _verify({
    required String firebaseUid,
    required String microsoftAccessToken,
  }) async {
    _requireSameUser(firebaseUid);

    final verification = await _identityGateway.verify(
      microsoftAccessToken: microsoftAccessToken,
    );

    _requireSameUser(firebaseUid);

    return InstitutionalVerificationResult(
      firebaseUid: firebaseUid,
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
