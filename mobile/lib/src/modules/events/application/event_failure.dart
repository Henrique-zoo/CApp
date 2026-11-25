/// Diagnósticos e exceções da camada de aplicação para consultas e inscrições em eventos.
library;

/// Categorias semânticas de falha ao consultar a agenda, detalhes ou realizar inscrição/cancelamento.
enum EventQueryFailureKind {
  /// O evento solicitado não existe, não está publicado ou pertence a outro CA.
  notFound,

  /// O identificador do CA ativo ou parâmetro obrigatório está ausente/inválido.
  invalidRequest,

  /// O evento atingiu o limite máximo de participantes (`409 event_full`).
  eventFull,

  /// O usuário já possui inscrição ativa no evento (`409 already_registered`).
  alreadyRegistered,

  /// Não há inscrição ativa para cancelar (`409 no_active_registration`).
  noActiveRegistration,

  /// O evento não está disponível para inscrição (`422 event_not_available`).
  eventNotAvailable,

  /// O prazo de inscrição para o evento já foi encerrado.
  deadlinePassed,

  /// Falha de comunicação, tempo esgotado ou indisponibilidade do servidor.
  unavailable,

  /// Erro inesperado ao interpretar ou processar a resposta.
  unexpected,
}

/// Exceção tipada lançada pelos casos de uso de visualização e participação em eventos.
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
