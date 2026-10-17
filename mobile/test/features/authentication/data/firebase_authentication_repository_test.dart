import 'package:capp/features/authentication/data/firebase_authentication_repository.dart';
import 'package:capp/features/authentication/domain/authentication_repository.dart';
import 'package:flutter_test/flutter_test.dart';

Future<void> expectConfigurationFailure({
  required String apiBaseUrl,
  required String microsoftTenantId,
}) async {
  final repository = FirebaseAuthenticationRepository(
    config: AuthenticationConfig(
      apiBaseUrl: apiBaseUrl,
      microsoftTenantId: microsoftTenantId,
    ),
  );

  await expectLater(
    repository.signIn(),
    throwsA(
      isA<AuthenticationFailure>().having(
        (failure) => failure.reason,
        'reason',
        AuthenticationFailureReason.configuration,
      ),
    ),
  );
}

void main() {
  group('FirebaseAuthenticationRepository configuration', () {
    test('rejects a missing Microsoft tenant', () async {
      await expectConfigurationFailure(
        apiBaseUrl: 'http://localhost:3000',
        microsoftTenantId: '',
      );
    });

    test('rejects a whitespace-only Microsoft tenant', () async {
      await expectConfigurationFailure(
        apiBaseUrl: 'http://localhost:3000',
        microsoftTenantId: '   ',
      );
    });

    test('rejects a relative API URL', () async {
      await expectConfigurationFailure(
        apiBaseUrl: 'api',
        microsoftTenantId: 'test-tenant',
      );
    });

    test('rejects an unsupported API URL scheme', () async {
      await expectConfigurationFailure(
        apiBaseUrl: 'ftp://api.example.com',
        microsoftTenantId: 'test-tenant',
      );
    });

    test('rejects API URLs containing credentials', () async {
      await expectConfigurationFailure(
        apiBaseUrl: 'https://user:password@api.example.com',
        microsoftTenantId: 'test-tenant',
      );
    });

    test('rejects API URLs containing query parameters', () async {
      await expectConfigurationFailure(
        apiBaseUrl: 'https://api.example.com?token=value',
        microsoftTenantId: 'test-tenant',
      );
    });

    test('rejects API URLs containing fragments', () async {
      await expectConfigurationFailure(
        apiBaseUrl: 'https://api.example.com#fragment',
        microsoftTenantId: 'test-tenant',
      );
    });
  });
}
