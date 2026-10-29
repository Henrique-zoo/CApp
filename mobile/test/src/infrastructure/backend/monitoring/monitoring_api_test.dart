/// Contrato HTTP da consulta de prontidão utilizada na abertura do aplicativo.
///
/// Mantém Dio e o interceptor reais para verificar acesso público e rejeição de
/// resultados que não confirmam prontidão, substituindo somente o limite de rede.
library;

import 'package:capp/src/infrastructure/backend/backend_api.dart';
import 'package:capp/src/infrastructure/backend/monitoring/monitoring.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../../../support/http_fakes.dart';

/// Registra sucesso público, indisponibilidade e respostas intermediárias.
void main() {
  test('prontidão usa GET público sem consultar credencial Firebase', () async {
    final adapter = FakeHttpAdapter((_) => ResponseBody.fromString('', 200));
    var tokenCalls = 0;
    final backend = BackendApi(
      baseUrl: 'https://backend.example.invalid',
      getFirebaseIdToken: () async {
        tokenCalls++;
        return null;
      },
      dio: createTestDio(adapter),
    );
    addTearDown(() => backend.close(force: true));

    await backend.monitoring.getReadiness();
    expect(tokenCalls, 0);
    expect(adapter.requests, hasLength(1));
    expect(adapter.requests.single.path, '/ready');
    expect(adapter.requests.single.method, 'GET');
    expect(
      adapter.requests.single.headers.containsKey('Authorization'),
      isFalse,
    );
  });

  for (final status in [503, 204, 302]) {
    test('HTTP $status não confirma prontidão', () async {
      final adapter = FakeHttpAdapter(
        (_) => ResponseBody.fromString('', status),
      );
      final api = MonitoringApi(createTestDio(adapter));
      await expectLater(
        api.getReadiness(),
        throwsA(
          isA<DioException>().having(
            (error) => error.response?.statusCode,
            'status original',
            status,
          ),
        ),
      );
      expect(adapter.requests, hasLength(1));
    });
  }
}
