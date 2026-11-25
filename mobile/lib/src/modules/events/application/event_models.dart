/// Modelos da camada de aplicação para visualização, detalhes, vagas e inscrição em eventos.
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

/// Estado de uma inscrição de participante em um evento.
enum EventRegistrationStatus {
  confirmed,
  cancelled,
  waitlisted;

  /// Interpreta a representação textual do backend (`confirmed`, `cancelled`, `waitlisted`).
  static EventRegistrationStatus fromWire(String raw) {
    switch (raw.trim().toLowerCase()) {
      case 'confirmed':
      case 'confirmada':
        return EventRegistrationStatus.confirmed;
      case 'cancelled':
      case 'canceled':
      case 'cancelada':
        return EventRegistrationStatus.cancelled;
      case 'waitlisted':
      case 'fila_espera':
        return EventRegistrationStatus.waitlisted;
      default:
        throw FormatException('Status de inscrição desconhecido: $raw');
    }
  }

  /// Indica se a inscrição está ativa/confirmada para o participante.
  bool get isActive => this == EventRegistrationStatus.confirmed;
}

/// Registro retornado ao realizar ou cancelar uma inscrição em evento.
final class EventRegistrationRecord {
  const EventRegistrationRecord({
    required this.id,
    required this.eventId,
    required this.participantId,
    required this.status,
    required this.registeredAt,
    required this.cancelledAt,
    required this.message,
  });

  final UuidValue id;
  final UuidValue eventId;
  final UuidValue participantId;
  final EventRegistrationStatus status;
  final DateTime registeredAt;
  final DateTime? cancelledAt;
  final String message;

  /// Indica se o registro representa uma inscrição ativa.
  bool get isActive => status.isActive;
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

  /// Calcula as vagas restantes efetivas quando o evento possui limite configurado.
  int? get effectiveRemainingSpots {
    final cap = maxCapacity;
    if (cap == null) {
      return null;
    }
    final remaining = remainingSpots ?? (cap - confirmedRegistrations);
    return remaining.clamp(0, cap);
  }

  /// Indica se o evento atingiu a capacidade máxima de participantes.
  bool get isFull {
    if (maxCapacity == null) {
      return false;
    }
    final remaining = effectiveRemainingSpots ?? 0;
    return !hasAvailableSpots || remaining <= 0;
  }

  /// Verifica se o prazo de inscrição já expirou em relação a [referenceTime] (ou `DateTime.now().toUtc()`).
  bool isRegistrationDeadlinePassed({DateTime? referenceTime}) {
    final deadline = registrationDeadline;
    if (deadline == null) {
      return false;
    }
    final now = (referenceTime ?? DateTime.now()).toUtc();
    return now.isAfter(deadline.toUtc());
  }

  /// Retorna uma cópia do evento com contadores de inscrição e vagas atualizados.
  PublishedEvent copyWith({
    int? confirmedRegistrations,
    int? remainingSpots,
    bool? hasAvailableSpots,
  }) {
    final nextConfirmed =
        (confirmedRegistrations ?? this.confirmedRegistrations).clamp(
          0,
          1 << 30,
        );
    final cap = maxCapacity;
    final nextRemaining = cap == null
        ? null
        : (remainingSpots ?? (cap - nextConfirmed)).clamp(0, cap);
    final nextHasSpots =
        hasAvailableSpots ?? (cap == null || (nextRemaining ?? 0) > 0);

    return PublishedEvent(
      id: id,
      academicCenterId: academicCenterId,
      authorId: authorId,
      title: title,
      description: description,
      location: location,
      startsAt: startsAt,
      endsAt: endsAt,
      status: status,
      requiresRegistration: requiresRegistration,
      maxCapacity: maxCapacity,
      confirmedRegistrations: nextConfirmed,
      remainingSpots: nextRemaining,
      hasAvailableSpots: nextHasSpots,
      registrationDeadline: registrationDeadline,
      publishedAt: publishedAt,
    );
  }
}
