/// Regressões do fluxo interno e do provisionamento na coordenação real.
///
/// Confere que restaurar/selecionar uma conta não usa Graph dentro do prazo,
/// que Incomplete bloqueia a área principal e que respostas de outra sessão
/// não podem ativar ou sobrescrever o perfil atual. Os contratos são controlados;
/// persistência SDK e transporte possuem suítes próprias.
library;

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:capp/src/modules/identity/identity.dart';
import 'package:capp/src/modules/identity/application/models/internal_user.dart';

import '../../../../support/identity_fakes.dart';
import '../../../../support/account_identity_fakes.dart';

void main() {
  for (final complete in [true, false]) {
    test(
      'login Microsoft reutiliza evidência vigente com perfil completo=$complete',
      () async {
        final h = AccountIdentityHarness(uid: null);

        addTearDown(h.dispose);

        h.gateway.user = testInternalUser(complete: complete);

        await h.start();

        final result = await h.controller.signIn();

        expect(h.ms.signInCalls, 1);
        expect(h.gateway.restoreCalls, 1);
        expect(h.ms.currentUserTokenCalls, 0);
        expect(h.identity.calls, 0);
        expect(result.user, same(h.gateway.user));
        expect(result.verification, same(h.proof));
        expect(h.store.results['user-a'], isA<SessionCacheFound>());
        expect(
          h.controller.state.access,
          complete ? SessionAccess.ready : SessionAccess.profileRequired,
        );
      },
    );
  }

  for (final expired in [true, false]) {
    test(
      'login sincroniza evidência ausente ou vencida: expired=$expired',
      () async {
        final h = AccountIdentityHarness(uid: null);

        addTearDown(h.dispose);

        h.gateway.proof = expired
            ? testVerification(expiresAt: accountTestNow)
            : null;

        await h.start();

        final result = await h.controller.signIn();

        expect(h.gateway.restoreCalls, 1);
        expect(h.ms.currentUserTokenCalls, 1);
        expect(h.identity.calls, 1);
        expect(result.verification, same(h.proof));
        expect(h.controller.state.access, SessionAccess.ready);
      },
    );
  }

  test(
    'renovação explícita confirma novamente mesmo com evidência vigente',
    () async {
      final h = AccountIdentityHarness();

      addTearDown(h.dispose);
      await h.start();

      await h.controller.renewVerification();

      expect(h.gateway.restoreCalls, 1);
      expect(h.ms.currentUserTokenCalls, 1);
      expect(h.identity.calls, 1);
    },
  );

  test(
    'recusa ao restaurar após login não tenta contornar com Microsoft',
    () async {
      final h = AccountIdentityHarness(uid: null);

      addTearDown(h.dispose);

      final refusal = IdentityException(
        kind: IdentityFailureKind.credentialsRejected,
        cause: Exception('Conta inativa'),
        stackTrace: StackTrace.current,
        backendCode: 'account_inactive',
      );

      h.gateway.restoreFailure = refusal;

      await h.start();

      await expectLater(h.controller.signIn(), throwsA(same(refusal)));

      expect(h.gateway.restoreCalls, 1);
      expect(h.ms.currentUserTokenCalls, 0);
      expect(h.identity.calls, 0);
      expect(h.controller.state.access, SessionAccess.entryRequired);
    },
  );

  test(
    'troca de conta durante restauração do login descarta a resposta',
    () async {
      final h = AccountIdentityHarness(uid: null);

      addTearDown(h.dispose);

      final response = Completer<AccountSession>();

      h.gateway.onRestore = () => response.future;

      await h.start();

      final rejected = expectLater(
        h.controller.signIn(),
        throwsA(isA<SessionIdentityChangedException>()),
      );

      await flushSessionEvents();

      h.auth.setUserId(null);
      response.complete(
        AccountSession(user: h.gateway.user, verification: h.proof),
      );

      await rejected;

      expect(h.controller.state, isA<SessionSignedOut>());
      expect(h.ms.currentUserTokenCalls, 0);
      expect(h.identity.calls, 0);
      expect(h.store.saveCalls, 0);
    },
  );

  testWidgets(
    'falha de rede após login repete restauração antes de obter token MSAL',
    (tester) async {
      final h = AccountIdentityHarness(uid: null);

      h.gateway.restoreFailure = IdentityException(
        kind: IdentityFailureKind.communicationFailed,
        cause: Exception('Rede indisponível'),
        stackTrace: StackTrace.current,
      );

      h.controller.start();
      await tester.pump();

      await expectLater(
        h.controller.signIn(),
        throwsA(isA<IdentityException>()),
      );

      expect(h.gateway.restoreCalls, 1);
      expect(h.ms.currentUserTokenCalls, 0);

      h.currentTime = accountTestNow.add(const Duration(seconds: 29));

      await tester.pump(const Duration(seconds: 29));

      expect(h.gateway.restoreCalls, 1);

      h.gateway.restoreFailure = null;
      h.currentTime = accountTestNow.add(const Duration(seconds: 30));

      await tester.pump(const Duration(seconds: 1));
      await tester.pump();

      expect(h.gateway.restoreCalls, 2);
      expect(h.ms.signInCalls, 1);
      expect(h.ms.currentUserTokenCalls, 0);
      expect(h.identity.calls, 0);
      expect(h.controller.state.access, SessionAccess.ready);

      await tester.runAsync(h.dispose);
    },
  );

  test(
    'retoma primeiro acesso interrompido com tentativa silenciosa',
    () async {
      final h = AccountIdentityHarness();

      addTearDown(h.dispose);

      h.gateway.restoreFailure = IdentityException(
        kind: IdentityFailureKind.accountNotProvisioned,
        cause: Exception('Primeira confirmação pendente'),
        stackTrace: StackTrace.current,
      );

      await h.start();

      expect(h.ms.signInCalls, 0);
      expect(h.ms.interactionPermissions, [false]);
      expect(h.identity.calls, 1);
      expect(h.controller.state.access, SessionAccess.ready);
    },
  );

  test(
    'primeiro acesso consulta a sessão e provisiona sem repetir a consulta',
    () async {
      final h = AccountIdentityHarness(uid: null);

      addTearDown(h.dispose);

      h.gateway.user = testInternalUser(complete: false);
      h.accounts.hasPassword = false;
      h.gateway.restoreFailure = IdentityException(
        kind: IdentityFailureKind.accountNotProvisioned,
        cause: Exception('Conta ainda não provisionada'),
        stackTrace: StackTrace.current,
      );

      await h.start();

      final result = await h.controller.signIn();

      expect(h.identity.calls, 1);
      expect(h.gateway.restoreCalls, 1);
      expect(result.user.status, InternalUserStatus.incomplete);
      expect(h.controller.state.access, SessionAccess.profileRequired);
    },
  );

  test(
    'recusa da origem do aplicativo preserva os caches sem liberar entrada',
    () async {
      final h = AccountIdentityHarness();

      addTearDown(h.dispose);

      h.users.values['a'] = testInternalUser();
      h.store.results['a'] = SessionCacheFound(h.proof);
      h.gateway.restoreFailure = IdentityException(
        kind: IdentityFailureKind.appVerificationFailed,
        cause: Exception('App Check recusado'),
        stackTrace: StackTrace.current,
      );

      await h.start();

      expect(h.auth.currentUserId, 'a');
      expect(h.controller.state.access, SessionAccess.entryRequired);
      expect(h.users.values.containsKey('a'), isTrue);
      expect(h.store.results['a'], isA<SessionCacheFound>());
      expect(h.ms.currentUserTokenCalls, 0);
    },
  );

  test(
    'restaura perfil e evidência do backend mesmo sem cache, sem MSAL',
    () async {
      final h = AccountIdentityHarness();

      addTearDown(h.dispose);
      await h.start();

      expect(h.controller.state.access, SessionAccess.ready);
      expect(h.gateway.restoreCalls, 1);
      expect(h.ms.currentUserTokenCalls, 0);
      expect(
        (h.controller.state as SessionSignedIn).user?.email,
        'pessoal@example.test',
      );
    },
  );

  test(
    'usuário Incomplete recebe formulário mesmo com vínculo válido',
    () async {
      final h = AccountIdentityHarness();

      addTearDown(h.dispose);

      h.gateway.user = testInternalUser(complete: false);
      h.accounts.hasPassword = false;

      await h.start();

      expect(h.controller.state.access, SessionAccess.profileRequired);
      expect(h.ms.currentUserTokenCalls, 0);

      await h.controller.completeProfile(
        email: 'escolhido@example.test',
        password: 'transient',
        profile: testProfileCompletion,
      );

      expect(h.accounts.linkEmail, 'escolhido@example.test');
      expect(h.gateway.completedProfile, same(testProfileCompletion));
      expect(h.controller.state.access, SessionAccess.ready);
      expect(h.auth.currentUserId, 'a');
      expect(h.users.values['a']?.status, InternalUserStatus.active);
    },
  );

  test('falha ao concluir perfil não torna conta ativa', () async {
    final h = AccountIdentityHarness();

    addTearDown(h.dispose);

    h.gateway.user = testInternalUser(complete: false);

    await h.start();

    h.gateway.completeFailure = Exception('backend');

    await expectLater(
      h.controller.completeProfile(
        email: 'pessoal@example.test',
        password: 'transient',
        profile: testProfileCompletion,
      ),
      throwsException,
    );

    expect(h.controller.state.access, SessionAccess.profileRequired);
    expect(
      (h.controller.state as SessionSignedIn).user?.status,
      InternalUserStatus.incomplete,
    );
    expect(h.users.values['a']?.status, InternalUserStatus.incomplete);
  });

  test('sair e voltar retoma perfil incompleto sem Microsoft', () async {
    final h = AccountIdentityHarness();

    addTearDown(h.dispose);

    h.gateway.user = testInternalUser(complete: false);

    await h.start();
    await h.controller.signOut();

    expect(h.store.removedUids, isEmpty);

    await h.controller.selectAccount('a');
    await flushSessionEvents();

    expect(h.controller.state.access, SessionAccess.profileRequired);
    expect(h.ms.currentUserTokenCalls, 0);
  });

  for (final mode in AccountSignOutMode.values) {
    test(
      'modo ${mode.name} preserva outros perfis e limpa só quando esquecer',
      () async {
        final h = AccountIdentityHarness();

        addTearDown(h.dispose);
        await h.start();

        h.users.values['b'] = testInternalUser();
        h.store.results['b'] = SessionCacheFound(h.proof);

        await h.controller.signOut(mode: mode);

        expect(h.accounts.mode, mode);
        expect(h.auth.currentUserId, isNull);
        expect(
          h.users.values.containsKey('a'),
          mode != AccountSignOutMode.forgetAccount,
        );
        expect(
          h.store.results.containsKey('a'),
          mode != AccountSignOutMode.forgetAccount,
        );
        expect(h.users.values.containsKey('b'), isTrue);
        expect(h.store.results.containsKey('b'), isTrue);
      },
    );
  }

  test(
    'evidência vencida exige MSAL silencioso, sem refazer Firebase',
    () async {
      final h = AccountIdentityHarness();

      addTearDown(h.dispose);

      h.gateway.proof = null;

      await h.start();

      expect(h.ms.currentUserTokenCalls, 1);
      expect(h.ms.interactionPermissions, [false]);
      expect(h.ms.signInCalls, 0);
      expect(h.identity.calls, 1);
    },
  );

  test('resultado de conclusão após troca de UID é descartado', () async {
    final h = AccountIdentityHarness();

    addTearDown(h.dispose);

    h.gateway.user = testInternalUser(complete: false);

    await h.start();

    final result = Completer<InternalUser>();

    h.gateway.onComplete = () => result.future;

    final operation = h.controller.completeProfile(
      email: 'pessoal@example.test',
      password: 'transient',
      profile: testProfileCompletion,
    );

    await flushSessionEvents();
    h.auth.setUserId('b');
    await flushSessionEvents();
    result.complete(testInternalUser());
    await expectLater(
      operation,
      throwsA(isA<SessionIdentityChangedException>()),
    );

    expect((h.controller.state as SessionSignedIn).firebaseUid, 'b');
    expect(h.users.values['a']?.status, InternalUserStatus.incomplete);
  });

  test(
    'rede indisponível pode retomar fotografia válida da conta autenticada',
    () async {
      final h = AccountIdentityHarness();

      addTearDown(h.dispose);

      h.users.values['a'] = testInternalUser();
      h.store.results['a'] = SessionCacheFound(h.proof);
      h.gateway.restoreFailure = IdentityException(
        kind: IdentityFailureKind.communicationFailed,
        cause: Exception('network'),
        stackTrace: StackTrace.current,
      );

      await h.start();

      expect(h.controller.state.access, SessionAccess.ready);
      expect(h.ms.currentUserTokenCalls, 0);
      expect(h.gateway.restoreCalls, 1);

      h.controller.refresh();
      await flushSessionEvents();

      expect(h.gateway.restoreCalls, 1);
    },
  );

  test(
    'limpeza que falhou após esquecer pode ser repetida sem login',
    () async {
      final h = AccountIdentityHarness();

      addTearDown(h.dispose);
      await h.start();

      h.store.onRemove = (_) async => throw Exception('storage');

      await expectLater(
        h.controller.signOut(mode: AccountSignOutMode.forgetAccount),
        throwsA(isA<SessionCacheRemovalException>()),
      );

      expect(h.auth.currentUserId, isNull);

      h.store.onRemove = null;

      await h.controller.retryForgottenAccountCleanup();

      expect(h.store.results.containsKey('a'), isFalse);
      expect(h.users.values.containsKey('a'), isFalse);
      expect(h.controller.state.failure, isNull);
    },
  );

  test('esquecer pelo seletor limpa só os dados solicitados', () async {
    final h = AccountIdentityHarness();

    addTearDown(h.dispose);
    await h.start();

    h.store.results['b'] = SessionCacheFound(h.proof);
    h.users.values['b'] = testInternalUser();

    await h.controller.signOut();
    await h.controller.forgetSavedAccount('a');

    expect(h.store.results.containsKey('a'), isFalse);
    expect(h.users.values.containsKey('a'), isFalse);
    expect(h.store.results.containsKey('b'), isTrue);
    expect(h.users.values.containsKey('b'), isTrue);
    expect(h.ms.signInCalls, 0);
  });

  testWidgets(
    'nova tentativa de restauração respeita espera e dispensa Microsoft',
    (tester) async {
      final h = AccountIdentityHarness();

      h.users.values['a'] = testInternalUser();
      h.store.results['a'] = SessionCacheFound(h.proof);
      h.gateway.restoreFailure = IdentityException(
        kind: IdentityFailureKind.communicationFailed,
        cause: Exception('network'),
        stackTrace: StackTrace.current,
      );

      h.controller.start();
      await tester.pump();

      expect(h.gateway.restoreCalls, 1);

      h.currentTime = accountTestNow.add(const Duration(seconds: 29));

      await tester.pump(const Duration(seconds: 29));

      expect(h.gateway.restoreCalls, 1);

      h.gateway.restoreFailure = null;
      h.currentTime = accountTestNow.add(const Duration(seconds: 30));

      await tester.pump(const Duration(seconds: 1));
      await tester.pump();

      expect(h.gateway.restoreCalls, 2);
      expect(h.controller.state.access, SessionAccess.ready);
      expect(h.ms.currentUserTokenCalls, 0);

      await tester.runAsync(h.dispose);
    },
  );

  test(
    'recusa do backend não é substituída por um perfil ativo do cache',
    () async {
      final h = AccountIdentityHarness();

      addTearDown(h.dispose);

      h.users.values['a'] = testInternalUser();
      h.store.results['a'] = SessionCacheFound(h.proof);
      h.gateway.restoreFailure = IdentityException(
        kind: IdentityFailureKind.credentialsRejected,
        cause: Exception('disabled'),
        stackTrace: StackTrace.current,
        backendCode: 'account_inactive',
      );

      await h.start();

      expect(h.controller.state.access, SessionAccess.entryRequired);
      expect((h.controller.state as SessionSignedIn).user, isNull);
      expect(h.ms.currentUserTokenCalls, 0);
      expect(h.users.values.containsKey('a'), isFalse);
      expect(h.store.results.containsKey('a'), isFalse);
    },
  );
}
