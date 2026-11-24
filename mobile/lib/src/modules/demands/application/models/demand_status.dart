/// Estados e regras de transição do ciclo de vida das demandas.
library;

import 'demand_forum.dart';

/// Status de uma demanda no ciclo de vida do Centro Acadêmico.
enum DemandStatus {
  open('open', 'Aberta'),
  inAnalysis('in_analysis', 'Em análise'),
  forwardedToCoordination(
    'forwarded_to_coordination',
    'Enviada à coordenação',
  ),
  inExecution('in_execution', 'Em execução'),
  resolved('resolved', 'Resolvida'),
  completed('completed', 'Concluída'),
  rejected('rejected', 'Rejeitada'),
  cancelled('cancelled', 'Cancelada');

  const DemandStatus(this.wireValue, this.label);

  /// Valor canônico trafegado no contrato HTTP.
  final String wireValue;

  /// Rótulo legível em português para badges e filtros.
  final String label;

  /// Converte o valor textual da API para [DemandStatus].
  static DemandStatus fromWire(String value) {
    for (final status in DemandStatus.values) {
      if (status.wireValue == value) return status;
    }
    throw FormatException('Status de demanda desconhecido: $value');
  }

  /// Filtros de status exibidos na listagem para cada fórum.
  ///
  /// O status `forwardedToCoordination` é exclusivo do fórum institucional.
  static List<DemandStatus> filterOptionsFor(DemandForum forum) => switch (forum) {
    DemandForum.institutional => const [
      DemandStatus.open,
      DemandStatus.inAnalysis,
      DemandStatus.forwardedToCoordination,
      DemandStatus.completed,
    ],
    DemandForum.physicalSpace => const [
      DemandStatus.open,
      DemandStatus.inAnalysis,
      DemandStatus.completed,
    ],
  };

  /// Próximos status válidos pela regra de transição do fórum atual.
  static List<DemandStatus> allowedTransitions({
    required DemandForum forum,
    required DemandStatus from,
  }) => switch ((forum, from)) {
    (DemandForum.physicalSpace, DemandStatus.open) => const [
      DemandStatus.inAnalysis,
      DemandStatus.completed,
    ],
    (DemandForum.physicalSpace, DemandStatus.inAnalysis) => const [
      DemandStatus.completed,
      DemandStatus.open,
    ],
    (DemandForum.physicalSpace, DemandStatus.completed) => const [
      DemandStatus.open,
      DemandStatus.inAnalysis,
    ],
    (DemandForum.institutional, DemandStatus.open) => const [
      DemandStatus.inAnalysis,
      DemandStatus.forwardedToCoordination,
      DemandStatus.completed,
    ],
    (DemandForum.institutional, DemandStatus.inAnalysis) => const [
      DemandStatus.forwardedToCoordination,
      DemandStatus.completed,
      DemandStatus.open,
    ],
    (DemandForum.institutional, DemandStatus.forwardedToCoordination) => const [
      DemandStatus.inAnalysis,
      DemandStatus.completed,
      DemandStatus.open,
    ],
    (DemandForum.institutional, DemandStatus.completed) => const [
      DemandStatus.inAnalysis,
      DemandStatus.forwardedToCoordination,
    ],
    (DemandForum.physicalSpace, _) =>
      const [
        DemandStatus.open,
        DemandStatus.inAnalysis,
        DemandStatus.completed,
      ].where((s) => s != from).toList(),
    (DemandForum.institutional, _) =>
      const [
        DemandStatus.open,
        DemandStatus.inAnalysis,
        DemandStatus.forwardedToCoordination,
        DemandStatus.completed,
      ].where((s) => s != from).toList(),
  };
}
