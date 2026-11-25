/// Adaptador da camada de infraestrutura que conecta [EventsGateway] ao [EventsApi].
library;

import 'package:dio/dio.dart';
import 'package:uuid/uuid.dart';

import '../../../infrastructure/backend/events/events.dart';
import '../application/event_failure.dart';
import '../application/event_models.dart';
import '../application/events_gateway.dart';

/// Implementação de [EventsGateway] apoiada nos endpoints HTTP do backend.
final class BackendEventsGateway implements EventsGateway {
  const BackendEventsGateway(this._api);

  final EventsApi _api;

  @override
  Future<List<PublishedEvent>> listPublishedByAcademicCenter(
    UuidValue academicCenterId,
  ) async {
    try {
      final dtos = await _api.listPublishedEventsByCa(academicCenterId);
      return dtos.map(_mapDtoToDomain).toList(growable: false);
    } on DioException catch (error) {
      throw _mapDioException(
        error,
        fallbackMessage: 'Não foi possível carregar os eventos.',
      );
    } on FormatException catch (error) {
      throw EventQueryException(
        kind: EventQueryFailureKind.unexpected,
        message: 'Não foi possível carregar os eventos.',
        cause: error,
      );
    }
  }

  @override
  Future<PublishedEvent> getPublishedDetail({
    required UuidValue academicCenterId,
    required UuidValue eventId,
  }) async {
    try {
      final dto = await _api.getPublishedEventDetail(
        academicCenterId: academicCenterId,
        eventId: eventId,
      );
      final event = _mapDtoToDomain(dto);
      if (!event.isPublished || event.academicCenterId != academicCenterId) {
        throw const EventQueryException(
          kind: EventQueryFailureKind.notFound,
          message: 'Os detalhes do evento não estão disponíveis.',
        );
      }
      return event;
    } on DioException catch (error) {
      throw _mapDioException(
        error,
        fallbackMessage: 'Os detalhes do evento não estão disponíveis.',
      );
    } on FormatException catch (error) {
      throw EventQueryException(
        kind: EventQueryFailureKind.unexpected,
        message: 'Os detalhes do evento não estão disponíveis.',
        cause: error,
      );
    }
  }

  @override
  Future<EventRegistrationRecord> registerInEvent({
    required UuidValue academicCenterId,
    required UuidValue eventId,
  }) async {
    try {
      final dto = await _api.registerInEvent(
        academicCenterId: academicCenterId,
        eventId: eventId,
      );
      return _mapRegistrationDtoToDomain(dto);
    } on DioException catch (error) {
      throw _mapDioException(
        error,
        fallbackMessage: 'Não foi possível realizar a inscrição no evento.',
      );
    } on FormatException catch (error) {
      throw EventQueryException(
        kind: EventQueryFailureKind.unexpected,
        message: 'Não foi possível realizar a inscrição no evento.',
        cause: error,
      );
    }
  }

  @override
  Future<EventRegistrationRecord> cancelRegistration({
    required UuidValue academicCenterId,
    required UuidValue eventId,
  }) async {
    try {
      final dto = await _api.cancelRegistration(
        academicCenterId: academicCenterId,
        eventId: eventId,
      );
      return _mapRegistrationDtoToDomain(dto);
    } on DioException catch (error) {
      throw _mapDioException(
        error,
        fallbackMessage: 'Não foi possível cancelar a inscrição no evento.',
      );
    } on FormatException catch (error) {
      throw EventQueryException(
        kind: EventQueryFailureKind.unexpected,
        message: 'Não foi possível cancelar a inscrição no evento.',
        cause: error,
      );
    }
  }

  PublishedEvent _mapDtoToDomain(EventDto dto) => PublishedEvent(
    id: dto.id,
    academicCenterId: dto.academicCenterId,
    authorId: dto.authorId,
    title: dto.title,
    description: dto.description,
    location: dto.location,
    startsAt: dto.startsAt,
    endsAt: dto.endsAt,
    status: EventPublicationStatus.fromWire(dto.status),
    requiresRegistration: dto.requiresRegistration,
    maxCapacity: dto.maxCapacity,
    confirmedRegistrations: dto.confirmedRegistrations,
    remainingSpots: dto.remainingSpots,
    hasAvailableSpots: dto.hasAvailableSpots,
    registrationDeadline: dto.registrationDeadline,
    publishedAt: dto.publishedAt,
  );

  EventRegistrationRecord _mapRegistrationDtoToDomain(
    EventRegistrationDto dto,
  ) => EventRegistrationRecord(
    id: dto.id,
    eventId: dto.eventId,
    participantId: dto.participantId,
    status: EventRegistrationStatus.fromWire(dto.status),
    registeredAt: dto.registeredAt,
    cancelledAt: dto.cancelledAt,
    message: dto.message,
  );

  EventQueryException _mapDioException(
    DioException error, {
    required String fallbackMessage,
  }) {
    final statusCode = error.response?.statusCode;
    final errorDto = EventErrorDto.tryFromDynamic(error.response?.data);
    final code = errorDto?.code;

    if (code == 'event_full') {
      return EventQueryException(
        kind: EventQueryFailureKind.eventFull,
        message: 'As vagas para este evento estão esgotadas.',
        cause: error,
      );
    }

    if (code == 'already_registered') {
      return EventQueryException(
        kind: EventQueryFailureKind.alreadyRegistered,
        message: 'Você já está inscrito neste evento.',
        cause: error,
      );
    }

    if (code == 'no_active_registration') {
      return EventQueryException(
        kind: EventQueryFailureKind.noActiveRegistration,
        message: 'Não há inscrição ativa para cancelar.',
        cause: error,
      );
    }

    if (code == 'event_not_available' || statusCode == 422) {
      return EventQueryException(
        kind: EventQueryFailureKind.eventNotAvailable,
        message: 'Este evento não está disponível para inscrição.',
        cause: error,
      );
    }

    if (statusCode == 404 || code == 'not_found') {
      return EventQueryException(
        kind: EventQueryFailureKind.notFound,
        message: 'Os detalhes do evento não estão disponíveis.',
        cause: error,
      );
    }

    if (statusCode == 400 || (code != null && code.startsWith('missing:'))) {
      return EventQueryException(
        kind: EventQueryFailureKind.invalidRequest,
        message: errorDto?.message ?? fallbackMessage,
        cause: error,
      );
    }

    return EventQueryException(
      kind: EventQueryFailureKind.unavailable,
      message: fallbackMessage,
      cause: error,
    );
  }
}
