/// Testes do contrato HTTP institucional antes da tradução para a aplicação.
///
/// Mantém o transporte e a desserialização de Dio sobre respostas em memória.
/// O cliente devolve DTOs e não classifica falhas de negócio da sessão.
library;

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:json_annotation/json_annotation.dart';
import 'package:capp/src/infrastructure/backend/identity/dto/institutional_auth_request.dart';
import 'package:capp/src/infrastructure/backend/identity/dto/institutional_affiliation.dart';
import 'package:capp/src/infrastructure/backend/identity/identity_api.dart';

import '../../../../support/http_fakes.dart';

/// Requisição fictícia reutilizada nas verificações do protocolo.
InstitutionalAuthRequest _request() =>
    InstitutionalAuthRequest(microsoftAccessToken: 'microsoft-test-token');

/// Registra autenticação optativa, status e formatos de resposta.
void main() {
  test(
    'envia a credencial no JSON e solicita autenticação ao interceptor',
    () async {
      final adapter = FakeHttpAdapter(
        (_) => jsonTestResponse(backendVerificationBody()),
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
      expect(request.data, {'microsoftAccessToken': 'microsoft-test-token'});
      expect(response.profile.affiliation, isA<NonStudentAffiliation>());
      expect(response.profile.email, isNull);
    },
  );

  test('status diferente de 200 não é aceito como sucesso', () async {
    final adapter = FakeHttpAdapter(
      (_) => jsonTestResponse(backendVerificationBody(), status: 201),
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
      final body = backendVerificationBody()..remove('profile');
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
}
