/// Testes do adaptador que traduz HTTP para a API de identidade.
///
/// A tradução de erros é responsabilidade do gateway atual, que substituiu
/// o interceptor de erros previsto na estrutura inicial dos testes.
/// Usa Dio e desserialização reais sobre um transporte em memória.
library;

import 'dart:async';

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:json_annotation/json_annotation.dart';
import 'package:uuid/uuid.dart';
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
  test('restauração sem conta distingue primeiro acesso de recusa', () async {
    final gateway = _gateway((options) {
      expect(options.method, 'GET');
      expect(options.path, '/auth/session');

      return jsonTestResponse(
        backendErrorBody('account_not_provisioned'),
        status: 404,
      );
    });

    await expectLater(
      gateway.restoreSession(),
      throwsA(
        isA<IdentityException>()
            .having(
              (e) => e.kind,
              'categoria',
              IdentityFailureKind.accountNotProvisioned,
            )
            .having((e) => e.backendCode, 'código', 'account_not_provisioned'),
      ),
    );
  });

  test('404 genérico não é interpretado como conta não provisionada', () async {
    final gateway = _gateway(
      (_) => jsonTestResponse(backendErrorBody('route_not_found'), status: 404),
    );

    await expectLater(
      gateway.restoreSession(),
      throwsA(
        isA<IdentityException>().having(
          (e) => e.kind,
          'categoria',
          isNot(IdentityFailureKind.accountNotProvisioned),
        ),
      ),
    );
  });

  test(
    'perfil semanticamente inválido é rejeitado ao converter o DTO',
    () async {
      final body = backendInstitutionalAuthBody();

      (body['user'] as Map)['status'] = 'active';

      final gateway = _gateway((_) => jsonTestResponse(body));

      await expectLater(
        gateway.verify(microsoftAccessToken: 'microsoft-test-token'),
        throwsA(
          isA<IdentityException>().having(
            (e) => e.kind,
            'categoria',
            IdentityFailureKind.invalidResponse,
          ),
        ),
      );
    },
  );

  for (final scenario in [
    (
      'non-student',
      '00000000-0000-0000-0000-000000000001',
      'Pessoa Exemplo',
      null,
      const NonStudentAffiliation(),
      'IEX',
    ),
    (
      'student',
      '00000000-0000-0000-0000-000000000001',
      'Pessoa Exemplo',
      'pessoa@instituicao.example',
      const StudentAffiliation(courseName: 'Engenharia de Software'),
      'IEX',
    ),
    (
      'student-null-fields',
      '00000000-0000-0000-0000-000000000002',
      null,
      null,
      const StudentAffiliation(courseName: null),
      null,
    ),
  ]) {
    test('converte todos os campos do exemplo ${scenario.$1}', () async {
      final body = loadApiContractExample(
        'institutional-verification-${scenario.$1}.json',
      );
      final gateway = _gateway((options) {
        expect(options.path, '/auth/institutional/verify');
        expect(
          (options.data as Map)['microsoftAccessToken'],
          'microsoft-test-token',
        );

        return jsonTestResponse(body);
      });

      final result = await gateway.verify(
        microsoftAccessToken: 'microsoft-test-token',
      );
      final proof = result.verification!;

      expect(
        result.user.id,
        const UuidValue.raw('00000000-0000-0000-0000-000000000010'),
      );
      expect(result.user.status, InternalUserStatus.incomplete);
      expect(proof.profile.microsoftObjectId, UuidValue.raw(scenario.$2));
      expect(proof.profile.name, scenario.$3);
      expect(proof.profile.email, scenario.$4);
      expect(proof.profile.affiliation, scenario.$5);
      expect(
        proof.institution,
        InstitutionSummary(
          id: const UuidValue.raw('00000000-0000-0000-0000-000000000002'),
          name: 'Instituição Exemplo',
          acronym: scenario.$6,
        ),
      );
      expect(proof.expiresAt, DateTime.utc(2026, 10, 22, 12));
    });
  }

  test('preserva instituição sem sigla no perfil estudantil', () async {
    final gateway = _gateway(
      (_) => jsonTestResponse(
        backendInstitutionalAuthBody(student: true, institutionAcronym: null),
      ),
    );
    final result = await gateway.verify(
      microsoftAccessToken: 'microsoft-test-token',
    );

    expect(
      result.verification!.profile.affiliation,
      const StudentAffiliation(courseName: null),
    );
    expect(result.verification!.institution.name, 'Instituição Exemplo');
    expect(result.verification!.institution.acronym, isNull);
  });

  for (final field in [
    ('profile', 'microsoftObjectId'),
    ('institution', 'id'),
  ]) {
    test(
      'UUID inválido em ${field.$1}.${field.$2} é resposta inválida',
      () async {
        final body = backendInstitutionalAuthBody();

        ((body['verification'] as Map)[field.$1] as Map)[field.$2] =
            'not-a-uuid';

        final gateway = _gateway((_) => jsonTestResponse(body));

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
                  'causa',
                  isA<CheckedFromJsonException>(),
                ),
          ),
        );
      },
    );
  }

  for (final scenario in [
    (403, 'app_check_required', IdentityFailureKind.appVerificationFailed),
    (403, 'app_check_invalid', IdentityFailureKind.appVerificationFailed),
    (503, 'app_check_unavailable', IdentityFailureKind.serviceUnavailable),
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
