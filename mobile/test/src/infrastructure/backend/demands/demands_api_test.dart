/// Testes do cliente HTTP [DemandsApi] conforme o contrato OpenAPI.
library;

import 'package:capp/src/infrastructure/backend/demands/demands.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

import '../../../../support/http_fakes.dart';

void main() {
  const caId = UuidValue.raw('00000000-0000-0000-0000-000000000001');
  const authorId = UuidValue.raw('00000000-0000-0000-0000-000000000010');
  const demandId = UuidValue.raw('00000000-0000-0000-0000-000000000020');
  const historyId = UuidValue.raw('00000000-0000-0000-0000-000000000030');
  const commentId = UuidValue.raw('00000000-0000-0000-0000-000000000040');

  Map<String, Object?> sampleDemandJson({
    String forum = 'physical_space',
    String status = 'open',
    String? courseName,
    List<Map<String, Object?>>? statusHistory,
  }) => {
    'id': demandId.uuid,
    'academicCenterId': caId.uuid,
    'authorId': authorId.uuid,
    'forum': forum,
    'title': 'Reparo do ar-condicionado',
    'description': 'O aparelho da sala não refrigera.',
    'category': 'Infraestrutura',
    'courseName': courseName,
    'status': status,
    'createdAt': '2026-11-20T12:00:00Z',
    'updatedAt': '2026-11-20T13:00:00Z',
    'statusHistory': ?statusHistory,
  };

  test('listDemands envia query params e desserializa lista', () async {
    final adapter = FakeHttpAdapter(
      (_) => jsonTestResponse([sampleDemandJson()]),
    );
    final api = DemandsApi(createTestDio(adapter));

    final result = await api.listDemands(
      academicCenterId: caId,
      forum: 'physical_space',
      status: 'open',
      mine: true,
      limit: 20,
      offset: 0,
    );

    final request = adapter.requests.single;
    expect(request.method, 'GET');
    expect(request.path, '/api/v1/academic-centers/${caId.uuid}/demands');
    expect(request.queryParameters, {
      'forum': 'physical_space',
      'status': 'open',
      'mine': true,
      'limit': 20,
      'offset': 0,
    });
    expect(request.extra['firebaseAuth'], isTrue);
    expect(request.extra['firebaseAppCheck'], isTrue);
    expect(result, hasLength(1));
    expect(result.single.id, demandId);
    expect(result.single.forum, 'physical_space');
  });

  test('createDemand envia payload e exige HTTP 201', () async {
    final adapter = FakeHttpAdapter(
      (_) => jsonTestResponse(
        sampleDemandJson(
          forum: 'institutional',
          courseName: 'Ciência da Computação',
        ),
        status: 201,
      ),
    );
    final api = DemandsApi(createTestDio(adapter));

    final created = await api.createDemand(
      academicCenterId: caId,
      request: const CreateDemandRequestDto(
        forum: 'institutional',
        title: 'Revisão de ementa',
        description: 'Solicitação de atualização da ementa.',
        category: 'Pedagógico',
        courseName: 'Ciência da Computação',
      ),
    );

    final request = adapter.requests.single;
    expect(request.method, 'POST');
    expect(request.path, '/api/v1/academic-centers/${caId.uuid}/demands');
    expect(request.data, {
      'forum': 'institutional',
      'description': 'Solicitação de atualização da ementa.',
      'title': 'Revisão de ementa',
      'category': 'Pedagógico',
      'courseName': 'Ciência da Computação',
    });
    expect(created.courseName, 'Ciência da Computação');
  });

  test('getDemand e updateDemandStatus desserializam statusHistory', () async {
    final historyJson = [
      {
        'id': historyId.uuid,
        'demandId': demandId.uuid,
        'fromStatus': 'open',
        'toStatus': 'in_analysis',
        'changedById': authorId.uuid,
        'comment': 'Em triagem pela diretoria',
        'opinion': 'Em triagem pela diretoria',
        'createdAt': '2026-11-20T13:00:00Z',
      },
    ];
    final adapter = FakeHttpAdapter(
      (options) => jsonTestResponse(
        sampleDemandJson(status: 'in_analysis', statusHistory: historyJson),
      ),
    );
    final api = DemandsApi(createTestDio(adapter));

    final detail = await api.getDemand(
      academicCenterId: caId,
      demandId: demandId,
    );
    expect(detail.statusHistory, hasLength(1));
    expect(detail.statusHistory!.single.toStatus, 'in_analysis');

    final updated = await api.updateDemandStatus(
      academicCenterId: caId,
      demandId: demandId,
      request: const UpdateDemandStatusRequestDto(
        status: 'in_analysis',
        opinion: 'Em triagem pela diretoria',
      ),
    );
    expect(adapter.requests.last.method, 'PATCH');
    expect(
      adapter.requests.last.path,
      '/api/v1/academic-centers/${caId.uuid}/demands/${demandId.uuid}/status',
    );
    expect(updated.status, 'in_analysis');
  });

  test(
    'listComments e createComment respeitam contrato de comentários',
    () async {
    final commentJson = {
      'id': commentId.uuid,
      'demandId': demandId.uuid,
      'authorId': authorId.uuid,
      'authorRole': 'ca_management',
      'content': 'Encaminhamos para manutenção.',
      'createdAt': '2026-11-20T14:00:00Z',
    };
    final adapter = FakeHttpAdapter((options) {
      if (options.method == 'GET') {
        return jsonTestResponse([commentJson]);
      }
      return jsonTestResponse(commentJson, status: 201);
    });
    final api = DemandsApi(createTestDio(adapter));

    final comments = await api.listComments(
      academicCenterId: caId,
      demandId: demandId,
    );
    expect(comments.single.authorRole, 'ca_management');

    final posted = await api.createComment(
      academicCenterId: caId,
      demandId: demandId,
      request: const CreateDemandCommentRequestDto(
        content: 'Encaminhamos para manutenção.',
      ),
    );
    expect(posted.id, commentId);
    expect(adapter.requests.last.method, 'POST');
  });

  test('rejeita resposta incompatível com FormatException', () async {
    final api = DemandsApi(
      createTestDio(
        FakeHttpAdapter((_) => jsonTestResponse({'invalid': true})),
      ),
    );

    await expectLater(
      api.listDemands(academicCenterId: caId),
      throwsFormatException,
    );
  });

  test('propaga DioException em status de erro', () async {
    final api = DemandsApi(
      createTestDio(
        FakeHttpAdapter(
          (_) => jsonTestResponse(
            backendErrorBody('user_not_permitted_in_ca'),
            status: 403,
          ),
        ),
      ),
    );

    await expectLater(
      api.listDemands(academicCenterId: caId, forum: 'institutional'),
      throwsA(
        isA<DioException>().having(
          (e) => e.response?.statusCode,
          'status',
          403,
        ),
      ),
    );
  });
}
