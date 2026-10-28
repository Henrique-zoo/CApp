/// Resultado confirmado acompanhado de falha exclusiva de persistência local.
///
/// A verificação remota pode permitir entrada mesmo quando o dispositivo não
/// consegue salvar seu resultado. Esta biblioteca torna esse sucesso parcial
/// explícito, evitando que uma falha de preferências pareça recusa de autenticação.
///
/// ## Lugar no fluxo
///
/// SessionController publica a verificação em memória e tenta gravá-la pelo
/// SessionStore. Se a gravação lançar uma Exception, o controlador conserva
/// o destino pronto, publica o aviso e lança SessionCacheWriteException.
///
/// A exceção transporta o resultado confirmado, a causa da gravação e sua pilha.
/// O chamador consegue distinguir essa situação de uma falha do provedor ou
/// backend sem consultar tipos de preferências.
///
/// ## Alcance
///
/// A entrada na execução atual permanece permitida enquanto a evidência for
/// válida. A persistência para a próxima abertura não está garantida.
/// Este tipo não repete a gravação, remove registros ou executa logout; apenas
/// representa o desfecho observado pelo controlador.
library;

import '../models/institutional_verification_result.dart';

/// A verificação concluiu, mas a persistência local falhou.
///
/// O resultado continua disponível em memória. A falha não representa recusa
/// do backend; a restauração futura pode exigir outra entrada.
final class SessionCacheWriteException implements Exception {
  /// Preserva o resultado confirmado e o diagnóstico original da gravação.
  ///
  /// A causa pode conter dados do perfil e não é uma mensagem pública.
  const SessionCacheWriteException({
    required this.result,
    required this.cause,
    required this.causeStackTrace,
  });

  /// Identidade e verificação confirmadas antes da tentativa de gravação.
  final InstitutionalVerificationResult result;

  /// Exceção original produzida pelo armazenamento.
  final Exception cause;

  /// Pilha original capturada na falha.
  final StackTrace causeStackTrace;

  /// Descreve a situação sem serializar resultado ou diagnóstico.
  @override
  String toString() =>
      'A verificação foi concluída, mas não pôde ser persistida';
}
