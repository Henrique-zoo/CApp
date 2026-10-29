/// Testes do adaptador de preferências, separados do codec do formato.
///
/// Exercita a persistência real sobre preferências controladas, incluindo
/// isolamento entre contas e preservação das falhas de acesso.
library;

import 'package:flutter_test/flutter_test.dart';
import 'package:capp/src/modules/identity/identity.dart';
import 'package:capp/src/modules/identity/infrastructure/preferences/preferences_session_store.dart';

import '../../../../../support/identity_fakes.dart';
import '../../../../../support/preferences_fake.dart';

/// Registra leitura, gravação e remoção do contrato de armazenamento.
void main() {
  test('tipo incorreto e registro ausente não são falhas de acesso', () async {
    final preferences = FakePreferences();
    addTearDown(preferences.dispose);
    final store = PreferencesSessionStore(preferences);
    expect(await store.load('user-a'), isA<SessionCacheMissing>());
    preferences.values['capp.identity.session.user-a'] = 42;
    final result = await store.load('user-a') as SessionCacheRejected;
    expect(result.reason, SessionCacheRejectionReason.unexpectedValueType);
  });

  test('acesso falhou: preserva causa; erro de programação: propaga', () async {
    final preferences = FakePreferences();
    addTearDown(preferences.dispose);
    final store = PreferencesSessionStore(preferences);
    final cause = Exception('armazenamento indisponível');
    preferences.readFailure.value = cause;
    final result = await store.load('user-a') as SessionCacheReadFailed;
    expect(result.cause, same(cause));
    preferences.readFailure.value = StateError('erro de implementação');
    await expectLater(store.load('user-a'), throwsStateError);
  });

  test(
    'grava e recupera o modelo sem compartilhar registros entre contas',
    () async {
      final preferences = FakePreferences();
      addTearDown(preferences.dispose);
      final store = PreferencesSessionStore(preferences);
      final verification = testVerification();
      await store.save(firebaseUid: 'user-a', verification: verification);
      final found = await store.load('user-a') as SessionCacheFound;
      expect(found.verification, verification);
      expect(await store.load('user-b'), isA<SessionCacheMissing>());
      expect(preferences.values.keys, ['capp.identity.session.user-a']);
    },
  );

  test('remover uma conta preserva outras contas e preferências', () async {
    final preferences = FakePreferences();
    addTearDown(preferences.dispose);
    final store = PreferencesSessionStore(preferences);
    preferences.values['other-setting'] = true;
    await store.save(firebaseUid: 'user-a', verification: testVerification());
    await store.save(firebaseUid: 'user-b', verification: testVerification());
    await store.remove('user-a');
    expect(await store.load('user-a'), isA<SessionCacheMissing>());
    expect(await store.load('user-b'), isA<SessionCacheFound>());
    expect(preferences.values['other-setting'], isTrue);
  });

  test(
    'falha de gravação propaga sem substituir o registro anterior',
    () async {
      final preferences = FakePreferences();
      addTearDown(preferences.dispose);
      final store = PreferencesSessionStore(preferences);
      await store.save(firebaseUid: 'user-a', verification: testVerification());
      final original = preferences.values['capp.identity.session.user-a'];
      final failure = Exception('gravação indisponível');
      preferences.writeFailure.value = failure;
      await expectLater(
        store.save(
          firebaseUid: 'user-a',
          verification: testVerification(student: true),
        ),
        throwsA(same(failure)),
      );
      expect(preferences.values['capp.identity.session.user-a'], original);
    },
  );
}
