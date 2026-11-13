/// Resultado da entrada ou renovação associado à identidade autenticada.
///
/// A resposta HTTP contém a conta provisionada e sua evidência, sem UID Firebase.
/// Esta biblioteca fecha essa diferença no contrato de aplicação, juntando a
/// verificação ao identificador observado durante o caso de uso.
///
/// ## Percurso do resultado
///
/// SessionController reutiliza a confirmação vigente da conta ou solicita
/// renovação a InstitutionalAuthService. Ambos associam a resposta ao UID
/// observado, sem estender a validade de uma confirmação reutilizada. O
/// controlador confere o contexto e o prazo antes de publicar o resultado
/// e persistir sob a chave da conta correspondente.
///
/// ## Alcance da associação
///
/// O envelope não representa uma nova sessão de backend e não contém tokens.
/// Ele também não substitui as guardas de revisão: uma operação antiga pode ter
/// o mesmo UID de uma sessão posterior. Essa distinção pertence ao controlador.
///
/// A biblioteca contém apenas um modelo imutável da aplicação, independente
/// de Firebase, HTTP e preferências.
library;

import 'institutional_verification.dart';
import 'internal_user.dart';

/// Conta, confirmação institucional e UID usados durante a operação.
///
/// O UID vem da autenticação local. A resposta institucional
/// continua sendo produzida e validada pelo backend.
/// A associação por UID não substitui o contexto de execução: uma sequência
/// A/B/A também precisa ser identificada pela revisão local do controlador.
final class AccountSessionResult {
  /// Agrupa a identidade local esperada e a resposta recebida.
  ///
  /// Não realiza I/O, não verifica expiração e não persiste o resultado.
  const AccountSessionResult({
    required this.firebaseUid,
    required this.verification,
    required this.user,
  });

  /// Usuário ao qual pertence a resposta.
  final String firebaseUid;

  /// Conta provisionada retornada pela restauração ou renovação.
  final InternalUser user;

  /// Perfil, vínculo e validade retornados pelo backend.
  final InstitutionalVerification verification;
}
