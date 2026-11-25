/// Testes de widget e controlador para listagem, detalhes, inscrição,
/// cancelamento e controle de vagas de eventos publicados (#95, #96 e #99).
///
/// Cobre os estados principais:
/// - listagem populada (exibindo apenas eventos publicados do CA ativo);
/// - lista vazia quando não há eventos publicados no CA ativo;
/// - estado de erro com opção de tentar novamente;
/// - isolamento e troca de Centro Acadêmico ativo;
/// - visualização de detalhes de um evento publicado e recusa para evento indisponível;
/// - inscrição bem-sucedida com redução imediata de vagas e indicador "Inscrição confirmada";
/// - cancelamento de inscrição com diálogo de confirmação e liberação imediata de vaga;
/// - bloqueio visual quando evento está lotado ("Vagas esgotadas"), prazo encerrado
///   ("Inscrições encerradas") ou entrada livre (`requiresRegistration == false`);
/// - tratamento de erro de negócio ao tentar se inscrever quando última vaga esgota (`event_full`).
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
  EventQueryException? registrationError;
  int listCalls = 0;
  int registerCalls = 0;
  int cancelCalls = 0;

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

  @override
  Future<EventRegistrationRecord> registerInEvent({
    required UuidValue academicCenterId,
    required UuidValue eventId,
  }) async {
    registerCalls += 1;
    final error = registrationError;
    if (error != null) {
      throw error;
    }

    _updateStoredEventSpots(academicCenterId, eventId, delta: 1);

    return EventRegistrationRecord(
      id: UuidValue.fromString('880e8400-e29b-41d4-a716-446655440900'),
      eventId: eventId,
      participantId: UuidValue.fromString(
        '990e8400-e29b-41d4-a716-446655440901',
      ),
      status: EventRegistrationStatus.confirmed,
      registeredAt: DateTime.utc(2026, 11, 25, 14),
      cancelledAt: null,
      message: 'Inscrição realizada com sucesso.',
    );
  }

  @override
  Future<EventRegistrationRecord> cancelRegistration({
    required UuidValue academicCenterId,
    required UuidValue eventId,
  }) async {
    cancelCalls += 1;
    final error = registrationError;
    if (error != null) {
      throw error;
    }

    _updateStoredEventSpots(academicCenterId, eventId, delta: -1);

    return EventRegistrationRecord(
      id: UuidValue.fromString('880e8400-e29b-41d4-a716-446655440900'),
      eventId: eventId,
      participantId: UuidValue.fromString(
        '990e8400-e29b-41d4-a716-446655440901',
      ),
      status: EventRegistrationStatus.cancelled,
      registeredAt: DateTime.utc(2026, 11, 25, 14),
      cancelledAt: DateTime.utc(2026, 11, 25, 14, 30),
      message: 'Cancelamento realizado com sucesso.',
    );
  }

  void _updateStoredEventSpots(
    UuidValue caId,
    UuidValue eventId, {
    required int delta,
  }) {
    final list = eventsByCa[caId];
    if (list == null) return;
    final index = list.indexWhere((e) => e.id == eventId);
    if (index == -1) return;
    final current = list[index];
    final nextConfirmed = (current.confirmedRegistrations + delta).clamp(
      0,
      1 << 30,
    );
    list[index] = current.copyWith(confirmedRegistrations: nextConfirmed);
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
    bool requiresRegistration = true,
    int? maxCapacity = 50,
    int confirmedRegistrations = 10,
    DateTime? registrationDeadline,
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
    requiresRegistration: requiresRegistration,
    maxCapacity: maxCapacity,
    confirmedRegistrations: confirmedRegistrations,
    remainingSpots: maxCapacity == null
        ? null
        : maxCapacity - confirmedRegistrations,
    hasAvailableSpots:
        maxCapacity == null || confirmedRegistrations < maxCapacity,
    registrationDeadline:
        registrationDeadline ?? DateTime.utc(2030, 11, 28, 12),
    publishedAt: status == EventPublicationStatus.published
        ? DateTime.utc(2026, 11, 20, 10)
        : null,
  );

  Widget wrapWithTheme(Widget home) => MaterialApp(
    theme: createDarkTheme(),
    home: Scaffold(body: home),
  );

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

  testWidgets('não disponibiliza detalhes quando o evento não está publicado', (
    tester,
  ) async {
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
        home: EventDetailPage(controller: controller, eventId: unpublishedId),
      ),
    );
    await tester.pumpAndSettle();

    expect(
      find.text('Os detalhes deste evento não estão disponíveis.'),
      findsOneWidget,
    );
  });

  testWidgets(
    'realiza inscrição com sucesso, reduz vagas imediatamente e cancela após confirmação no diálogo',
    (tester) async {
      final gateway = _FakeEventsGateway();
      final eventId = UuidValue.fromString(
        '660e8400-e29b-41d4-a716-446655440501',
      );
      gateway.eventsByCa[caComputacao.id] = [
        buildEvent(
          id: eventId,
          caId: caComputacao.id,
          title: 'Oficina de Flutter',
          maxCapacity: 20,
          confirmedRegistrations: 18,
        ),
      ];

      final controller = EventsScheduleController(
        gateway: gateway,
        academicCenters: [caComputacao],
      );
      await controller.loadEvents();

      await tester.pumpWidget(
        MaterialApp(
          theme: createDarkTheme(),
          home: EventDetailPage(controller: controller, eventId: eventId),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('2 de 20 vagas disponíveis'), findsOneWidget);
      expect(find.text('Inscrever-se'), findsOneWidget);

      await tester.tap(find.text('Inscrever-se'));
      await tester.pumpAndSettle();

      expect(gateway.registerCalls, 1);
      expect(controller.isRegisteredIn(eventId), isTrue);
      expect(find.text('Inscrição confirmada'), findsOneWidget);
      expect(find.text('Cancelar inscrição'), findsOneWidget);
      expect(find.text('1 de 20 vagas disponíveis'), findsOneWidget);
      expect(find.text('Inscrição realizada com sucesso.'), findsOneWidget);

      // Toca em Cancelar inscrição -> abre diálogo de confirmação
      await tester.tap(find.text('Cancelar inscrição'));
      await tester.pumpAndSettle();

      expect(find.text('Cancelar inscrição?'), findsOneWidget);
      expect(find.text('Confirmar cancelamento'), findsOneWidget);

      await tester.tap(find.text('Confirmar cancelamento'));
      await tester.pumpAndSettle();

      expect(gateway.cancelCalls, 1);
      expect(controller.isRegisteredIn(eventId), isFalse);
      expect(find.text('Inscrever-se'), findsOneWidget);
      expect(find.text('2 de 20 vagas disponíveis'), findsOneWidget);
      expect(find.text('Cancelamento realizado com sucesso.'), findsOneWidget);
    },
  );

  testWidgets(
    'desabilita botão com "Vagas esgotadas" quando o evento atingiu lotação máxima',
    (tester) async {
      final gateway = _FakeEventsGateway();
      final fullEventId = UuidValue.fromString(
        '660e8400-e29b-41d4-a716-446655440502',
      );
      gateway.eventsByCa[caComputacao.id] = [
        buildEvent(
          id: fullEventId,
          caId: caComputacao.id,
          title: 'Minicurso de Rust',
          maxCapacity: 30,
          confirmedRegistrations: 30,
        ),
      ];

      final controller = EventsScheduleController(
        gateway: gateway,
        academicCenters: [caComputacao],
      );

      await tester.pumpWidget(
        MaterialApp(
          theme: createDarkTheme(),
          home: EventDetailPage(controller: controller, eventId: fullEventId),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('Lotado (30/30 vagas preenchidas)'), findsOneWidget);
      final buttonFinder = find.byKey(const ValueKey('event-full-button'));
      expect(buttonFinder, findsOneWidget);
      final button = tester.widget<FilledButton>(buttonFinder);
      expect(button.onPressed, isNull);
      expect(find.text('Vagas esgotadas'), findsOneWidget);
    },
  );

  testWidgets(
    'desabilita botão com "Inscrições encerradas" quando o prazo de inscrição já expirou',
    (tester) async {
      final gateway = _FakeEventsGateway();
      final closedEventId = UuidValue.fromString(
        '660e8400-e29b-41d4-a716-446655440503',
      );
      gateway.eventsByCa[caComputacao.id] = [
        buildEvent(
          id: closedEventId,
          caId: caComputacao.id,
          title: 'Simpósio de Redes',
          maxCapacity: 40,
          confirmedRegistrations: 10,
          registrationDeadline: DateTime.utc(2026, 11, 20, 18),
        ),
      ];

      final controller = EventsScheduleController(
        gateway: gateway,
        academicCenters: [caComputacao],
      );

      await tester.pumpWidget(
        MaterialApp(
          theme: createDarkTheme(),
          home: EventDetailPage(
            controller: controller,
            eventId: closedEventId,
            referenceTime: DateTime.utc(2026, 11, 25, 12),
          ),
        ),
      );
      await tester.pumpAndSettle();

      final buttonFinder = find.byKey(
        const ValueKey('registration-closed-button'),
      );
      expect(buttonFinder, findsOneWidget);
      final button = tester.widget<FilledButton>(buttonFinder);
      expect(button.onPressed, isNull);
      expect(find.text('Inscrições encerradas'), findsOneWidget);
    },
  );

  testWidgets(
    'não exibe botão de inscrição quando o evento é de entrada livre (requiresRegistration == false)',
    (tester) async {
      final gateway = _FakeEventsGateway();
      final freeEntryId = UuidValue.fromString(
        '660e8400-e29b-41d4-a716-446655440504',
      );
      gateway.eventsByCa[caComputacao.id] = [
        buildEvent(
          id: freeEntryId,
          caId: caComputacao.id,
          title: 'Assembleia Geral Estudantil',
          requiresRegistration: false,
          maxCapacity: null,
          confirmedRegistrations: 0,
        ),
      ];

      final controller = EventsScheduleController(
        gateway: gateway,
        academicCenters: [caComputacao],
      );

      await tester.pumpWidget(
        MaterialApp(
          theme: createDarkTheme(),
          home: EventDetailPage(controller: controller, eventId: freeEntryId),
        ),
      );
      await tester.pumpAndSettle();

      expect(
        find.text('Entrada livre (não requer inscrição prévia)'),
        findsOneWidget,
      );
      expect(find.text('Inscrever-se'), findsNothing);
      expect(find.text('Vagas esgotadas'), findsNothing);
    },
  );

  testWidgets(
    'exibe feedback de erro e atualiza estado para lotado quando o servidor retorna event_full',
    (tester) async {
      final gateway = _FakeEventsGateway()
        ..registrationError = const EventQueryException(
          kind: EventQueryFailureKind.eventFull,
          message: 'As vagas para este evento estão esgotadas.',
        );
      final raceEventId = UuidValue.fromString(
        '660e8400-e29b-41d4-a716-446655440505',
      );
      gateway.eventsByCa[caComputacao.id] = [
        buildEvent(
          id: raceEventId,
          caId: caComputacao.id,
          title: 'Workshop de Segurança',
          maxCapacity: 15,
          confirmedRegistrations: 14,
        ),
      ];

      final controller = EventsScheduleController(
        gateway: gateway,
        academicCenters: [caComputacao],
      );
      await controller.loadEvents();

      await tester.pumpWidget(
        MaterialApp(
          theme: createDarkTheme(),
          home: EventDetailPage(controller: controller, eventId: raceEventId),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('1 de 15 vagas disponíveis'), findsOneWidget);
      await tester.tap(find.text('Inscrever-se'));
      await tester.pumpAndSettle();

      expect(
        find.text('As vagas para este evento estão esgotadas.'),
        findsOneWidget,
      );
      expect(find.text('Vagas esgotadas'), findsOneWidget);
      expect(find.text('Lotado (15/15 vagas preenchidas)'), findsOneWidget);
    },
  );
}
