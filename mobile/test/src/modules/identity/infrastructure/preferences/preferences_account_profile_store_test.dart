/// Cache do perfil interno preserva proprietário e separação entre contas.
library;

import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:capp/src/modules/identity/identity.dart';
import 'package:capp/src/modules/identity/infrastructure/preferences/preferences_account_profile_store.dart';

import '../../../../../support/account_identity_fakes.dart';
import '../../../../../support/preferences_fake.dart';

void main() {
  test(
    'preserva complementos declarados, inclusive após restaurar o cache',
    () async {
      final preferences = FakePreferences();

      addTearDown(preferences.dispose);

      final store = PreferencesAccountProfileStore(preferences);
      final base = testInternalUser();

      await store.saveUser(
        'a',
        InternalUser(
          id: base.id,
          status: base.status,
          nickname: base.nickname,
          email: base.email,
          currentSemester: base.currentSemester,
          institutionEntrySemester: base.institutionEntrySemester,
          name: 'Nome declarado',
          institutionalEmail: 'declarado@example.test',
          courseName: 'Curso declarado',
        ),
      );

      final restored = (await store.loadUser('a'))!;

      expect(restored.name, 'Nome declarado');
      expect(restored.institutionalEmail, 'declarado@example.test');
      expect(restored.courseName, 'Curso declarado');
    },
  );

  test('rejeita perfil ativo corrompido ao reconstituir o cache', () async {
    final preferences = FakePreferences();

    addTearDown(preferences.dispose);

    final store = PreferencesAccountProfileStore(preferences);

    await store.saveUser('a', testInternalUser());

    final data = jsonDecode(
      preferences.values['capp.identity.user.a'] as String,
    ) as Map<String, dynamic>;

    data['nickname'] = '   ';
    preferences.values['capp.identity.user.a'] = jsonEncode(data);

    await expectLater(store.loadUser('a'), throwsFormatException);
  });

  test('persiste sem credenciais e remove somente o UID escolhido', () async {
    final preferences = FakePreferences();

    addTearDown(preferences.dispose);

    final store = PreferencesAccountProfileStore(preferences);

    await store.saveUser('a', testInternalUser());
    await store.saveUser('b', testInternalUser(complete: false));

    expect((await store.loadUser('a'))!.institutionEntrySemester, '2023.2');
    expect((await store.loadUser('b'))!.nickname, isNull);

    final a = preferences.values['capp.identity.user.a'] as String;

    preferences.values['capp.identity.user.b'] = a;

    await expectLater(store.loadUser('b'), throwsFormatException);
    await store.removeUser('a');

    expect(await store.loadUser('a'), isNull);
    expect(preferences.values.containsKey('capp.identity.user.b'), isTrue);
    expect(a, isNot(contains('password')));
    expect(a, isNot(contains('token')));
  });
}
