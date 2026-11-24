/// Testes da tradução de protocolo e falhas no [BackendFaqGateway].
library;

import 'package:capp/src/infrastructure/backend/faq/faq.dart';
import 'package:capp/src/modules/faq/faq.dart';
import 'package:capp/src/modules/faq/infrastructure/backend_faq_gateway.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

import '../../../../support/http_fakes.dart';

void main() {
  const caId = UuidValue.raw('00000000-0000-0000-0000-000000000001');
  const entryId = UuidValue.raw('00000000-0000-0000-0000-000000000010');

  test('traduz códigos de erro do backend para FaqException', () async {
    final cases = <(String code, int status, FaqFailureKind expected)>[
      ('user_not_permitted_in_ca', 403, FaqFailureKind.forbidden),
      ('academic_center_not_found', 404, FaqFailureKind.academicCenterNotFound),
      ('academic_center_inactive', 422, FaqFailureKind.academicCenterInactive),
      ('faq_entry_not_found', 404, FaqFailureKind.entryNotFound),
      ('empty_question', 422, FaqFailureKind.validationError),
      ('app_check_invalid', 403, FaqFailureKind.appVerificationFailed),
      ('unauthorized', 401, FaqFailureKind.unauthorized),
      ('service_unavailable', 503, FaqFailureKind.serviceUnavailable),
    ];

    for (final testCase in cases) {
      final api = FaqApi(
        createTestDio(
          FakeHttpAdapter(
            (_) => jsonTestResponse(
              backendErrorBody(testCase.$1),
              status: testCase.$2,
            ),
          ),
        ),
      );
      final gateway = BackendFaqGateway(api);

      await expectLater(
        gateway.listEntries(academicCenterId: caId),
        throwsA(
          isA<FaqException>()
              .having((e) => e.kind, 'kind', testCase.$3)
              .having((e) => e.backendCode, 'backendCode', testCase.$1)
              .having(
                (e) => e.publicMessage,
                'publicMessage',
                'Mensagem pública do backend',
              ),
        ),
      );
    }
  });

  test('traduz falha de conexão Dio para communicationFailed', () async {
    final api = FaqApi(
      createTestDio(
        FakeHttpAdapter(
          (options) => throw DioException(
            requestOptions: options,
            type: DioExceptionType.connectionError,
          ),
        ),
      ),
    );
    final gateway = BackendFaqGateway(api);

    await expectLater(
      gateway.deleteEntry(academicCenterId: caId, entryId: entryId),
      throwsA(
        isA<FaqException>().having(
          (e) => e.kind,
          'kind',
          FaqFailureKind.communicationFailed,
        ),
      ),
    );
  });
}
