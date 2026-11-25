/// Controlador da agenda pública, consulta de detalhes e inscrição em eventos por CA ativo.
library;

import 'package:flutter/foundation.dart';
import 'package:uuid/uuid.dart';

import 'event_failure.dart';
import 'event_models.dart';
import 'events_gateway.dart';

/// Coordena a seleção do CA ativo, a listagem de eventos publicados, a consulta de detalhes
/// e o ciclo de inscrição/cancelamento com atualização imediata de vagas.
final class EventsScheduleController extends ChangeNotifier {
  EventsScheduleController({
    required EventsGateway gateway,
    List<AcademicCenterContext> academicCenters = const [],
    AcademicCenterContext? initialAcademicCenter,
    Set<UuidValue> initialRegisteredEventIds = const {},
  }) : _academicCenters = List.unmodifiable(academicCenters),
       // ignore: prefer_initializing_formals
       _gateway = gateway,
       _activeAcademicCenter =
           initialAcademicCenter ??
           (academicCenters.isNotEmpty ? academicCenters.first : null),
       _registeredEventIds = {...initialRegisteredEventIds};

  final EventsGateway _gateway;
  List<AcademicCenterContext> _academicCenters;
  AcademicCenterContext? _activeAcademicCenter;
  List<PublishedEvent> _events = const [];
  final Set<UuidValue> _registeredEventIds;
  final Map<UuidValue, EventRegistrationRecord> _registrationsByEventId = {};
  bool _isLoading = false;
  bool _isSubmittingRegistration = false;
  EventQueryException? _error;
  int _loadGeneration = 0;

  /// Centros Acadêmicos disponíveis para seleção na agenda.
  List<AcademicCenterContext> get academicCenters => _academicCenters;

  /// Centro Acadêmico atualmente ativo na visualização.
  AcademicCenterContext? get activeAcademicCenter => _activeAcademicCenter;

  /// Eventos publicados pertencentes exclusivamente ao CA ativo.
  List<PublishedEvent> get events => _events;

  /// Indica se uma consulta da listagem do CA ativo está em andamento.
  bool get isLoading => _isLoading;

  /// Indica se uma requisição de inscrição ou cancelamento está em andamento.
  bool get isSubmittingRegistration => _isSubmittingRegistration;

  /// Última falha ocorrida ao carregar os eventos do CA ativo, quando houver.
  EventQueryException? get error => _error;

  /// Verifica se o usuário possui inscrição ativa no evento [eventId].
  bool isRegisteredIn(UuidValue eventId) =>
      _registeredEventIds.contains(eventId);

  /// Retorna o último registro de inscrição conhecido para o evento [eventId], quando houver.
  EventRegistrationRecord? registrationFor(UuidValue eventId) =>
      _registrationsByEventId[eventId];

  /// Busca um evento já carregado em memória pelo seu [eventId].
  PublishedEvent? findCachedEvent(UuidValue eventId) {
    for (final event in _events) {
      if (event.id == eventId) {
        return event;
      }
    }
    return null;
  }

  /// Carrega os eventos publicados do CA ativo atual.
  ///
  /// Filtra defensivamente apenas eventos com status [EventPublicationStatus.published]
  /// e cujo `academicCenterId` coincida com o CA ativo, garantindo isolamento estrito.
  Future<void> loadEvents() async {
    final activeCa = _activeAcademicCenter;
    if (activeCa == null) {
      _events = const [];
      _error = null;
      _isLoading = false;
      notifyListeners();
      return;
    }

    final generation = ++_loadGeneration;
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      final fetched = await _gateway.listPublishedByAcademicCenter(activeCa.id);
      if (generation != _loadGeneration) return;

      _events = List.unmodifiable(
        fetched.where(
          (event) => event.isPublished && event.academicCenterId == activeCa.id,
        ),
      );
      _isLoading = false;
      _error = null;
      notifyListeners();
    } on EventQueryException catch (failure) {
      if (generation != _loadGeneration) return;
      _events = const [];
      _isLoading = false;
      _error = failure;
      notifyListeners();
    } catch (cause) {
      if (generation != _loadGeneration) return;
      _events = const [];
      _isLoading = false;
      _error = EventQueryException(
        kind: EventQueryFailureKind.unavailable,
        message: 'Não foi possível carregar os eventos.',
        cause: cause,
      );
      notifyListeners();
    }
  }

  /// Altera o Centro Acadêmico ativo e recarrega imediatamente a lista de eventos publicados.
  Future<void> selectAcademicCenter(
    AcademicCenterContext academicCenter,
  ) async {
    if (!_academicCenters.contains(academicCenter)) {
      _academicCenters = List.unmodifiable([
        ..._academicCenters,
        academicCenter,
      ]);
    }
    _activeAcademicCenter = academicCenter;
    _events = const [];
    _error = null;
    await loadEvents();
  }

  /// Consulta os detalhes de um evento publicado específico dentro do CA ativo.
  ///
  /// Recusa o acesso com [EventQueryFailureKind.notFound] caso o evento retornado
  /// não esteja publicado ou pertença a outro Centro Acadêmico.
  Future<PublishedEvent> loadEventDetail(UuidValue eventId) async {
    final activeCa = _activeAcademicCenter;
    if (activeCa == null) {
      throw const EventQueryException(
        kind: EventQueryFailureKind.invalidRequest,
        message: 'Selecione um Centro Acadêmico ativo para consultar o evento.',
      );
    }

    final detail = await _gateway.getPublishedDetail(
      academicCenterId: activeCa.id,
      eventId: eventId,
    );

    if (!detail.isPublished || detail.academicCenterId != activeCa.id) {
      throw const EventQueryException(
        kind: EventQueryFailureKind.notFound,
        message: 'Os detalhes do evento não estão disponíveis.',
      );
    }

    _replaceCachedEvent(detail);
    return detail;
  }

  /// Realiza a inscrição do participante autenticado no evento [eventId] do CA ativo.
  ///
  /// Atualiza imediatamente a contagem de inscritos e as vagas disponíveis em memória.
  Future<EventRegistrationRecord> registerInEvent(
    UuidValue eventId, {
    PublishedEvent? currentEvent,
  }) async {
    final activeCa = _activeAcademicCenter;
    if (activeCa == null) {
      throw const EventQueryException(
        kind: EventQueryFailureKind.invalidRequest,
        message:
            'Selecione um Centro Acadêmico ativo para inscrever-se no evento.',
      );
    }

    _isSubmittingRegistration = true;
    notifyListeners();

    try {
      final record = await _gateway.registerInEvent(
        academicCenterId: activeCa.id,
        eventId: eventId,
      );

      _registeredEventIds.add(eventId);
      _registrationsByEventId[eventId] = record;

      final target = currentEvent ?? findCachedEvent(eventId);
      if (target != null) {
        final nextConfirmed = target.confirmedRegistrations + 1;
        final cap = target.maxCapacity;
        final nextRemaining = cap == null
            ? null
            : (cap - nextConfirmed).clamp(0, cap);
        final updated = target.copyWith(
          confirmedRegistrations: nextConfirmed,
          remainingSpots: nextRemaining,
          hasAvailableSpots: cap == null || (nextRemaining ?? 0) > 0,
        );
        _replaceCachedEvent(updated, notify: false);
      }

      return record;
    } on EventQueryException catch (failure) {
      if (failure.kind == EventQueryFailureKind.alreadyRegistered) {
        _registeredEventIds.add(eventId);
      } else if (failure.kind == EventQueryFailureKind.eventFull) {
        final target = currentEvent ?? findCachedEvent(eventId);
        if (target != null && target.maxCapacity != null) {
          _replaceCachedEvent(
            target.copyWith(
              confirmedRegistrations: target.maxCapacity,
              remainingSpots: 0,
              hasAvailableSpots: false,
            ),
            notify: false,
          );
        }
      }
      rethrow;
    } finally {
      _isSubmittingRegistration = false;
      notifyListeners();
    }
  }

  /// Cancela a inscrição ativa do participante autenticado no evento [eventId] do CA ativo.
  ///
  /// Libera imediatamente a vaga correspondente nos contadores em memória.
  Future<EventRegistrationRecord> cancelRegistration(
    UuidValue eventId, {
    PublishedEvent? currentEvent,
  }) async {
    final activeCa = _activeAcademicCenter;
    if (activeCa == null) {
      throw const EventQueryException(
        kind: EventQueryFailureKind.invalidRequest,
        message: 'Selecione um Centro Acadêmico ativo para cancelar inscrição.',
      );
    }

    _isSubmittingRegistration = true;
    notifyListeners();

    try {
      final record = await _gateway.cancelRegistration(
        academicCenterId: activeCa.id,
        eventId: eventId,
      );

      _registeredEventIds.remove(eventId);
      _registrationsByEventId[eventId] = record;

      final target = currentEvent ?? findCachedEvent(eventId);
      if (target != null) {
        final nextConfirmed = (target.confirmedRegistrations - 1).clamp(
          0,
          1 << 30,
        );
        final cap = target.maxCapacity;
        final nextRemaining = cap == null
            ? null
            : (cap - nextConfirmed).clamp(0, cap);
        final updated = target.copyWith(
          confirmedRegistrations: nextConfirmed,
          remainingSpots: nextRemaining,
          hasAvailableSpots: cap == null || (nextRemaining ?? 0) > 0,
        );
        _replaceCachedEvent(updated, notify: false);
      }

      return record;
    } on EventQueryException catch (failure) {
      if (failure.kind == EventQueryFailureKind.noActiveRegistration) {
        _registeredEventIds.remove(eventId);
      }
      rethrow;
    } finally {
      _isSubmittingRegistration = false;
      notifyListeners();
    }
  }

  void _replaceCachedEvent(PublishedEvent updated, {bool notify = true}) {
    final index = _events.indexWhere((item) => item.id == updated.id);
    if (index == -1) {
      return;
    }
    final nextList = [..._events];
    nextList[index] = updated;
    _events = List.unmodifiable(nextList);
    if (notify) {
      notifyListeners();
    }
  }
}
