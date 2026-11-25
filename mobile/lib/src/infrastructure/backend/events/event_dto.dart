/// Contratos JSON de transporte para listagem, detalhes e inscrições em eventos do backend.
///
/// Espelha exatamente `EventResponse` / `EventDetailResponse`, `EventRegistrationResponse`
/// e `EventErrorResponse` definidos no módulo `modules::events::api::dto` do servidor Rust.
library;

import 'package:uuid/uuid.dart';

import '../converters/uuid_value_converter.dart';

const _uuidConverter = UuidValueConverter();

/// DTO de resposta de um evento retornado pelos endpoints `/api/v1/events`
/// e `/api/v1/academic-centers/{ca_id}/events`.
final class EventDto {
  const EventDto({
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
    required this.createdAt,
    required this.updatedAt,
  });

  final UuidValue id;
  final UuidValue academicCenterId;
  final UuidValue authorId;
  final String title;
  final String description;
  final String location;
  final DateTime startsAt;
  final DateTime endsAt;
  final String status;
  final bool requiresRegistration;
  final int? maxCapacity;
  final int confirmedRegistrations;
  final int? remainingSpots;
  final bool hasAvailableSpots;
  final DateTime? registrationDeadline;
  final DateTime? publishedAt;
  final DateTime createdAt;
  final DateTime updatedAt;

  /// Desserializa um objeto JSON de evento validando tipos e formatos obrigatórios.
  factory EventDto.fromJson(Map<String, dynamic> json) {
    final idRaw = json['id'];
    final caIdRaw = json['academic_center_id'];
    final authorIdRaw = json['author_id'];
    final title = json['title'];
    final description = json['description'];
    final location = json['location'];
    final startsAtRaw = json['starts_at'];
    final endsAtRaw = json['ends_at'];
    final status = json['status'];
    final requiresRegistration = json['requires_registration'];
    final maxCapacity = json['max_capacity'];
    final confirmedRegistrations = json['confirmed_registrations'];
    final remainingSpots = json['remaining_spots'];
    final hasAvailableSpots = json['has_available_spots'];
    final registrationDeadlineRaw = json['registration_deadline'];
    final publishedAtRaw = json['published_at'];
    final createdAtRaw = json['created_at'];
    final updatedAtRaw = json['updated_at'];

    if (idRaw is! String ||
        caIdRaw is! String ||
        authorIdRaw is! String ||
        title is! String ||
        description is! String ||
        location is! String ||
        startsAtRaw is! String ||
        endsAtRaw is! String ||
        status is! String ||
        requiresRegistration is! bool ||
        (maxCapacity != null && maxCapacity is! int) ||
        confirmedRegistrations is! int ||
        (remainingSpots != null && remainingSpots is! int) ||
        hasAvailableSpots is! bool ||
        (registrationDeadlineRaw != null &&
            registrationDeadlineRaw is! String) ||
        (publishedAtRaw != null && publishedAtRaw is! String) ||
        createdAtRaw is! String ||
        updatedAtRaw is! String) {
      throw const FormatException('Contrato JSON de evento inválido.');
    }

    return EventDto(
      id: _uuidConverter.fromJson(idRaw),
      academicCenterId: _uuidConverter.fromJson(caIdRaw),
      authorId: _uuidConverter.fromJson(authorIdRaw),
      title: title,
      description: description,
      location: location,
      startsAt: DateTime.parse(startsAtRaw).toUtc(),
      endsAt: DateTime.parse(endsAtRaw).toUtc(),
      status: status,
      requiresRegistration: requiresRegistration,
      maxCapacity: maxCapacity as int?,
      confirmedRegistrations: confirmedRegistrations,
      remainingSpots: remainingSpots as int?,
      hasAvailableSpots: hasAvailableSpots,
      registrationDeadline: registrationDeadlineRaw == null
          ? null
          : DateTime.parse(registrationDeadlineRaw as String).toUtc(),
      publishedAt: publishedAtRaw == null
          ? null
          : DateTime.parse(publishedAtRaw as String).toUtc(),
      createdAt: DateTime.parse(createdAtRaw).toUtc(),
      updatedAt: DateTime.parse(updatedAtRaw).toUtc(),
    );
  }

  Map<String, dynamic> toJson() => {
    'id': _uuidConverter.toJson(id),
    'academic_center_id': _uuidConverter.toJson(academicCenterId),
    'author_id': _uuidConverter.toJson(authorId),
    'title': title,
    'description': description,
    'location': location,
    'starts_at': startsAt.toUtc().toIso8601String(),
    'ends_at': endsAt.toUtc().toIso8601String(),
    'status': status,
    'requires_registration': requiresRegistration,
    'max_capacity': maxCapacity,
    'confirmed_registrations': confirmedRegistrations,
    'remaining_spots': remainingSpots,
    'has_available_spots': hasAvailableSpots,
    'registration_deadline': registrationDeadline?.toUtc().toIso8601String(),
    'published_at': publishedAt?.toUtc().toIso8601String(),
    'created_at': createdAt.toUtc().toIso8601String(),
    'updated_at': updatedAt.toUtc().toIso8601String(),
  };
}

/// DTO de resposta da operação de inscrição ou cancelamento (`EventRegistrationResponse` no backend).
final class EventRegistrationDto {
  const EventRegistrationDto({
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
  final String status;
  final DateTime registeredAt;
  final DateTime? cancelledAt;
  final String message;

  /// Desserializa um objeto JSON de participação/inscrição em evento.
  factory EventRegistrationDto.fromJson(Map<String, dynamic> json) {
    final idRaw = json['id'];
    final eventIdRaw = json['event_id'];
    final participantIdRaw = json['participant_id'];
    final status = json['status'];
    final registeredAtRaw = json['registered_at'];
    final cancelledAtRaw = json['cancelled_at'];
    final message = json['message'];

    if (idRaw is! String ||
        eventIdRaw is! String ||
        participantIdRaw is! String ||
        status is! String ||
        registeredAtRaw is! String ||
        (cancelledAtRaw != null && cancelledAtRaw is! String) ||
        message is! String) {
      throw const FormatException(
        'Contrato JSON de inscrição em evento inválido.',
      );
    }

    return EventRegistrationDto(
      id: _uuidConverter.fromJson(idRaw),
      eventId: _uuidConverter.fromJson(eventIdRaw),
      participantId: _uuidConverter.fromJson(participantIdRaw),
      status: status,
      registeredAt: DateTime.parse(registeredAtRaw).toUtc(),
      cancelledAt: cancelledAtRaw == null
          ? null
          : DateTime.parse(cancelledAtRaw as String).toUtc(),
      message: message,
    );
  }

  Map<String, dynamic> toJson() => {
    'id': _uuidConverter.toJson(id),
    'event_id': _uuidConverter.toJson(eventId),
    'participant_id': _uuidConverter.toJson(participantId),
    'status': status,
    'registered_at': registeredAt.toUtc().toIso8601String(),
    'cancelled_at': cancelledAt?.toUtc().toIso8601String(),
    'message': message,
  };
}

/// DTO de erro retornado pelos endpoints de eventos (`EventErrorResponse` no backend).
final class EventErrorDto {
  const EventErrorDto({required this.code, required this.message});

  final String code;
  final String message;

  static EventErrorDto? tryFromDynamic(Object? raw) {
    if (raw is Map<String, dynamic>) {
      final code = raw['code'];
      final message = raw['message'];
      if (code is String && message is String) {
        return EventErrorDto(code: code, message: message);
      }
    }
    return null;
  }
}
