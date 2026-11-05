/// Testes do contrato HTTP institucional antes da tradução para a aplicação.
///
/// Mantém o transporte e a desserialização de Dio sobre respostas em memória.
/// O cliente devolve DTOs e não classifica falhas de negócio da sessão.
library;

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:json_annotation/json_annotation.dart';
import 'package:uuid/uuid.dart';
import 'package:capp/src/infrastructure/backend/identity/dto/institutional_auth_request.dart';
import 'package:capp/src/infrastructure/backend/identity/dto/institutional_affiliation.dart';
import 'package:capp/src/infrastructure/backend/identity/identity_api.dart';

import '../../../../support/http_fakes.dart';

/// Requisição fictícia reutilizada nas verificações do protocolo.
InstitutionalAuthRequest _request() => InstitutionalAuthRequest.fromJson(
  loadApiContractExample('institutional-auth-request.json'),
);

/// Registra autenticação optativa, status e formatos de resposta.
void main() {
  test(
    'envia a credencial no JSON e solicita autenticação ao interceptor',
    () async {
      final adapter = FakeHttpAdapter(
        (_) => jsonTestResponse(backendInstitutionalAuthBody()),
      );
      final api = IdentityApi(createTestDio(adapter));
      final response = await api.verifyInstitutionalIdentity(
        request: _request(),
      );
      final request = adapter.requests.single;

      expect(request.method, 'POST');
      expect(request.path, '/auth/institutional/verify');
      expect(request.extra['firebaseAuth'], isTrue);
      expect(request.contentType, Headers.jsonContentType);
      expect(
        request.data,
        loadApiContractExample('institutional-auth-request.json'),
      );
      expect(
        response.verification.profile.affiliation,
        isA<NonStudentAffiliation>(),
      );
      expect(response.verification.profile.email, isNull);
      expect(
        response.verification.profile.microsoftObjectId,
        const UuidValue.raw('00000000-0000-0000-0000-000000000001'),
      );
      expect(
        response.verification.institution.id,
        const UuidValue.raw('00000000-0000-0000-0000-000000000002'),
      );
      expect(response.verification.institution.name, 'Instituição Exemplo');
      expect(response.verification.institution.acronym, 'IEX');
      expect(response.verification.toJson(), {
        ...backendVerificationBody(),
        'expiresAt': '2026-10-22T12:00:00.000Z',
      });
    },
  );

  test(
    'desserializa a resposta compartilhada com conta e curso informado',
    () async {
      final body = loadApiContractExample(
        'institutional-verification-student.json',
      );
      final api = IdentityApi(
        createTestDio(FakeHttpAdapter((_) => jsonTestResponse(body))),
      );

      final response = await api.verifyInstitutionalIdentity(
        request: _request(),
      );

      expect(
        response.user.id,
        const UuidValue.raw('00000000-0000-0000-0000-000000000010'),
      );
      expect(response.user.status, 'incomplete');
      expect(
        response.verification.profile.affiliation,
        isA<StudentAffiliation>().having(
          (affiliation) => affiliation.courseName,
          'curso',
          'Engenharia de Software',
        ),
      );
    },
  );

  test('status diferente de 200 não é aceito como sucesso', () async {
    final adapter = FakeHttpAdapter(
      (_) => jsonTestResponse(backendInstitutionalAuthBody(), status: 201),
    );
    final api = IdentityApi(createTestDio(adapter));

    await expectLater(
      api.verifyInstitutionalIdentity(request: _request()),
      throwsA(
        isA<DioException>().having(
          (e) => e.response?.statusCode,
          'status preservado',
          201,
        ),
      ),
    );
  });

  test('sucesso com raiz incompatível preserva uma falha explícita', () async {
    final adapter = FakeHttpAdapter((_) => jsonTestResponse(<Object?>[]));
    final api = IdentityApi(createTestDio(adapter));

    await expectLater(
      api.verifyInstitutionalIdentity(request: _request()),
      throwsFormatException,
    );
  });

  test(
    'campo obrigatório ausente falha pela desserialização checked',
    () async {
      final body = backendInstitutionalAuthBody();

      (body['verification'] as Map).remove('profile');

      final api = IdentityApi(
        createTestDio(FakeHttpAdapter((_) => jsonTestResponse(body))),
      );

      await expectLater(
        api.verifyInstitutionalIdentity(request: _request()),
        throwsA(isA<CheckedFromJsonException>()),
      );
    },
  );

  test('resposta de erro fica disponível para o adaptador do módulo', () async {
    final body = backendErrorBody('unknown_institution');
    final api = IdentityApi(
      createTestDio(
        FakeHttpAdapter((_) => jsonTestResponse(body, status: 403)),
      ),
    );

    await expectLater(
      api.verifyInstitutionalIdentity(request: _request()),
      throwsA(
        isA<DioException>()
            .having((e) => e.response?.statusCode, 'status', 403)
            .having((e) => e.response?.data, 'corpo original', body),
      ),
    );
  });

  test('instituição sem sigla preserva a propriedade nula no JSON', () async {
    final body = backendInstitutionalAuthBody(institutionAcronym: null);
    final api = IdentityApi(
      createTestDio(FakeHttpAdapter((_) => jsonTestResponse(body))),
    );
    final response = await api.verifyInstitutionalIdentity(request: _request());

    expect(response.verification.institution.acronym, isNull);
    expect(response.verification.toJson(), {
      ...(body['verification'] as Map),
      'expiresAt': '2026-10-22T12:00:00.000Z',
    });
  });

  for (final field in [
    ('profile', 'microsoftObjectId'),
    ('institution', 'id'),
  ]) {
    for (final value in <Object?>['not-a-uuid', null, 42]) {
      test(
        '${field.$1}.${field.$2} incompatível falha na leitura checked: $value',
        () async {
          final body = backendInstitutionalAuthBody();

          ((body['verification'] as Map)[field.$1] as Map)[field.$2] = value;

          final api = IdentityApi(
            createTestDio(FakeHttpAdapter((_) => jsonTestResponse(body))),
          );

          await expectLater(
            api.verifyInstitutionalIdentity(request: _request()),
            throwsA(isA<CheckedFromJsonException>()),
          );
        },
      );
    }
  }

  test('instituição é obrigatória na resposta institucional', () async {
    final body = backendInstitutionalAuthBody();

    (body['verification'] as Map).remove('institution');

    final api = IdentityApi(
      createTestDio(FakeHttpAdapter((_) => jsonTestResponse(body))),
    );

    await expectLater(
      api.verifyInstitutionalIdentity(request: _request()),
      throwsA(isA<CheckedFromJsonException>()),
    );
  });

  test('sigla ausente difere de sigla explicitamente nula', () async {
    final body = backendInstitutionalAuthBody();

    ((body['verification'] as Map)['institution'] as Map).remove('acronym');

    final api = IdentityApi(
      createTestDio(FakeHttpAdapter((_) => jsonTestResponse(body))),
    );

    await expectLater(
      api.verifyInstitutionalIdentity(request: _request()),
      throwsA(isA<CheckedFromJsonException>()),
    );
  });

  for (final field in ['user', 'verification']) {
    test('verificação concluída exige $field na resposta', () async {
      final body = backendInstitutionalAuthBody();

      body[field] = null;

      final api = IdentityApi(
        createTestDio(FakeHttpAdapter((_) => jsonTestResponse(body))),
      );

      await expectLater(
        api.verifyInstitutionalIdentity(request: _request()),
        throwsFormatException,
      );
    });
  }
}
