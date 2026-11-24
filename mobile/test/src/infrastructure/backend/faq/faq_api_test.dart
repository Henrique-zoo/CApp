/// Testes do cliente HTTP [FaqApi] e seus DTOs contra o contrato OpenAPI.
library;

import 'package:capp/src/infrastructure/backend/faq/faq.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

import '../../../../support/http_fakes.dart';

void main() {
  const caId = UuidValue.raw('00000000-0000-0000-0000-000000000001');
  const entryId = UuidValue.raw('00000000-0000-0000-0000-000000000010');
  const authorId = UuidValue.raw('00000000-0000-0000-0000-000000000020');

  Map<String, Object?> sampleEntryJson({String? category = 'Carteirinhas'}) => {
    'id': entryId.uuid,
    'academicCenterId': caId.uuid,
    'question': 'Como solicito a carteirinha de estudante do CA?',
    'answer': 'Compareça ao balcão do CA com documento oficial.',
    'category': category,
    'createdBy': authorId.uuid,
    'updatedBy': authorId.uuid,
    'createdAt': '2026-11-22T13:00:00Z',
    'updatedAt': '2026-11-22T14:30:00Z',
  };

  test('listEntries envia filtros category e q com headers protegidos', () async {
    final adapter = FakeHttpAdapter(
      (_) => jsonTestResponse([sampleEntryJson()]),
    );
    final api = FaqApi(createTestDio(adapter));

    final entries = await api.listEntries(
      academicCenterId: caId,
      category: ' Carteirinhas ',
      query: ' balcão ',
    );

    final request = adapter.requests.single;
    expect(request.method, 'GET');
    expect(request.path, '/api/v1/academic-centers/${caId.uuid}/faq');
    expect(request.queryParameters, {
      'category': 'Carteirinhas',
      'q': 'balcão',
    });
    expect(request.extra['firebaseAuth'], isTrue);
    expect(request.extra['firebaseAppCheck'], isTrue);
    expect(entries, hasLength(1));
    expect(entries.single.id, entryId);
    expect(entries.single.category, 'Carteirinhas');
  });

  test('listEntries aceita categoria omitida ou nula', () async {
    final json = sampleEntryJson()..remove('category');
    final api = FaqApi(
      createTestDio(FakeHttpAdapter((_) => jsonTestResponse([json]))),
    );

    final entries = await api.listEntries(academicCenterId: caId);
    expect(entries.single.category, isNull);
  });

  test('listEntries rejeita raiz não vetorial com FormatException', () async {
    final api = FaqApi(
      createTestDio(FakeHttpAdapter((_) => jsonTestResponse(<String, Object?>{}))),
    );

    await expectLater(
      api.listEntries(academicCenterId: caId),
      throwsFormatException,
    );
  });

  test('listCategories desserializa lista de strings do endpoint', () async {
    final adapter = FakeHttpAdapter(
      (_) => jsonTestResponse(['Apoio Acadêmico', 'Carteirinhas']),
    );
    final api = FaqApi(createTestDio(adapter));

    final categories = await api.listCategories(academicCenterId: caId);

    final request = adapter.requests.single;
    expect(request.method, 'GET');
    expect(
      request.path,
      '/api/v1/academic-centers/${caId.uuid}/faq/categories',
    );
    expect(categories, ['Apoio Acadêmico', 'Carteirinhas']);
  });

  test('createEntry envia POST e exige HTTP 201', () async {
    final adapter = FakeHttpAdapter(
      (_) => jsonTestResponse(sampleEntryJson(), status: 201),
    );
    final api = FaqApi(createTestDio(adapter));

    final created = await api.createEntry(
      academicCenterId: caId,
      request: const CreateFaqEntryRequest(
        question: 'Como solicito a carteirinha?',
        answer: 'No balcão do CA.',
      ),
    );

    final request = adapter.requests.single;
    expect(request.method, 'POST');
    expect(request.data, {
      'question': 'Como solicito a carteirinha?',
      'answer': 'No balcão do CA.',
    });
    expect(created.id, entryId);
  });

  test('updateEntry envia PATCH apenas com campos preenchidos', () async {
    final adapter = FakeHttpAdapter(
      (_) => jsonTestResponse(sampleEntryJson(), status: 200),
    );
    final api = FaqApi(createTestDio(adapter));

    await api.updateEntry(
      academicCenterId: caId,
      faqEntryId: entryId,
      request: const UpdateFaqEntryRequest(answer: 'Nova resposta.'),
    );

    final request = adapter.requests.single;
    expect(request.method, 'PATCH');
    expect(
      request.path,
      '/api/v1/academic-centers/${caId.uuid}/faq/${entryId.uuid}',
    );
    expect(request.data, {'answer': 'Nova resposta.'});
  });

  test('deleteEntry envia DELETE e aceita HTTP 204', () async {
    final adapter = FakeHttpAdapter(
      (_) => ResponseBody.fromString('', 204),
    );
    final api = FaqApi(createTestDio(adapter));

    await api.deleteEntry(academicCenterId: caId, faqEntryId: entryId);

    final request = adapter.requests.single;
    expect(request.method, 'DELETE');
    expect(
      request.path,
      '/api/v1/academic-centers/${caId.uuid}/faq/${entryId.uuid}',
    );
  });
}
