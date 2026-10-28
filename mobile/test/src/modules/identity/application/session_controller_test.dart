/// Regressões do fluxo de entrada, identidade, cache e expiração.
///
/// Os contratos simulam dependências; a coordenação e suas revisões são reais.
library;

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:capp/src/modules/identity/identity.dart';

import '../../../../support/identity_fakes.dart';

/// Registra os cenários da política de sessão, sem SDK ou rede.
void main() {
  test(
    'cache válido da conta restaurada entra sem Microsoft ou backend',
    () async {
      final h = IdentityHarness(uid: 'user-a');
      addTearDown(h.dispose);
      h.store.results['user-a'] = SessionCacheFound(testVerification());
      h.controller.start();
      await flushSessionEvents();
      expect(h.controller.state.access, SessionAccess.ready);
      expect(h.store.loadedUids, ['user-a']);
      expect(h.microsoft.signInCalls, 0);
      expect(h.microsoft.reauthenticationCalls, 0);
      expect(h.gateway.calls, 0);
    },
  );

  test('cache não autentica uma conta deslogada', () async {
    final h = IdentityHarness();
    addTearDown(h.dispose);
    h.store.results['user-a'] = SessionCacheFound(testVerification());
    h.controller.start();
    await flushSessionEvents();
    expect(h.controller.state, isA<SessionSignedOut>());
    expect(h.controller.state.access, SessionAccess.entryRequired);
    expect(h.store.loadedUids, isEmpty);
  });

  test(
    'logout observado bloqueia a entrada mesmo preservando o cache',
    () async {
      final h = IdentityHarness(uid: 'user-a');
      addTearDown(h.dispose);
      h.store.results['user-a'] = SessionCacheFound(testVerification());
      h.controller.start();
      await flushSessionEvents();
      expect(h.controller.state.access, SessionAccess.ready);
      await h.auth.signOut();
      await flushSessionEvents();
      expect(h.controller.state, isA<SessionSignedOut>());
      expect(h.controller.state.access, SessionAccess.entryRequired);
      expect(h.store.results['user-a'], isA<SessionCacheFound>());
      expect(h.store.loadedUids, ['user-a']);
    },
  );

  test(
    'descarte não aguarda leitura externa nem publica sua conclusão',
    () async {
      final h = IdentityHarness(uid: 'user-a');
      addTearDown(h.dispose);
      final read = Completer<SessionCacheLoadResult>();
      h.store.onLoad = (_) => read.future;
      h.controller.start();
      await flushSessionEvents();
      expect(h.controller.state.access, SessionAccess.restoring);
      final firstDispose = h.controller.dispose();
      expect(h.controller.dispose(), same(firstDispose));
      await firstDispose;
      read.complete(SessionCacheFound(testVerification()));
      await flushSessionEvents();
      expect(h.controller.state.access, SessionAccess.restoring);
    },
  );

  test('UID Firebase não permite entrar antes do backend responder', () async {
    final h = IdentityHarness();
    addTearDown(h.dispose);
    final backend = Completer<InstitutionalVerification>();
    h.gateway.onVerify = () => backend.future;
    addTearDown(() {
      if (!backend.isCompleted) backend.complete(testVerification());
    });
    h.controller.start();
    await flushSessionEvents();
    final entry = h.controller.signIn();
    await flushSessionEvents();
    expect(h.auth.currentUserId, 'user-a');
    expect(h.controller.state.access, SessionAccess.entryRequired);
    expect(h.controller.state.isBusy, isTrue);
    backend.complete(testVerification());
    await entry;
    expect(h.controller.state.access, SessionAccess.ready);
    expect(h.microsoft.signInCalls, 1);
    expect(h.gateway.calls, 1);
    expect(h.store.saveCalls, 1);
  });

  test(
    'cache expirado mantém conta mas usa reautenticação na entrada',
    () async {
      final h = IdentityHarness(uid: 'user-a');
      addTearDown(h.dispose);
      h.store.results['user-a'] = SessionCacheFound(
        testVerification(expiresAt: h.instant),
      );
      h.controller.start();
      await flushSessionEvents();
      expect(h.controller.state, isA<SessionSignedIn>());
      expect(h.controller.state.access, SessionAccess.entryRequired);
      await h.controller.signIn();
      expect(h.microsoft.signInCalls, 0);
      expect(h.microsoft.reauthenticationCalls, 1);
      expect(h.controller.state.access, SessionAccess.ready);
    },
  );

  test('recusa do backend preserva causa e mantém entrada pendente', () async {
    final h = IdentityHarness();
    addTearDown(h.dispose);
    final stack = StackTrace.current;
    final failure = IdentityException(
      kind: IdentityFailureKind.institutionNotSupported,
      cause: Exception('recusa simulada'),
      stackTrace: stack,
      publicMessage: 'Instituição não cadastrada',
    );
    h.gateway.onVerify = () => Future.error(failure, stack);
    h.controller.start();
    await flushSessionEvents();
    await expectLater(h.controller.signIn(), throwsA(same(failure)));
    expect(h.controller.state.access, SessionAccess.entryRequired);
    expect(h.controller.state.failureDetails?.cause, same(failure));
    expect(h.store.saveCalls, 0);
    expect(h.auth.currentUserId, 'user-a');
  });

  test('falha de gravação mantém resultado válido e permite entrar', () async {
    final h = IdentityHarness();
    addTearDown(h.dispose);
    final cause = Exception('gravação indisponível');
    h.store.onSave = (_, _) async => throw cause;
    h.controller.start();
    await flushSessionEvents();
    await expectLater(
      h.controller.signIn(),
      throwsA(
        isA<SessionCacheWriteException>().having(
          (e) => e.cause,
          'causa original',
          same(cause),
        ),
      ),
    );
    expect(h.controller.state.access, SessionAccess.ready);
    expect(h.controller.state.failure, SessionFailure.cacheWriteFailed);
    expect(
      h.controller.state.failureDetails?.cause,
      isA<SessionCacheWriteException>(),
    );
  });

  test('resposta não publica depois de uma troca A/B/A', () async {
    final h = IdentityHarness(uid: 'user-a');
    addTearDown(h.dispose);
    final backend = Completer<InstitutionalVerification>();
    h.gateway.onVerify = () => backend.future;
    addTearDown(() {
      if (!backend.isCompleted) backend.complete(testVerification());
    });
    h.controller.start();
    await flushSessionEvents();
    final rejected = expectLater(
      h.controller.signIn(),
      throwsA(isA<SessionIdentityChangedException>()),
    );
    await flushSessionEvents();
    h.auth.setUserId('user-b');
    await flushSessionEvents();
    h.auth.setUserId('user-a');
    await flushSessionEvents();
    backend.complete(testVerification());
    await rejected;
    expect(h.controller.state.access, SessionAccess.entryRequired);
    expect(h.store.saveCalls, 0);
  });

  test('sincroniza o UID se o evento Firebase ainda não chegou', () async {
    final h = IdentityHarness();
    addTearDown(h.dispose);
    h.microsoft.onSignIn = () async {
      h.auth.setUserId('user-a', notify: false);
      return 'microsoft-test-token';
    };
    h.controller.start();
    await flushSessionEvents();
    await h.controller.signIn();
    expect(h.controller.state.access, SessionAccess.ready);
    expect(h.store.saveCalls, 1);
  });

  test('cache de outra identidade não é consultado', () async {
    final h = IdentityHarness(uid: 'user-b');
    addTearDown(h.dispose);
    h.store.results['user-a'] = SessionCacheFound(testVerification());
    h.controller.start();
    await flushSessionEvents();
    expect(h.store.loadedUids, ['user-b']);
    expect(h.controller.state.access, SessionAccess.entryRequired);
  });

  test('falha de observação não se transforma em logout', () async {
    final h = IdentityHarness(uid: 'user-a');
    addTearDown(h.dispose);
    h.controller.start();
    await flushSessionEvents();
    final cause = Exception('observação indisponível');
    h.auth.fail(cause, StackTrace.current);
    await flushSessionEvents();
    expect(h.controller.state, isA<SessionRestorationFailed>());
    expect((h.controller.state as SessionRestorationFailed).cause, same(cause));
    expect(h.auth.currentUserId, 'user-a');
  });

  testWidgets('expiração altera o destino sem chamar Microsoft ou backend', (
    tester,
  ) async {
    final h = IdentityHarness(uid: 'user-a');
    h.store.results['user-a'] = SessionCacheFound(
      testVerification(
        expiresAt: h.instant.add(const Duration(milliseconds: 20)),
      ),
    );
    h.controller.start();
    await tester.pump();
    expect(h.controller.state.access, SessionAccess.ready);
    h.instant = h.instant.add(const Duration(milliseconds: 20));
    await tester.pump(const Duration(milliseconds: 20));
    expect(h.controller.state.access, SessionAccess.entryRequired);
    expect(h.gateway.calls, 0);
    expect(h.microsoft.reauthenticationCalls, 0);
    await tester.runAsync(h.dispose);
  });
}
