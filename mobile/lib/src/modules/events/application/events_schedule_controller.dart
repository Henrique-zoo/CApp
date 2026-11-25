/// Controlador da agenda pública e consulta de detalhes de eventos por CA ativo.
library;

import 'package:flutter/foundation.dart';
import 'package:uuid/uuid.dart';

import 'event_failure.dart';
import 'event_models.dart';
import 'events_gateway.dart';

/// Coordena a seleção do CA ativo, a listagem de eventos publicados e a consulta de detalhes.
final class EventsScheduleController extends ChangeNotifier {
  EventsScheduleController({
    required EventsGateway gateway,
    List<AcademicCenterContext> academicCenters = const [],
    AcademicCenterContext? initialAcademicCenter,
  }) : // ignore: prefer_initializing_formals
       _gateway = gateway,
       _academicCenters = List.unmodifiable(academicCenters),
       _activeAcademicCenter =
           initialAcademicCenter ??
           (academicCenters.isNotEmpty ? academicCenters.first : null);

  final EventsGateway _gateway;
  List<AcademicCenterContext> _academicCenters;
  AcademicCenterContext? _activeAcademicCenter;
  List<PublishedEvent> _events = const [];
  bool _isLoading = false;
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

  /// Última falha ocorrida ao carregar os eventos do CA ativo, quando houver.
  EventQueryException? get error => _error;

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
          (event) =>
              event.isPublished && event.academicCenterId == activeCa.id,
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
  Future<void> selectAcademicCenter(AcademicCenterContext academicCenter) async {
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

    return detail;
  }
}
