/// Diagnósticos e exceções da camada de aplicação para consultas de eventos.
library;

/// Categorias semânticas de falha ao consultar a agenda ou detalhes de um evento.
enum EventQueryFailureKind {
  /// O evento solicitado não existe, não está publicado ou pertence a outro CA.
  notFound,

  /// O identificador do CA ativo ou parâmetro obrigatório está ausente/inválido.
  invalidRequest,

  /// Falha de comunicação, tempo esgotado ou indisponibilidade do servidor.
  unavailable,

  /// Erro inesperado ao interpretar ou processar a resposta.
  unexpected,
}

/// Exceção tipada lançada pelos casos de uso de visualização de eventos.
final class EventQueryException implements Exception {
  const EventQueryException({
    required this.kind,
    required this.message,
    this.cause,
  });

  final EventQueryFailureKind kind;
  final String message;
  final Object? cause;

  @override
  String toString() => 'EventQueryException($kind: $message)';
}
