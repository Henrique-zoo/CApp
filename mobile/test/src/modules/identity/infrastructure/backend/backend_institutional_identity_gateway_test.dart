/// Testes do adaptador que traduz HTTP para a API de identidade.
///
/// A tradução de erros é responsabilidade do gateway atual, que substituiu
/// o interceptor de erros previsto na estrutura inicial dos testes.
/// Usa Dio e desserialização reais sobre um transporte em memória.
library;

import 'dart:async';

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:capp/src/infrastructure/backend/identity/identity_api.dart';
import 'package:capp/src/modules/identity/identity.dart';
import 'package:capp/src/modules/identity/infrastructure/backend/backend_institutional_identity_gateway.dart';

import '../../../../../support/http_fakes.dart';

/// Conecta o adaptador do módulo ao cliente HTTP real, sem conexões externas.
BackendInstitutionalIdentityGateway _gateway(
  FutureOr<ResponseBody> Function(RequestOptions options) respond,
) => BackendInstitutionalIdentityGateway(
  IdentityApi(createTestDio(FakeHttpAdapter(respond))),
);

/// Registra conversões e preservação de falhas na fronteira do módulo.
void main() {
  test('não estudante é sucesso com modelos próprios', () async {
    final gateway = _gateway((options) {
      expect(options.path, '/auth/institutional/verify');
      expect(
        (options.data as Map)['microsoftAccessToken'],
        'microsoft-test-token',
      );
      return jsonTestResponse(backendVerificationBody());
    });
    final result = await gateway.verify(
      microsoftAccessToken: 'microsoft-test-token',
    );
    expect(result.profile.affiliation, const NonStudentAffiliation());
    expect(result.profile.name, 'Pessoa Exemplo');
    expect(result.expiresAt, DateTime.utc(2026, 1, 1, 13));
  });

  for (final scenario in [
    (401, 'expired_token', IdentityFailureKind.credentialsRejected),
    (403, 'unknown_institution', IdentityFailureKind.institutionNotSupported),
    (
      403,
      'institutional_verification_required',
      IdentityFailureKind.verificationConflict,
    ),
    (503, 'service_unavailable', IdentityFailureKind.serviceUnavailable),
    (503, 'new_backend_code', IdentityFailureKind.serviceUnavailable),
  ]) {
    test('HTTP ${scenario.$1}: ${scenario.$2}', () async {
      final gateway = _gateway(
        (_) => jsonTestResponse(
          backendErrorBody(scenario.$2),
          status: scenario.$1,
        ),
      );
      await expectLater(
        gateway.verify(microsoftAccessToken: 'microsoft-test-token'),
        throwsA(
          isA<IdentityException>()
              .having((e) => e.kind, 'classificação', scenario.$3)
              .having((e) => e.backendCode, 'código preservado', scenario.$2)
              .having(
                (e) => e.publicMessage,
                'mensagem pública',
                'Mensagem pública do backend',
              )
              .having(
                (e) => e.cause,
                'transporte preservado',
                isA<DioException>(),
              ),
        ),
      );
    });
  }

  test('sucesso com corpo incompatível gera falha explícita', () async {
    final gateway = _gateway((_) => jsonTestResponse(<Object?>[]));
    await expectLater(
      gateway.verify(microsoftAccessToken: 'microsoft-test-token'),
      throwsA(
        isA<IdentityException>()
            .having(
              (e) => e.kind,
              'classificação',
              IdentityFailureKind.invalidResponse,
            )
            .having((e) => e.cause, 'causa', isA<FormatException>()),
      ),
    );
  });

  test('envelope malformado preserva transporte e conversão', () async {
    final gateway = _gateway(
      (_) => jsonTestResponse({
        'error': {'code': 42},
      }, status: 503),
    );
    await expectLater(
      gateway.verify(microsoftAccessToken: 'microsoft-test-token'),
      throwsA(
        isA<IdentityException>()
            .having(
              (e) => e.kind,
              'classificação',
              IdentityFailureKind.invalidResponse,
            )
            .having(
              (e) => e.cause,
              'diagnósticos',
              isA<
                ({
                  DioException transport,
                  Object decoding,
                  StackTrace decodingStackTrace,
                })
              >(),
            ),
      ),
    );
  });

  test('erro de programação encapsulado não vira indisponibilidade', () async {
    final cause = StateError('erro interno simulado');
    final gateway = _gateway((_) => throw cause);
    await expectLater(
      gateway.verify(microsoftAccessToken: 'microsoft-test-token'),
      throwsA(same(cause)),
    );
  });

  test('cancelamento HTTP é distinto do cancelamento do login', () async {
    final gateway = _gateway(
      (options) => throw DioException.requestCancelled(
        requestOptions: options,
        reason: 'cancelamento simulado',
      ),
    );
    await expectLater(
      gateway.verify(microsoftAccessToken: 'microsoft-test-token'),
      throwsA(
        isA<IdentityException>().having(
          (e) => e.kind,
          'classificação',
          IdentityFailureKind.operationCancelled,
        ),
      ),
    );
  });
}
