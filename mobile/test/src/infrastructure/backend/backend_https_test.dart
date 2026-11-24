import 'package:capp/src/infrastructure/backend/backend_api.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../../support/http_fakes.dart';

void main() {
  const server = 'https://backend.example.invalid';
  late FakeHttpAdapter adapter;
  late Dio dio;
  var tokenRequests = 0;

  Future<String?> token() async {
    tokenRequests++;
    return 'token-de-teste';
  }

  BackendApi backend(String url) => BackendApi(
    baseUrl: url,
    getFirebaseIdToken: token,
    getFirebaseAppCheckToken: token,
    dio: dio,
  );

  setUp(() {
    tokenRequests = 0;
    adapter = FakeHttpAdapter(
      (_) => ResponseBody.fromString(
        '',
        302,
        headers: {
          'location': ['http://elsewhere.example.invalid/'],
        },
      ),
    );
    dio = createTestDio(adapter);
  });

  test('a URL HTTP é rejeitada antes da consulta de tokens e da rede', () {
    expect(() => backend('http://10.0.2.2:8443'), throwsArgumentError);
    expect(tokenRequests, 0);
    expect(adapter.requests, isEmpty);
  });

  test('destino absoluto fora da origem não recebe as credenciais', () async {
    backend(server);
    for (final target in [
      'http://backend.example.invalid/auth/session',
      'https://another.example.invalid/auth/session',
      'https://backend.example.invalid:444/auth/session',
    ]) {
      await expectLater(
        dio.get<void>(
          target,
          options: Options(
            extra: {'firebaseAuth': true, 'firebaseAppCheck': true},
          ),
        ),
        throwsA(
          isA<DioException>().having(
            (e) => e.error,
            'causa',
            isA<FormatException>(),
          ),
        ),
      );
    }
    expect(tokenRequests, 0);
    expect(adapter.requests, isEmpty);
  });

  test(
    'o transporte autenticado devolve 302 e bloqueia o redirecionamento',
    () async {
      final api = backend(server);
      await expectLater(
        api.identity.restoreSession(),
        throwsA(
          isA<DioException>().having(
            (e) => e.response?.statusCode,
            'status',
            302,
          ),
        ),
      );
      final request = adapter.requests.single;
      expect(request.uri.origin, server);
      expect(request.followRedirects, isFalse);
      expect(request.headers['Authorization'], 'Bearer token-de-teste');
      expect(request.headers['X-Firebase-AppCheck'], 'token-de-teste');
    },
  );
}
