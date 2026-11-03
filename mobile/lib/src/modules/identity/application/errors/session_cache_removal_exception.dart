/// Falha de limpeza local depois do encerramento da autenticação.
///
/// O logout do provedor e a remoção do perfil persistido são operações distintas.
/// Esta biblioteca representa o sucesso da primeira com falha da segunda, sem
/// restaurar artificialmente uma autenticação que já foi encerrada.
///
/// SessionController publica a falha na entrada e propaga esta exceção. A causa
/// e a pilha do armazenamento permanecem disponíveis para diagnóstico; o cache
/// remanescente não permite entrada sem uma conta autenticada.
library;

/// A conta saiu do provedor, mas seu registro local não pôde ser removido.
///
/// Não revoga tokens de outros dispositivos ou desfaz efeitos no backend.
final class SessionCacheRemovalException implements Exception {
  /// Preserva a conta afetada e o diagnóstico da tentativa de remoção.
  ///
  /// [firebaseUid] identifica somente o registro cuja limpeza foi solicitada.
  /// [cause] e [causeStackTrace] conservam a falha original do armazenamento.
  const SessionCacheRemovalException({
    required this.firebaseUid,
    required this.cause,
    required this.causeStackTrace,
  });

  /// Conta cujo registro permaneceu após a tentativa de remoção.
  final String firebaseUid;

  /// Exceção original da remoção, sem interpretação de sua mensagem técnica.
  final Exception cause;

  /// Pilha original da falha de acesso ao armazenamento.
  final StackTrace causeStackTrace;

  /// Descreve o sucesso parcial sem incluir identificadores ou dados pessoais.
  @override
  String toString() =>
      'A autenticação foi encerrada, mas o cache local não pôde ser removido';
}
