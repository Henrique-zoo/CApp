/// Associação interna entre uma verificação institucional e a conta solicitante.
///
/// A resposta HTTP contém perfil e validade, mas não devolve o UID Firebase.
/// Esta biblioteca fecha essa diferença no contrato de aplicação, juntando a
/// verificação ao identificador observado durante o caso de uso.
///
/// ## Percurso do resultado
///
/// InstitutionalAuthService captura o UID, verifica sua permanência ao redor da
/// chamada ao gateway e constrói InstitutionalVerificationResult.
/// SessionController recebe esse resultado, confere seu contexto e avalia o prazo
/// antes de publicá-lo e persistir sob a chave da conta correspondente.
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

/// Resposta da verificação e o UID usado durante a operação.
///
/// O UID vem da autenticação local. A resposta institucional
/// continua sendo produzida e validada pelo backend.
/// A associação por UID não substitui o contexto de execução: uma sequência
/// A/B/A também precisa ser identificada pela revisão local do controlador.
final class InstitutionalVerificationResult {
  /// Agrupa a identidade local esperada e a resposta recebida.
  ///
  /// Não realiza I/O, não verifica expiração e não persiste o resultado.
  const InstitutionalVerificationResult({
    required this.firebaseUid,
    required this.verification,
  });

  /// Usuário ao qual pertence a resposta.
  final String firebaseUid;

  /// Perfil, vínculo e validade retornados pelo backend.
  final InstitutionalVerification verification;
}
