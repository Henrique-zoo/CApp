/// Testes unitários do cliente HTTP [EventsApi] e do contrato JSON [EventDto].
library;

import 'package:capp/src/infrastructure/backend/events/events.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

import '../../../../support/http_fakes.dart';

void main() {
  final caId = UuidValue.fromString('550e8400-e29b-41d4-a716-446655440000');
  final eventId = UuidValue.fromString('660e8400-e29b-41d4-a716-446655440001');
  final authorId = UuidValue.fromString('770e8400-e29b-41d4-a716-446655440002');

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

  test('listPublishedEventsByCa consome a rota do CA ativo e desserializa DTOs', () async {
    RequestOptions? captured;
    final dio = createTestDio(
      FakeHttpAdapter((options) {
        captured = options;
        return jsonTestResponse([sampleEventJson()]);
      }),
    );

    final api = EventsApi(dio);
    final events = await api.listPublishedEventsByCa(caId);

    expect(
      captured?.path,
      '/api/v1/academic-centers/${caId.uuid}/events',
    );
    expect(captured?.queryParameters['only_published'], isTrue);
    expect(captured?.headers['X-Academic-Center-Id'], caId.uuid);
    expect(events, hasLength(1));
    expect(events.single.id, eventId);
    expect(events.single.academicCenterId, caId);
    expect(events.single.title, 'Palestra de IA');
    expect(events.single.remainingSpots, 88);
  });

  test('getPublishedEventDetail consome a rota de detalhes do evento do CA', () async {
    RequestOptions? captured;
    final dio = createTestDio(
      FakeHttpAdapter((options) {
        captured = options;
        return jsonTestResponse(
          sampleEventJson(title: 'Mudança de sala'),
        );
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
  });
}
