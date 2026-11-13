/// App Check é enviado junto com a identidade, sem permitir envio sem atestado.
///
/// Usa Dio e as fachadas reais, com rede em memória. Verifica obtenção por
/// requisição, ausência de token, preservação de falhas e sondagens públicas.
library;

import 'package:capp/src/infrastructure/backend/auth/firebase_app_check_interceptor.dart';
import 'package:capp/src/infrastructure/backend/backend_api.dart';
import 'package:capp/src/infrastructure/backend/identity/dto/institutional_auth_request.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../../../support/http_fakes.dart';

void main() {
  test(
    'operações de identidade enviam App Check e Bearer separadamente',
    () async {
      final user = {
        'id': '00000000-0000-0000-0000-000000000010',
        'status': 'active',
        'nickname': 'Pessoa',
        'email': 'pessoal@example.test',
        'currentSemester': 4,
        'institutionEntrySemester': '2023.2',
      };
      final adapter = FakeHttpAdapter(
        (request) => jsonTestResponse(switch (request.path) {
          '/auth/institutional/verify' => backendInstitutionalAuthBody(),
          '/auth/session' => {'user': user, 'verification': null},
          _ => user,
        }),
      );
      var appToken = 'app-token-1';
      var appCalls = 0;
      final api = BackendApi(
        baseUrl: 'https://backend.example.invalid',
        getFirebaseIdToken: () async => 'user-id-token',
        getFirebaseAppCheckToken: () async {
          appCalls++;

          return appToken;
        },
        dio: createTestDio(adapter),
      );

      addTearDown(() => api.close(force: true));
      await api.identity.restoreSession();

      appToken = 'app-token-2';

      await api.identity.verifyInstitutionalIdentity(
        request: InstitutionalAuthRequest(microsoftAccessToken: 'graph-token'),
      );

      await api.identity.completeProfile(
        nickname: 'Pessoa',
        currentSemester: 4,
        institutionEntrySemester: '2023.2',
      );

      expect(appCalls, 3);
      expect(
        adapter.requests.first.headers['X-Firebase-AppCheck'],
        'app-token-1',
      );

      for (final request in adapter.requests) {
        expect(request.headers['Authorization'], 'Bearer user-id-token');
        expect(request.headers['X-Firebase-AppCheck'], isNotEmpty);
      }

      expect(
        adapter.requests.last.headers['X-Firebase-AppCheck'],
        'app-token-2',
      );
    },
  );

  test('requisição pública não solicita App Check', () async {
    final adapter = FakeHttpAdapter((_) => jsonTestResponse({}));
    final dio = createTestDio(adapter);

    dio.interceptors.add(
      FirebaseAppCheckInterceptor(
        () async => throw StateError('Não deve consultar o SDK'),
      ),
    );

    await dio.get<Object?>('/public');

    expect(
      adapter.requests.single.headers.containsKey('X-Firebase-AppCheck'),
      isFalse,
    );
  });

  for (final token in <String?>[null, '']) {
    test('token ausente ($token) impede o envio ao backend', () async {
      final adapter = FakeHttpAdapter((_) => jsonTestResponse({}));
      final dio = createTestDio(adapter)
        ..interceptors.add(FirebaseAppCheckInterceptor(() async => token));

      await expectLater(
        dio.get<Object?>(
          '/private',
          options: Options(extra: const {'firebaseAppCheck': true}),
        ),
        throwsA(
          isA<DioException>().having(
            (e) => e.error,
            'causa',
            isA<FirebaseAppCheckTokenException>(),
          ),
        ),
      );

      expect(adapter.requests, isEmpty);
    });
  }

  test('falha do SDK conserva causa e não envia a requisição', () async {
    final cause = Exception('SDK indisponível');
    final adapter = FakeHttpAdapter((_) => jsonTestResponse({}));
    final dio = createTestDio(adapter)
      ..interceptors.add(FirebaseAppCheckInterceptor(() async => throw cause));

    await expectLater(
      dio.get<Object?>(
        '/private',
        options: Options(extra: const {'firebaseAppCheck': true}),
      ),
      throwsA(
        isA<DioException>().having(
          (e) => (e.error as FirebaseAppCheckTokenException).cause,
          'causa',
          same(cause),
        ),
      ),
    );

    expect(adapter.requests, isEmpty);
  });
}
