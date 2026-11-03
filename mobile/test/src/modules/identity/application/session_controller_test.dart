/// Regressões de entrada, saída, identidade, cache e expiração.
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

  test(
    'saída encerra o provedor e remove somente o cache da conta atual',
    () async {
      final h = IdentityHarness(uid: 'user-a');
      addTearDown(h.dispose);
      h.store.results['user-b'] = SessionCacheFound(testVerification());
      await _restoreCachedSession(h);

      await h.controller.signOut();

      expect(h.auth.currentUserId, isNull);
      expect(h.auth.signOutCalls, 1);
      expect(h.controller.state, isA<SessionSignedOut>());
      expect(h.controller.state.isBusy, isFalse);
      expect(h.store.removedUids, ['user-a']);
      expect(h.store.results.containsKey('user-a'), isFalse);
      expect(h.store.results['user-b'], isA<SessionCacheFound>());
      expect(h.gateway.calls, 0);
      expect(h.microsoft.signInCalls, 0);
    },
  );

  test('saída sincroniza UID nulo mesmo antes do evento do provedor', () async {
    final h = IdentityHarness(uid: 'user-a');
    addTearDown(h.dispose);
    await _restoreCachedSession(h);
    final provider = Completer<void>();
    addTearDown(() {
      if (!provider.isCompleted) provider.complete();
    });
    h.auth.onSignOut = () async {
      await provider.future;
      h.auth.setUserId(null, notify: false);
    };
    final completed = expectLater(h.controller.signOut(), completes);
    expect(h.controller.state.operation, SessionOperation.signingOut);
    await expectLater(h.controller.signOut(), throwsStateError);
    await expectLater(h.controller.renewVerification(), throwsStateError);
    provider.complete();
    await completed;
    expect(h.controller.state, isA<SessionSignedOut>());
    expect(h.store.removedUids, ['user-a']);
    expect(h.auth.signOutCalls, 1);
  });

  test(
    'falha do provedor conserva conta, cache, diagnóstico e nova tentativa',
    () async {
      final h = IdentityHarness(uid: 'user-a');
      addTearDown(h.dispose);
      await _restoreCachedSession(h);
      final cause = Exception('falha de logout');
      final stack = StackTrace.current;
      h.auth.onSignOut = () async => Error.throwWithStackTrace(cause, stack);

      await expectLater(h.controller.signOut(), throwsA(same(cause)));

      expect(h.auth.currentUserId, 'user-a');
      expect(h.controller.state.access, SessionAccess.ready);
      expect(h.controller.state.failure, SessionFailure.signOutFailed);
      expect(h.controller.state.failureDetails?.cause, same(cause));
      expect(h.controller.state.failureDetails?.stackTrace, same(stack));
      expect(h.controller.state.isBusy, isFalse);
      expect(h.store.removedUids, isEmpty);
      expect(h.store.results['user-a'], isA<SessionCacheFound>());

      h.auth.onSignOut = null;
      await h.controller.signOut();
      expect(h.controller.state, isA<SessionSignedOut>());
    },
  );

  test(
    'falha exclusiva de remoção mantém logout e diagnóstico explícito',
    () async {
      final h = IdentityHarness(uid: 'user-a');
      addTearDown(h.dispose);
      await _restoreCachedSession(h);
      final cause = Exception('falha de remoção');
      final stack = StackTrace.current;
      h.store.onRemove = (_) async => Error.throwWithStackTrace(cause, stack);

      await expectLater(
        h.controller.signOut(),
        throwsA(
          isA<SessionCacheRemovalException>()
              .having((failure) => failure.firebaseUid, 'conta', 'user-a')
              .having((failure) => failure.cause, 'causa', same(cause))
              .having(
                (failure) => failure.causeStackTrace,
                'pilha',
                same(stack),
              ),
        ),
      );

      expect(h.auth.currentUserId, isNull);
      expect(h.controller.state, isA<SessionSignedOut>());
      expect(h.controller.state.failure, SessionFailure.cacheRemovalFailed);
      expect(
        h.controller.state.failureDetails?.cause,
        isA<SessionCacheRemovalException>(),
      );
      expect(h.controller.state.isBusy, isFalse);
      expect(h.store.results['user-a'], isA<SessionCacheFound>());
    },
  );

  test(
    'entrada permanece bloqueada durante a limpeza após sair do provedor',
    () async {
      final h = IdentityHarness(uid: 'user-a');
      addTearDown(h.dispose);
      await _restoreCachedSession(h);
      final removal = Completer<void>();
      addTearDown(() {
        if (!removal.isCompleted) removal.complete();
      });
      h.store.onRemove = (_) => removal.future;
      final completed = expectLater(h.controller.signOut(), completes);
      await flushSessionEvents();
      expect(h.auth.currentUserId, isNull);
      expect(h.controller.state, isA<SessionSignedOut>());
      expect(h.controller.state.operation, SessionOperation.signingOut);
      await expectLater(h.controller.signIn(), throwsStateError);
      await expectLater(h.controller.signOut(), throwsStateError);
      removal.complete();
      await completed;
      expect(h.controller.state.isBusy, isFalse);
    },
  );

  test('conclusão da saída não altera outra sessão após troca A/B/A', () async {
    final h = IdentityHarness(uid: 'user-a');
    addTearDown(h.dispose);
    await _restoreCachedSession(h);
    h.store.results['user-b'] = SessionCacheFound(testVerification());
    final provider = Completer<void>();
    addTearDown(() {
      if (!provider.isCompleted) provider.complete();
    });
    h.auth.onSignOut = () => provider.future;
    final rejected = expectLater(
      h.controller.signOut(),
      throwsA(isA<SessionIdentityChangedException>()),
    );
    h.auth.setUserId('user-b');
    await flushSessionEvents();
    h.auth.setUserId('user-a');
    await flushSessionEvents();
    provider.complete();
    await rejected;
    expect((h.controller.state as SessionSignedIn).firebaseUid, 'user-a');
    expect(h.controller.state.access, SessionAccess.ready);
    expect(h.controller.state.failure, isNull);
    expect(h.store.removedUids, isEmpty);
  });

  test(
    'conclusão de saída após descarte não inicia remoção ou publica estado',
    () async {
      final h = IdentityHarness(uid: 'user-a');
      addTearDown(h.dispose);
      await _restoreCachedSession(h);
      final provider = Completer<void>();
      addTearDown(() {
        if (!provider.isCompleted) provider.complete();
      });
      h.auth.onSignOut = () async {
        await provider.future;
        h.auth.setUserId(null);
      };
      final rejected = expectLater(
        h.controller.signOut(),
        throwsA(isA<SessionIdentityChangedException>()),
      );
      await h.controller.dispose();
      final disposedState = h.controller.state;
      provider.complete();
      await rejected;
      expect(h.controller.state, same(disposedState));
      expect(h.store.removedUids, isEmpty);
    },
  );

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

/// Restaura a conta inicial com um perfil válido, sem interagir com Microsoft.
Future<void> _restoreCachedSession(IdentityHarness h) async {
  h.store.results['user-a'] = SessionCacheFound(testVerification());
  h.controller.start();
  await flushSessionEvents();
  expect(h.controller.state.access, SessionAccess.ready);
}
