/// Testes de widget e controlador para listagem e detalhes de eventos publicados (#95).
///
/// Cobre os estados principais:
/// - listagem populada (exibindo apenas eventos publicados do CA ativo);
/// - lista vazia quando não há eventos publicados no CA ativo;
/// - estado de erro com opção de tentar novamente;
/// - isolamento e troca de Centro Acadêmico ativo;
/// - visualização de detalhes de um evento publicado e recusa para evento indisponível.
library;

import 'package:capp/src/app/theme.dart';
import 'package:capp/src/modules/events/events.dart';
import 'package:capp/src/modules/events/events_widgets.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

final class _FakeEventsGateway implements EventsGateway {
  final Map<UuidValue, List<PublishedEvent>> eventsByCa = {};
  bool shouldFailList = false;
  bool shouldFailDetail = false;
  int listCalls = 0;

  @override
  Future<List<PublishedEvent>> listPublishedByAcademicCenter(
    UuidValue academicCenterId,
  ) async {
    listCalls += 1;
    if (shouldFailList) {
      throw const EventQueryException(
        kind: EventQueryFailureKind.unavailable,
        message: 'Não foi possível carregar os eventos.',
      );
    }
    return eventsByCa[academicCenterId] ?? const [];
  }

  @override
  Future<PublishedEvent> getPublishedDetail({
    required UuidValue academicCenterId,
    required UuidValue eventId,
  }) async {
    if (shouldFailDetail) {
      throw const EventQueryException(
        kind: EventQueryFailureKind.notFound,
        message: 'Os detalhes do evento não estão disponíveis.',
      );
    }
    final list = eventsByCa[academicCenterId] ?? const [];
    for (final event in list) {
      if (event.id == eventId && event.isPublished) {
        return event;
      }
    }
    throw const EventQueryException(
      kind: EventQueryFailureKind.notFound,
      message: 'Os detalhes do evento não estão disponíveis.',
    );
  }
}

void main() {
  final caComputacao = AcademicCenterContext(
    id: UuidValue.fromString('550e8400-e29b-41d4-a716-446655440010'),
    name: 'CA de Computação',
    acronym: 'CACOMP',
  );
  final caMatematica = AcademicCenterContext(
    id: UuidValue.fromString('550e8400-e29b-41d4-a716-446655440020'),
    name: 'CA de Matemática',
    acronym: 'CAMAT',
  );
  final authorId = UuidValue.fromString('770e8400-e29b-41d4-a716-446655440099');

  PublishedEvent buildEvent({
    required UuidValue id,
    required UuidValue caId,
    required String title,
    String description = 'Descrição detalhada do evento.',
    String location = 'Auditório 2',
    EventPublicationStatus status = EventPublicationStatus.published,
    int? maxCapacity = 50,
    int confirmedRegistrations = 10,
  }) => PublishedEvent(
    id: id,
    academicCenterId: caId,
    authorId: authorId,
    title: title,
    description: description,
    location: location,
    startsAt: DateTime.utc(2026, 11, 28, 14),
    endsAt: DateTime.utc(2026, 11, 28, 16),
    status: status,
    requiresRegistration: true,
    maxCapacity: maxCapacity,
    confirmedRegistrations: confirmedRegistrations,
    remainingSpots: maxCapacity == null
        ? null
        : maxCapacity - confirmedRegistrations,
    hasAvailableSpots:
        maxCapacity == null || confirmedRegistrations < maxCapacity,
    registrationDeadline: DateTime.utc(2026, 11, 28, 12),
    publishedAt: status == EventPublicationStatus.published
        ? DateTime.utc(2026, 11, 20, 10)
        : null,
  );

  Widget wrapWithTheme(Widget home) =>
      MaterialApp(theme: createDarkTheme(), home: Scaffold(body: home));

  testWidgets(
    'exibe somente eventos publicados do CA ativo e abre os detalhes ao tocar no card',
    (tester) async {
      final gateway = _FakeEventsGateway();
      final semanaId = UuidValue.fromString(
        '660e8400-e29b-41d4-a716-446655440101',
      );
      final palestraId = UuidValue.fromString(
        '660e8400-e29b-41d4-a716-446655440102',
      );
      final rascunhoId = UuidValue.fromString(
        '660e8400-e29b-41d4-a716-446655440103',
      );

      gateway.eventsByCa[caComputacao.id] = [
        buildEvent(
          id: semanaId,
          caId: caComputacao.id,
          title: 'Semana Acadêmica',
        ),
        buildEvent(
          id: palestraId,
          caId: caComputacao.id,
          title: 'Mudança de sala',
          description: 'A palestra acontecerá no auditório 2.',
        ),
        buildEvent(
          id: rascunhoId,
          caId: caComputacao.id,
          title: 'Encontro em rascunho',
          status: EventPublicationStatus.draft,
        ),
      ];

      final controller = EventsScheduleController(
        gateway: gateway,
        academicCenters: [caComputacao],
      );

      await tester.pumpWidget(
        wrapWithTheme(EventsSchedulePage(controller: controller)),
      );
      await tester.pumpAndSettle();

      expect(find.text('Semana Acadêmica'), findsOneWidget);
      expect(find.text('Mudança de sala'), findsOneWidget);
      expect(find.text('Encontro em rascunho'), findsNothing);

      await tester.tap(find.text('Mudança de sala'));
      await tester.pumpAndSettle();

      expect(find.byType(EventDetailPage), findsOneWidget);
      expect(find.text('Mudança de sala'), findsOneWidget);
      expect(
        find.text('A palestra acontecerá no auditório 2.'),
        findsOneWidget,
      );
      expect(find.text('Auditório 2'), findsOneWidget);
    },
  );

  testWidgets('informa lista vazia quando não há eventos publicados no CA', (
    tester,
  ) async {
    final gateway = _FakeEventsGateway();
    final controller = EventsScheduleController(
      gateway: gateway,
      academicCenters: [caComputacao],
    );

    await tester.pumpWidget(
      wrapWithTheme(EventsSchedulePage(controller: controller)),
    );
    await tester.pumpAndSettle();

    expect(find.text('Não há eventos publicados.'), findsOneWidget);
  });

  testWidgets(
    'apresenta erro ao falhar carregamento e permite tentar novamente',
    (tester) async {
      final gateway = _FakeEventsGateway()..shouldFailList = true;
      final eventoId = UuidValue.fromString(
        '660e8400-e29b-41d4-a716-446655440201',
      );
      gateway.eventsByCa[caComputacao.id] = [
        buildEvent(
          id: eventoId,
          caId: caComputacao.id,
          title: 'Palestra de IA',
        ),
      ];

      final controller = EventsScheduleController(
        gateway: gateway,
        academicCenters: [caComputacao],
      );

      await tester.pumpWidget(
        wrapWithTheme(EventsSchedulePage(controller: controller)),
      );
      await tester.pumpAndSettle();

      expect(
        find.text('Não foi possível carregar os eventos.'),
        findsOneWidget,
      );
      expect(find.text('Tentar novamente'), findsOneWidget);

      gateway.shouldFailList = false;
      await tester.tap(find.text('Tentar novamente'));
      await tester.pumpAndSettle();

      expect(find.text('Palestra de IA'), findsOneWidget);
      expect(gateway.listCalls, 2);
    },
  );

  testWidgets('isola os eventos exibidos ao alternar o CA ativo', (
    tester,
  ) async {
    final gateway = _FakeEventsGateway();
    gateway.eventsByCa[caComputacao.id] = [
      buildEvent(
        id: UuidValue.fromString('660e8400-e29b-41d4-a716-446655440301'),
        caId: caComputacao.id,
        title: 'Hackathon de Computação',
      ),
    ];
    gateway.eventsByCa[caMatematica.id] = [
      buildEvent(
        id: UuidValue.fromString('660e8400-e29b-41d4-a716-446655440302'),
        caId: caMatematica.id,
        title: 'Semana de Matemática',
      ),
    ];

    final controller = EventsScheduleController(
      gateway: gateway,
      academicCenters: [caComputacao, caMatematica],
    );

    await tester.pumpWidget(
      wrapWithTheme(EventsSchedulePage(controller: controller)),
    );
    await tester.pumpAndSettle();

    expect(find.text('Hackathon de Computação'), findsOneWidget);
    expect(find.text('Semana de Matemática'), findsNothing);

    await controller.selectAcademicCenter(caMatematica);
    await tester.pumpAndSettle();

    expect(find.text('Semana de Matemática'), findsOneWidget);
    expect(find.text('Hackathon de Computação'), findsNothing);
  });

  testWidgets(
    'não disponibiliza detalhes quando o evento não está publicado',
    (tester) async {
      final gateway = _FakeEventsGateway()..shouldFailDetail = true;
      final controller = EventsScheduleController(
        gateway: gateway,
        academicCenters: [caComputacao],
      );
      final unpublishedId = UuidValue.fromString(
        '660e8400-e29b-41d4-a716-446655440401',
      );

      await tester.pumpWidget(
        MaterialApp(
          theme: createDarkTheme(),
          home: EventDetailPage(
            controller: controller,
            eventId: unpublishedId,
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(
        find.text('Os detalhes deste evento não estão disponíveis.'),
        findsOneWidget,
      );
    },
  );
}
