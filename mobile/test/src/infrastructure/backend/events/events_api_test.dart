/// Testes unitários do cliente HTTP [EventsApi], dos contratos JSON
/// [EventDto] / [EventRegistrationDto] e do mapeamento de erros em [BackendEventsGateway].
library;

import 'package:capp/src/infrastructure/backend/events/events.dart';
import 'package:capp/src/modules/events/events.dart';
import 'package:capp/src/modules/events/infrastructure/backend_events_gateway.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

import '../../../../support/http_fakes.dart';

void main() {
  final caId = UuidValue.fromString('550e8400-e29b-41d4-a716-446655440000');
  final eventId = UuidValue.fromString('660e8400-e29b-41d4-a716-446655440001');
  final authorId = UuidValue.fromString('770e8400-e29b-41d4-a716-446655440002');
  final registrationId = UuidValue.fromString(
    '880e8400-e29b-41d4-a716-446655440003',
  );
  final participantId = UuidValue.fromString(
    '990e8400-e29b-41d4-a716-446655440004',
  );

  Map<String, dynamic> sampleEventJson({
    String title = 'Palestra de IA',
    String description = 'A palestra acontecerá no auditório 2.',
  }) => {
    'id': eventId.uuid,
    'academic_center_id': caId.uuid,
    'author_id': authorId.uuid,
    'title': title,
    'description': description,
    'location': 'Auditório 2',
    'starts_at': '2026-11-28T14:00:00Z',
    'ends_at': '2026-11-28T16:00:00Z',
    'status': 'published',
    'requires_registration': true,
    'max_capacity': 100,
    'confirmed_registrations': 12,
    'remaining_spots': 88,
    'has_available_spots': true,
    'registration_deadline': '2026-11-28T12:00:00Z',
    'published_at': '2026-11-20T10:00:00Z',
    'created_at': '2026-11-20T09:00:00Z',
    'updated_at': '2026-11-20T10:00:00Z',
  };

  Map<String, dynamic> sampleRegistrationJson({
    String status = 'confirmed',
    String? cancelledAt,
    String message = 'Inscrição realizada com sucesso.',
  }) => {
    'id': registrationId.uuid,
    'event_id': eventId.uuid,
    'participant_id': participantId.uuid,
    'status': status,
    'registered_at': '2026-11-25T12:00:00Z',
    'cancelled_at': cancelledAt,
    'message': message,
  };

  test(
    'listPublishedEventsByCa consome a rota do CA ativo e desserializa DTOs',
    () async {
      RequestOptions? captured;
      final dio = createTestDio(
        FakeHttpAdapter((options) {
          captured = options;
          return jsonTestResponse([sampleEventJson()]);
        }),
      );

      final api = EventsApi(dio);
      final events = await api.listPublishedEventsByCa(caId);

      expect(captured?.path, '/api/v1/academic-centers/${caId.uuid}/events');
      expect(captured?.queryParameters['only_published'], isTrue);
      expect(captured?.headers['X-Academic-Center-Id'], caId.uuid);
      expect(events, hasLength(1));
      expect(events.single.id, eventId);
      expect(events.single.academicCenterId, caId);
      expect(events.single.title, 'Palestra de IA');
      expect(events.single.remainingSpots, 88);
    },
  );

  test(
    'getPublishedEventDetail consome a rota de detalhes do evento do CA',
    () async {
      RequestOptions? captured;
      final dio = createTestDio(
        FakeHttpAdapter((options) {
          captured = options;
          return jsonTestResponse(sampleEventJson(title: 'Mudança de sala'));
        }),
      );

      final api = EventsApi(dio);
      final detail = await api.getPublishedEventDetail(
        academicCenterId: caId,
        eventId: eventId,
      );

      expect(
        captured?.path,
        '/api/v1/academic-centers/${caId.uuid}/events/${eventId.uuid}',
      );
      expect(detail.title, 'Mudança de sala');
      expect(detail.description, 'A palestra acontecerá no auditório 2.');
    },
  );

  test('registerInEvent envia POST autenticado com X-Academic-Center-Id e desserializa resposta', () async {
    RequestOptions? captured;
    final dio = createTestDio(
      FakeHttpAdapter((options) {
        captured = options;
        return jsonTestResponse(sampleRegistrationJson(), status: 201);
      }),
    );

    final api = EventsApi(dio);
    final dto = await api.registerInEvent(
      academicCenterId: caId,
      eventId: eventId,
    );

    expect(captured?.method, 'POST');
    expect(
      captured?.path,
      '/api/v1/academic-centers/${caId.uuid}/events/${eventId.uuid}/registrations',
    );
    expect(captured?.headers['X-Academic-Center-Id'], caId.uuid);
    expect(captured?.extra['firebaseAuth'], isTrue);
    expect(dto.id, registrationId);
    expect(dto.eventId, eventId);
    expect(dto.participantId, participantId);
    expect(dto.status, 'confirmed');
    expect(dto.cancelledAt, isNull);
    expect(dto.message, 'Inscrição realizada com sucesso.');
  });

  test('cancelRegistration envia DELETE autenticado com X-Academic-Center-Id e desserializa resposta', () async {
    RequestOptions? captured;
    final dio = createTestDio(
      FakeHttpAdapter((options) {
        captured = options;
        return jsonTestResponse(
          sampleRegistrationJson(
            status: 'cancelled',
            cancelledAt: '2026-11-25T12:30:00Z',
            message: 'Cancelamento realizado com sucesso.',
          ),
        );
      }),
    );

    final api = EventsApi(dio);
    final dto = await api.cancelRegistration(
      academicCenterId: caId,
      eventId: eventId,
    );

    expect(captured?.method, 'DELETE');
    expect(
      captured?.path,
      '/api/v1/academic-centers/${caId.uuid}/events/${eventId.uuid}/registrations',
    );
    expect(captured?.headers['X-Academic-Center-Id'], caId.uuid);
    expect(captured?.extra['firebaseAuth'], isTrue);
    expect(dto.status, 'cancelled');
    expect(dto.cancelledAt, DateTime.utc(2026, 11, 25, 12, 30));
    expect(dto.message, 'Cancelamento realizado com sucesso.');
  });

  test('BackendEventsGateway mapeia códigos de erro de negócio do backend para EventQueryFailureKind', () async {
    Future<void> expectMappedFailure({
      required int statusCode,
      required String backendCode,
      required EventQueryFailureKind expectedKind,
      required String expectedMessage,
    }) async {
      final dio = createTestDio(
        FakeHttpAdapter(
          (_) => jsonTestResponse({
            'code': backendCode,
            'message': 'Mensagem original do servidor',
          }, status: statusCode),
        ),
      );
      final gateway = BackendEventsGateway(EventsApi(dio));

      await expectLater(
        () => gateway.registerInEvent(academicCenterId: caId, eventId: eventId),
        throwsA(
          isA<EventQueryException>()
              .having((e) => e.kind, 'kind', expectedKind)
              .having((e) => e.message, 'message', expectedMessage),
        ),
      );
    }

    await expectMappedFailure(
      statusCode: 409,
      backendCode: 'event_full',
      expectedKind: EventQueryFailureKind.eventFull,
      expectedMessage: 'As vagas para este evento estão esgotadas.',
    );

    await expectMappedFailure(
      statusCode: 409,
      backendCode: 'already_registered',
      expectedKind: EventQueryFailureKind.alreadyRegistered,
      expectedMessage: 'Você já está inscrito neste evento.',
    );

    await expectMappedFailure(
      statusCode: 409,
      backendCode: 'no_active_registration',
      expectedKind: EventQueryFailureKind.noActiveRegistration,
      expectedMessage: 'Não há inscrição ativa para cancelar.',
    );

    await expectMappedFailure(
      statusCode: 422,
      backendCode: 'event_not_available',
      expectedKind: EventQueryFailureKind.eventNotAvailable,
      expectedMessage: 'Este evento não está disponível para inscrição.',
    );
  });
}
