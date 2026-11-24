/// Resultado do recálculo eleitoral e alcance das evidências externas.
library;

/// Desfecho recalculado a partir das contribuições disponíveis.
enum AuditedTally { incomplete, valid, annulled, cancelled }

/// Contagens e alcance da verificação de uma eleição confidencial.
final class ElectionAudit {
  const ElectionAudit({
    required this.tally,
    required this.totals,
    required this.optionNames,
    required this.anchoredThrough,
    required this.finalClosureAnchored,
    this.cancellationReason,
  });

  final AuditedTally tally;

  /// Contagens na ordem do manifesto, com Nulo na última posição.
  final List<int>? totals;

  /// Rótulos verificados, na mesma ordem das contagens.
  final List<String> optionNames;

  /// Posição coberta pelo recibo independente fornecido à auditoria.
  final int anchoredThrough;

  /// Há um termo final obtido externamente que coincide com o quadro.
  final bool finalClosureAnchored;

  /// Motivo declarado pela comissão no cancelamento, quando disponível.
  final String? cancellationReason;
}

/// Solicita verificação a partir da rede, da cópia local ou de um pacote inserido.
typedef RunElectionAudit = Future<ElectionAudit> Function(
  bool download, {
  String? bundleJson,
  String? anchorJson,
});
