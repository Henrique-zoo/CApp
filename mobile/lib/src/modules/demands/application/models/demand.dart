/// Entidade de aplicação que representa uma demanda em um Centro Acadêmico.
library;

import 'package:uuid/uuid.dart';

import 'demand_forum.dart';
import 'demand_status.dart';
import 'demand_status_history_entry.dart';

/// Demanda registrada no fórum institucional ou de espaço físico do CA.
final class Demand {
  const Demand({
    required this.id,
    required this.academicCenterId,
    required this.authorId,
    required this.forum,
    required this.title,
    required this.description,
    required this.category,
    required this.courseName,
    required this.status,
    required this.createdAt,
    required this.updatedAt,
    this.statusHistory = const [],
    this.commentCount,
  });

  final UuidValue id;
  final UuidValue academicCenterId;
  final UuidValue authorId;
  final DemandForum forum;
  final String? title;
  final String description;
  final String? category;
  final String? courseName;
  final DemandStatus status;
  final DateTime createdAt;
  final DateTime updatedAt;
  final List<DemandStatusHistoryEntry> statusHistory;
  final int? commentCount;

  /// Retorna uma cópia preservando os dados não alterados.
  Demand copyWith({
    DemandStatus? status,
    DateTime? updatedAt,
    List<DemandStatusHistoryEntry>? statusHistory,
    int? commentCount,
  }) => Demand(
    id: id,
    academicCenterId: academicCenterId,
    authorId: authorId,
    forum: forum,
    title: title,
    description: description,
    category: category,
    courseName: courseName,
    status: status ?? this.status,
    createdAt: createdAt,
    updatedAt: updatedAt ?? this.updatedAt,
    statusHistory: statusHistory ?? this.statusHistory,
    commentCount: commentCount ?? this.commentCount,
  );

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Demand &&
          other.id == id &&
          other.academicCenterId == academicCenterId &&
          other.authorId == authorId &&
          other.forum == forum &&
          other.title == title &&
          other.description == description &&
          other.category == category &&
          other.courseName == courseName &&
          other.status == status &&
          other.createdAt == createdAt &&
          other.updatedAt == updatedAt &&
          other.commentCount == commentCount;

  @override
  int get hashCode => Object.hash(
    id,
    academicCenterId,
    authorId,
    forum,
    title,
    description,
    category,
    courseName,
    status,
    createdAt,
    updatedAt,
    commentCount,
  );
}
