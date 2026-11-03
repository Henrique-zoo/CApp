/// Testes do interceptor de credenciais HTTP, sem SDK Firebase.
///
/// Exercita o transporte real de Dio em memória, incluindo consulta por
/// requisição, proteção optativa e preservação de falhas do provedor de token.
library;

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:capp/src/infrastructure/backend/auth/firebase_bearer_interceptor.dart';

import '../../../../support/http_fakes.dart';

/// Registra o contrato de autenticação das requisições.
void main() {
  test('consulta o token atual para cada requisição protegida', () async {
    final adapter = FakeHttpAdapter((_) => jsonTestResponse({}));
    final dio = createTestDio(adapter);
    var token = 'first-test-token';
    var calls = 0;
    dio.interceptors.add(
      FirebaseBearerInterceptor(() async {
        calls++;
        return token;
      }),
    );
    await dio.get<Object?>(
      '/private',
      options: Options(extra: const {'firebaseAuth': true}),
    );
    token = 'second-test-token';
    await dio.get<Object?>(
      '/private',
      options: Options(extra: const {'firebaseAuth': true}),
    );
    expect(calls, 2);
    expect(
      adapter.requests[0].headers['Authorization'],
      'Bearer first-test-token',
    );
    expect(
      adapter.requests[1].headers['Authorization'],
      'Bearer second-test-token',
    );
  });

  test('requisição pública não consulta credenciais', () async {
    final adapter = FakeHttpAdapter((_) => jsonTestResponse({}));
    final dio = createTestDio(adapter);
    dio.interceptors.add(
      FirebaseBearerInterceptor(
        () async => throw StateError('O provedor não deveria ser consultado'),
      ),
    );
    await dio.get<Object?>('/public');
    expect(
      adapter.requests.single.headers.containsKey('Authorization'),
      isFalse,
    );
  });

  for (final token in <String?>[null, '']) {
    final description = token == null ? 'nulo' : 'vazio';
    test('token $description rejeita antes de alcançar o transporte', () async {
      final adapter = FakeHttpAdapter((_) => jsonTestResponse({}));
      final dio = createTestDio(adapter);
      dio.interceptors.add(FirebaseBearerInterceptor(() async => token));
      await expectLater(
        dio.get<Object?>(
          '/private',
          options: Options(extra: const {'firebaseAuth': true}),
        ),
        throwsA(
          isA<DioException>().having(
            (e) => e.error,
            'causa',
            isA<MissingFirebaseIdTokenException>(),
          ),
        ),
      );
      expect(adapter.requests, isEmpty);
    });
  }

  test('falha ao obter token preserva causa e pilha originais', () async {
    final adapter = FakeHttpAdapter((_) => jsonTestResponse({}));
    final dio = createTestDio(adapter);
    final cause = Exception('provedor indisponível');
    final stack = StackTrace.fromString('token-provider-test-trace');
    dio.interceptors.add(
      FirebaseBearerInterceptor(() => Future.error(cause, stack)),
    );
    await expectLater(
      dio.get<Object?>(
        '/private',
        options: Options(extra: const {'firebaseAuth': true}),
      ),
      throwsA(
        isA<DioException>()
            .having((e) => e.error, 'causa original', same(cause))
            .having(
              (e) => e.stackTrace.toString(),
              'pilha original',
              stack.toString(),
            ),
      ),
    );
    expect(adapter.requests, isEmpty);
  });
}
