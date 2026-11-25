/// Registro imutável de transição de status no histórico de uma demanda.
library;

import 'package:uuid/uuid.dart';

import 'demand_status.dart';

/// Entrada cronológica do histórico de alterações de status.
final class DemandStatusHistoryEntry {
  const DemandStatusHistoryEntry({
    required this.id,
    required this.demandId,
    required this.fromStatus,
    required this.toStatus,
    required this.changedById,
    required this.opinion,
    required this.createdAt,
  });

  final UuidValue id;
  final UuidValue demandId;
  final DemandStatus? fromStatus;
  final DemandStatus toStatus;
  final UuidValue changedById;
  final String? opinion;
  final DateTime createdAt;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is DemandStatusHistoryEntry &&
          other.id == id &&
          other.demandId == demandId &&
          other.fromStatus == fromStatus &&
          other.toStatus == toStatus &&
          other.changedById == changedById &&
          other.opinion == opinion &&
          other.createdAt == createdAt;

  @override
  int get hashCode => Object.hash(
    id,
    demandId,
    fromStatus,
    toStatus,
    changedById,
    opinion,
    createdAt,
  );
}
