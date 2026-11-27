/// Integridade do índice de contas: seleção, proteção e referências únicas.
library;

import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:capp/src/modules/identity/application/models/saved_account.dart';
import 'package:capp/src/modules/identity/infrastructure/preferences/account_registry.dart';
import 'package:capp/src/modules/identity/infrastructure/firebase/firebase_account_reference.dart';

import '../../../../../support/preferences_fake.dart';

void main() {
  test(
    'preserva conta protegida e ausência de seleção no mesmo envelope',
    () async {
      final preferences = FakePreferences();

      addTearDown(preferences.dispose);

      final registry = AccountRegistry(preferences);

      expect(await registry.read(), isNull);

      await registry.write([
        const FirebaseAccountReference(
          authAppName: 'app-a',
          account: SavedAccount(
            firebaseUid: 'a',
            requiresPassword: true,
            email: 'pessoal@example.test',
            name: 'Pessoa',
          ),
        ),
      ], null);

      final restored = (await registry.read())!;

      expect(restored.activeUid, isNull);
      expect(restored.accounts.single.requiresPassword, isTrue);
      expect(restored.accounts.single.email, 'pessoal@example.test');
    },
  );

  for (final invalid in [
    {'version': 2, 'accounts': []},
    {'version': 1, 'accounts': [], 'activeUid': 'missing'},
    {
      'version': 1,
      'accounts': [
        {'uid': 'a', 'app': 'app', 'requiresPassword': 'false'},
      ],
    },
    {
      'version': 1,
      'accounts': [
        {'uid': 'a', 'app': 'app', 'requiresPassword': false},
        {'uid': 'a', 'app': 'other', 'requiresPassword': false},
      ],
    },
    {
      'version': 1,
      'accounts': [
        {'uid': 'a', 'app': 'app', 'requiresPassword': false},
        {'uid': 'b', 'app': 'app', 'requiresPassword': false},
      ],
    },
  ]) {
    test('recusa índice incompatível: $invalid', () async {
      final preferences = FakePreferences();

      addTearDown(preferences.dispose);

      preferences.values[AccountRegistry.key] = jsonEncode(invalid);

      await expectLater(
        AccountRegistry(preferences).read(),
        throwsFormatException,
      );
    });
  }
}
