/// Testes de tradução de DTOs e falhas HTTP em [BackendDemandsGateway].
library;

import 'package:capp/src/infrastructure/backend/demands/demands.dart';
import 'package:capp/src/modules/demands/demands.dart';
import 'package:capp/src/modules/demands/infrastructure/backend/backend_demands_gateway.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

import '../../../../../support/http_fakes.dart';

void main() {
  const caId = UuidValue.raw('00000000-0000-0000-0000-000000000001');
  const authorId = UuidValue.raw('00000000-0000-0000-0000-000000000010');
  const demandId = UuidValue.raw('00000000-0000-0000-0000-000000000020');

  test('converte demanda e histórico de status do protocolo', () async {
    final adapter = FakeHttpAdapter(
      (_) => jsonTestResponse({
        'id': demandId.uuid,
        'academicCenterId': caId.uuid,
        'authorId': authorId.uuid,
        'forum': 'institutional',
        'title': 'Ajuste de pré-requisito',
        'description': 'Solicitação coletiva do curso.',
        'category': 'Pedagógico',
        'courseName': 'Ciência da Computação',
        'status': 'forwarded_to_coordination',
        'createdAt': '2026-11-20T12:00:00Z',
        'updatedAt': '2026-11-20T15:00:00Z',
        'statusHistory': [
          {
            'id': '00000000-0000-0000-0000-000000000030',
            'demandId': demandId.uuid,
            'fromStatus': 'open',
            'toStatus': 'forwarded_to_coordination',
            'changedById': authorId.uuid,
            'opinion': 'Encaminhado à coordenação',
            'createdAt': '2026-11-20T15:00:00Z',
          },
        ],
      }),
    );
    final gateway = BackendDemandsGateway(DemandsApi(createTestDio(adapter)));

    final demand = await gateway.getDemand(
      academicCenterId: caId,
      demandId: demandId,
    );

    expect(demand.forum, DemandForum.institutional);
    expect(demand.status, DemandStatus.forwardedToCoordination);
    expect(demand.courseName, 'Ciência da Computação');
    expect(demand.statusHistory, hasLength(1));
    expect(
      demand.statusHistory.single.toStatus,
      DemandStatus.forwardedToCoordination,
    );
    expect(demand.statusHistory.single.opinion, 'Encaminhado à coordenação');
  });

  final errorCases = [
    ('user_not_permitted_in_ca', 403, DemandFailureKind.forbiddenCaForum),
    (
      'student_membership_required',
      403,
      DemandFailureKind.studentMembershipRequired,
    ),
    (
      'academic_center_not_found',
      404,
      DemandFailureKind.academicCenterNotFound,
    ),
    ('academic_center_inactive', 422, DemandFailureKind.academicCenterInactive),
    ('demand_not_found', 404, DemandFailureKind.demandNotFound),
    (
      'course_not_represented_by_ca',
      422,
      DemandFailureKind.validationRejected,
    ),
    ('opinion_required', 422, DemandFailureKind.validationRejected),
    ('unauthorized', 401, DemandFailureKind.unauthorized),
    ('service_unavailable', 503, DemandFailureKind.serviceUnavailable),
  ];

  for (final (code, status, expectedKind) in errorCases) {
    test('traduz $code ($status) para $expectedKind', () async {
      final adapter = FakeHttpAdapter(
        (_) => jsonTestResponse(backendErrorBody(code), status: status),
      );
      final gateway = BackendDemandsGateway(DemandsApi(createTestDio(adapter)));

      await expectLater(
        gateway.listDemands(academicCenterId: caId),
        throwsA(
          isA<DemandException>()
              .having((e) => e.kind, 'kind', expectedKind)
              .having((e) => e.backendCode, 'backendCode', code),
        ),
      );
    });
  }

  test('traduz timeout de conexão para communicationFailed', () async {
    final adapter = FakeHttpAdapter(
      (options) => throw DioException(
        requestOptions: options,
        type: DioExceptionType.connectionTimeout,
      ),
    );
    final gateway = BackendDemandsGateway(DemandsApi(createTestDio(adapter)));

    await expectLater(
      gateway.listDemands(academicCenterId: caId),
      throwsA(
        isA<DemandException>().having(
          (e) => e.kind,
          'kind',
          DemandFailureKind.communicationFailed,
        ),
      ),
    );
  });
}
