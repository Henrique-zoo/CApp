import 'dart:io';

import 'package:capp/src/infrastructure/backend/backend_api.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';

/// Exercita o transporte do aplicativo com os servidores de test_backend_tls.py.
/// A mesma verificação roda no Flutter de host e no APK de integração.
void main() {
  if (!const bool.hasEnvironment('TLS_VALID')) {
    test(
      'transporte do backend com certificados temporários',
      () {},
      skip: 'Execute python tool/test_backend_tls.py para preparar os certificados.',
    );
    return;
  }

  BackendApi connect(
    String url, {
    String ca = const String.fromEnvironment('CAPP_DEMO_CA_PEM_BASE64'),
  }) {
    final backend = BackendApi(
      baseUrl: url,
      demoCaPemBase64: ca,
      getFirebaseIdToken: () async => 'test-id',
      getFirebaseAppCheckToken: () async => 'test-check',
    );
    addTearDown(() => backend.close(force: true));
    return backend;
  }

  test(
    'confia na CA da demonstração e valida o certificado do servidor',
    () async {
      await connect(const String.fromEnvironment('TLS_VALID')).monitoring
          .getReadiness();
    },
  );

  for (final (label, url) in [
    ('CA desconhecida', const String.fromEnvironment('TLS_UNKNOWN')),
    ('certificado expirado', const String.fromEnvironment('TLS_EXPIRED')),
    (
      'nome de servidor incorreto',
      const String.fromEnvironment('TLS_MISMATCH'),
    ),
  ]) {
    test('interrompe o handshake com $label', () async {
      final backend = connect(url);
      await expectLater(
        backend.identity.restoreSession(),
        throwsA(
          isA<DioException>().having(
            (e) => e.error,
            'TLS',
            isA<HandshakeException>(),
          ),
        ),
      );
    });
  }

  test('recusa HTTP direto e redirecionamento de uma operação autenticada para HTTP', () async {
    expect(
      () => connect(const String.fromEnvironment('TLS_CLEAR')),
      throwsArgumentError,
    );
    final backend = connect(const String.fromEnvironment('TLS_VALID'));
    await expectLater(
      backend.identity.restoreSession(),
      throwsA(
        isA<DioException>().having(
          (e) => e.response?.statusCode,
          'status',
          307,
        ),
      ),
    );
  });

  const caddyUrl = String.fromEnvironment('TLS_CADDY');
  if (caddyUrl.isNotEmpty) {
    test('alcança o backend da demonstração através do Caddy', () async {
      await connect(
        caddyUrl,
        ca: const String.fromEnvironment('TLS_CADDY_CA'),
      ).monitoring.getReadiness();
    });
  }
}
