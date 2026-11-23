/// Sinaliza que a autorização Microsoft exige participação do usuário.
///
/// A renovação automática precisa distinguir uma falha de comunicação de uma
/// credencial que exige consentimento ou autenticação. Esta biblioteca fornece
/// esse sinal no vocabulário da aplicação, sem propagar tipos MSAL como contrato.
///
/// O adaptador converte ausência de conta ou necessidade de interação para esta
/// exceção. FirebaseMicrosoftAuth pode iniciar autorização interativa quando a
/// operação permitir; caso contrário, a falha chega ao controlador e orienta a
/// mensagem da apresentação. A causa técnica permanece apenas como diagnóstico.
///
/// Pode indicar consentimento pendente, MFA ou uma credencial que já não pode
/// ser renovada silenciosamente. Não representa logout do Firebase e não altera
/// o vencimento da evidência institucional anterior.
library;

/// Necessidade de uma autorização Microsoft interativa.
final class MicrosoftInteractionRequiredException implements Exception {
  /// Preserva o diagnóstico e a pilha da tentativa que exigiu interação.
  const MicrosoftInteractionRequiredException({
    required this.cause,
    required this.stackTrace,
  });

  /// Diagnóstico original; não deve ser apresentado ou registrado integralmente.
  final Object cause;

  /// Pilha original da falha traduzida na fronteira do adaptador.
  final StackTrace stackTrace;

  /// Identifica a categoria sem expor mensagens ou credenciais do provedor.
  @override
  String toString() => 'MicrosoftInteractionRequiredException';
}
