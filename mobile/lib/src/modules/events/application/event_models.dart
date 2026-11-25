/// Modelos da camada de aplicação para visualização e detalhes de eventos publicados.
library;

import 'package:uuid/uuid.dart';

/// Estado de publicação de um evento no Centro Acadêmico.
enum EventPublicationStatus {
  draft,
  pendingApproval,
  approved,
  rejected,
  published,
  cancelled;

  /// Interpreta a representação textual canônica do backend (`snake_case` ou português).
  static EventPublicationStatus fromWire(String raw) {
    switch (raw.trim().toLowerCase()) {
      case 'draft':
      case 'rascunho':
        return EventPublicationStatus.draft;
      case 'pending_approval':
      case 'pendente':
        return EventPublicationStatus.pendingApproval;
      case 'approved':
      case 'aprovado':
        return EventPublicationStatus.approved;
      case 'rejected':
      case 'rejeitado':
        return EventPublicationStatus.rejected;
      case 'published':
      case 'publicado':
        return EventPublicationStatus.published;
      case 'cancelled':
      case 'canceled':
      case 'cancelado':
        return EventPublicationStatus.cancelled;
      default:
        throw FormatException('Status de evento desconhecido: $raw');
    }
  }

  /// Indica se o evento é visível na agenda pública do Centro Acadêmico.
  bool get isVisibleInSchedule => this == EventPublicationStatus.published;
}

/// Identificação do Centro Acadêmico ativo na consulta da agenda de eventos.
final class AcademicCenterContext {
  const AcademicCenterContext({
    required this.id,
    required this.name,
    this.acronym,
  });

  final UuidValue id;
  final String name;
  final String? acronym;

  /// Rótulo amigável combinando nome e sigla quando disponível.
  String get displayLabel {
    final trimmedAcronym = acronym?.trim();
    if (trimmedAcronym == null || trimmedAcronym.isEmpty) {
      return name;
    }
    return '$name ($trimmedAcronym)';
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is AcademicCenterContext &&
          other.id == id &&
          other.name == name &&
          other.acronym == acronym;

  @override
  int get hashCode => Object.hash(id, name, acronym);
}

/// Representação de um evento publicado na agenda oficial do Centro Acadêmico.
final class PublishedEvent {
  const PublishedEvent({
    required this.id,
    required this.academicCenterId,
    required this.authorId,
    required this.title,
    required this.description,
    required this.location,
    required this.startsAt,
    required this.endsAt,
    required this.status,
    required this.requiresRegistration,
    required this.maxCapacity,
    required this.confirmedRegistrations,
    required this.remainingSpots,
    required this.hasAvailableSpots,
    required this.registrationDeadline,
    required this.publishedAt,
  });

  final UuidValue id;
  final UuidValue academicCenterId;
  final UuidValue authorId;
  final String title;
  final String description;
  final String location;
  final DateTime startsAt;
  final DateTime endsAt;
  final EventPublicationStatus status;
  final bool requiresRegistration;
  final int? maxCapacity;
  final int confirmedRegistrations;
  final int? remainingSpots;
  final bool hasAvailableSpots;
  final DateTime? registrationDeadline;
  final DateTime? publishedAt;

  /// Indica se o evento está efetivamente publicado.
  bool get isPublished => status.isVisibleInSchedule;
}
