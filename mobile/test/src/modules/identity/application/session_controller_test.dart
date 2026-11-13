/// Regressões da política de entrada, renovação, saída e isolamento da sessão.
///
/// A evidência institucional vence independentemente da autenticação Firebase.
/// Esta suíte exercita a coordenação real com relógio e contratos controlados,
/// protegendo a renovação silenciosa, a preservação do prazo após falha e a
/// rejeição de conclusões de outra sessão. Não executa SDKs, HTTP ou persistência
/// nativa; mensagens e navegação pertencem às suítes da apresentação e do gate.
library;

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:capp/src/modules/identity/application/errors/microsoft_interaction_required_exception.dart';
import 'package:capp/src/modules/identity/identity.dart';

import '../../../../support/identity_fakes.dart';

/// Registra os cenários da política de sessão, sem SDK ou rede.
void main() {
  test(
    'cache válido da conta restaurada entra sem renovar o vínculo Microsoft',
    () async {
      final h = IdentityHarness(uid: 'user-a');

      addTearDown(h.dispose);

      h.store.results['user-a'] = SessionCacheFound(testVerification());

      h.controller.start();
      await flushSessionEvents();

      expect(h.controller.state.access, SessionAccess.ready);
      expect(h.store.loadedUids, ['user-a']);
      expect(h.microsoft.signInCalls, 0);
      expect(h.microsoft.currentUserTokenCalls, 0);
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
    'cache expirado é renovado silenciosamente ao restaurar a conta',
    () async {
      final h = IdentityHarness(uid: 'user-a');

      addTearDown(h.dispose);

      h.store.results['user-a'] = SessionCacheFound(
        testVerification(expiresAt: h.instant),
      );

      h.controller.start();
      await flushSessionEvents();

      expect(h.controller.state.access, SessionAccess.ready);
      expect(h.auth.currentUserId, 'user-a');
      expect(h.microsoft.signInCalls, 0);
      expect(h.microsoft.interactionPermissions, [false]);
      expect(h.gateway.calls, 1);
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

    final restored = testVerification(institutionAcronym: 'ATUAL');

    h.store.results['user-a'] = SessionCacheFound(restored);
    h.store.results['user-b'] = SessionCacheFound(testVerification());

    final backend = Completer<InstitutionalVerification>();

    h.gateway.onVerify = () => backend.future;

    addTearDown(() {
      if (!backend.isCompleted) backend.complete(testVerification());
    });

    h.controller.start();
    await flushSessionEvents();

    final rejected = expectLater(
      h.controller.renewVerification(),
      throwsA(isA<SessionIdentityChangedException>()),
    );

    await flushSessionEvents();
    h.auth.setUserId('user-b');
    await flushSessionEvents();
    h.auth.setUserId('user-a');
    await flushSessionEvents();
    backend.complete(testVerification());
    await rejected;

    expect(h.controller.state.access, SessionAccess.ready);
    expect(
      (h.controller.state as SessionSignedIn).verification,
      same(restored),
    );
    expect(h.store.saveCalls, 0);
  });

  test('sincroniza o UID se o evento Firebase ainda não chegou', () async {
    final h = IdentityHarness();

    addTearDown(h.dispose);

    h.microsoft.onSignIn = () async {
      h.auth.setUserId('user-a', notify: false);
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
    expect(h.controller.state.access, SessionAccess.ready);
    expect((h.controller.state as SessionSignedIn).firebaseUid, 'user-b');
    expect(h.gateway.calls, 1);
    expect(h.microsoft.signInCalls, 0);
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

      await h.controller.signOut(mode: AccountSignOutMode.forgetAccount);

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

    final completed = expectLater(
      h.controller.signOut(mode: AccountSignOutMode.forgetAccount),
      completes,
    );

    expect(h.controller.state.operation, SessionOperation.signingOut);

    await expectLater(
      h.controller.signOut(mode: AccountSignOutMode.forgetAccount),
      throwsStateError,
    );

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

      await expectLater(
        h.controller.signOut(mode: AccountSignOutMode.forgetAccount),
        throwsA(same(cause)),
      );

      expect(h.auth.currentUserId, 'user-a');
      expect(h.controller.state.access, SessionAccess.ready);
      expect(h.controller.state.failure, SessionFailure.signOutFailed);
      expect(h.controller.state.failureDetails?.cause, same(cause));
      expect(h.controller.state.failureDetails?.stackTrace, same(stack));
      expect(h.controller.state.isBusy, isFalse);
      expect(h.store.removedUids, isEmpty);
      expect(h.store.results['user-a'], isA<SessionCacheFound>());

      h.auth.onSignOut = null;

      await h.controller.signOut(mode: AccountSignOutMode.forgetAccount);

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
        h.controller.signOut(mode: AccountSignOutMode.forgetAccount),
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

      final completed = expectLater(
        h.controller.signOut(mode: AccountSignOutMode.forgetAccount),
        completes,
      );

      await flushSessionEvents();

      expect(h.auth.currentUserId, isNull);
      expect(h.controller.state, isA<SessionSignedOut>());
      expect(h.controller.state.operation, SessionOperation.signingOut);

      await expectLater(h.controller.signIn(), throwsStateError);
      await expectLater(
        h.controller.signOut(mode: AccountSignOutMode.forgetAccount),
        throwsStateError,
      );

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
      h.controller.signOut(mode: AccountSignOutMode.forgetAccount),
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
        h.controller.signOut(mode: AccountSignOutMode.forgetAccount),
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

  testWidgets('timer renova a evidência antes de ela vencer', (tester) async {
    final h = IdentityHarness(uid: 'user-a');

    h.store.results['user-a'] = SessionCacheFound(
      testVerification(expiresAt: h.instant.add(const Duration(minutes: 6))),
    );

    h.controller.start();
    await tester.pump();

    expect(h.controller.state.access, SessionAccess.ready);
    expect(h.gateway.calls, 0);

    h.instant = h.instant.add(const Duration(minutes: 1));

    await tester.pump(const Duration(minutes: 1));

    expect(h.controller.state.access, SessionAccess.ready);
    expect(h.gateway.calls, 1);
    expect(h.microsoft.interactionPermissions, [false]);
    expect(h.store.saveCalls, 1);

    await tester.runAsync(h.dispose);
  });

  group('renovação automática', () {
    testWidgets(
      'falhas temporárias respeitam espera progressiva mesmo após expirar',
      (tester) async {
        final h = IdentityHarness(uid: 'user-a');

        h.store.results['user-a'] = SessionCacheFound(
          testVerification(expiresAt: h.instant),
        );

        var kind = IdentityFailureKind.communicationFailed;

        h.gateway.onVerify = () async => throw IdentityException(
          kind: kind,
          cause: Exception('Falha temporária'),
          stackTrace: StackTrace.current,
        );

        try {
          h.controller.start();
          await tester.pump();

          expect(h.gateway.calls, 1);

          var calls = 1;

          for (final seconds in [30, 60, 120, 240, 300, 300]) {
            // Retomadas frequentes não reiniciam nem antecipam a espera.
            h.instant = h.instant.add(Duration(seconds: seconds - 1));

            h.controller.refresh();
            h.controller.refresh();
            await tester.pump(Duration(seconds: seconds - 1));

            expect(h.gateway.calls, calls);

            h.instant = h.instant.add(const Duration(seconds: 1));
            kind = IdentityFailureKind.serviceUnavailable;

            await tester.pump(const Duration(seconds: 1));

            expect(h.gateway.calls, ++calls);
            expect(h.controller.state.access, SessionAccess.entryRequired);
            expect(h.controller.state.isBusy, isFalse);
            expect(h.auth.currentUserId, 'user-a');
          }

          expect(h.microsoft.interactionPermissions, everyElement(false));
          expect(h.microsoft.signInCalls, 0);
          expect(h.store.saveCalls, 0);
        } finally {
          await tester.runAsync(h.dispose);
        }
      },
    );

    test('retomada após a espera recupera o acesso silenciosamente', () async {
      final h = IdentityHarness(uid: 'user-a');

      addTearDown(h.dispose);

      h.gateway.onVerify = () async => throw _networkFailure();

      h.controller.start();
      await flushSessionEvents();

      expect(h.gateway.calls, 1);

      h.instant = h.instant.add(const Duration(seconds: 29));

      h.controller.refresh();
      await flushSessionEvents();

      expect(h.gateway.calls, 1);

      h.gateway.onVerify = null;
      h.instant = h.instant.add(const Duration(seconds: 1));

      h.controller.refresh();
      await flushSessionEvents();

      expect(h.gateway.calls, 2);
      expect(h.microsoft.interactionPermissions, [false, false]);
      expect(h.controller.state.access, SessionAccess.ready);
    });

    test('sucesso reinicia espera progressiva no próximo ciclo', () async {
      final h = IdentityHarness(uid: 'user-a');

      addTearDown(h.dispose);

      h.gateway.onVerify = () async => throw _networkFailure();

      h.controller.start();
      await flushSessionEvents();

      h.instant = h.instant.add(const Duration(seconds: 30));

      h.controller.refresh();
      await flushSessionEvents();

      expect(h.gateway.calls, 2);

      h.gateway.onVerify = null;
      h.instant = h.instant.add(const Duration(minutes: 1));

      h.controller.refresh();
      await flushSessionEvents();

      expect(h.controller.state.access, SessionAccess.ready);

      h.gateway.onVerify = () async => throw _networkFailure();
      h.instant = DateTime.utc(2026, 1, 1, 12, 55);

      h.controller.refresh();
      await flushSessionEvents();

      expect(h.gateway.calls, 4);

      h.instant = h.instant.add(const Duration(seconds: 30));

      h.controller.refresh();
      await flushSessionEvents();

      expect(h.gateway.calls, 5);
    });

    for (final kind in [
      IdentityFailureKind.authenticationCancelled,
      IdentityFailureKind.credentialsRejected,
      IdentityFailureKind.institutionNotSupported,
      IdentityFailureKind.invalidResponse,
    ]) {
      test('falha ${kind.name} suspende tentativas automáticas', () async {
        final h = IdentityHarness(uid: 'user-a');

        addTearDown(h.dispose);

        final failure = IdentityException(
          kind: kind,
          cause: Exception('Recusa'),
          stackTrace: StackTrace.current,
        );

        h.microsoft.onGetAccessTokenForCurrentUser = () async => throw failure;

        h.controller.start();
        await flushSessionEvents();

        h.instant = h.instant.add(const Duration(days: 1));

        h.controller.refresh();
        h.controller.refresh();
        await flushSessionEvents();

        expect(h.microsoft.interactionPermissions, [false]);

        h.instant = h.instant.add(const Duration(minutes: 10));

        h.controller.refresh();
        await flushSessionEvents();

        expect(h.microsoft.interactionPermissions, [false]);
        expect(h.controller.state.failureDetails?.cause, same(failure));
        expect(h.gateway.calls, 0);
      });
    }

    testWidgets(
      'tentativa manual substitui o retry pendente sem duplicar consultas',
      (tester) async {
        final h = IdentityHarness(uid: 'user-a');

        h.gateway.onVerify = () async => throw _networkFailure();

        try {
          h.controller.start();
          await tester.pump();

          h.gateway.onVerify = null;

          final manual = h.controller.signIn();

          await tester.pump();
          await manual;

          h.instant = h.instant.add(const Duration(seconds: 30));

          await tester.pump(const Duration(seconds: 30));

          expect(h.gateway.calls, 2);
          expect(h.microsoft.interactionPermissions, [false, false]);
          expect(h.controller.state.access, SessionAccess.ready);
        } finally {
          await tester.runAsync(h.dispose);
        }
      },
    );

    testWidgets('descarte cancela a próxima tentativa silenciosa', (
      tester,
    ) async {
      final h = IdentityHarness(uid: 'user-a');

      h.gateway.onVerify = () async => throw _networkFailure();

      h.controller.start();
      await tester.pump();
      await tester.runAsync(h.dispose);

      h.instant = h.instant.add(const Duration(minutes: 10));

      await tester.pump(const Duration(minutes: 10));

      expect(h.gateway.calls, 1);
    });

    testWidgets('logout cancela a próxima tentativa silenciosa', (
      tester,
    ) async {
      final h = IdentityHarness(uid: 'user-a');

      h.gateway.onVerify = () async => throw _networkFailure();

      try {
        h.controller.start();
        await tester.pump();

        final signedOut = h.controller.signOut(
          mode: AccountSignOutMode.forgetAccount,
        );

        await tester.pump();
        await signedOut;

        h.instant = h.instant.add(const Duration(minutes: 10));

        await tester.pump(const Duration(minutes: 10));

        expect(h.gateway.calls, 1);
        expect(h.controller.state, isA<SessionSignedOut>());
      } finally {
        await tester.runAsync(h.dispose);
      }
    });

    test('falha antiga após A/B/A não bloqueia a nova agenda', () async {
      final h = IdentityHarness(uid: 'user-a');

      addTearDown(h.dispose);

      final old = Completer<InstitutionalVerification>();

      h.gateway.onVerify = () => old.future;

      h.controller.start();
      await flushSessionEvents();

      h.store.results['user-b'] = SessionCacheFound(testVerification());

      h.auth.setUserId('user-b');
      await flushSessionEvents();

      h.gateway.onVerify = () async => throw _networkFailure();

      h.auth.setUserId('user-a');
      await flushSessionEvents();
      old.completeError(
        MicrosoftInteractionRequiredException(
          cause: Exception('Antiga'),
          stackTrace: StackTrace.current,
        ),
      );

      await flushSessionEvents();

      expect(h.controller.state.failure, SessionFailure.communicationFailed);

      h.gateway.onVerify = null;
      h.instant = h.instant.add(const Duration(seconds: 30));

      h.controller.refresh();
      await flushSessionEvents();

      expect(h.gateway.calls, 3);
      expect(h.controller.state.access, SessionAccess.ready);
      expect(h.microsoft.interactionPermissions, [false, false, false]);
    });

    for (final cache in <SessionCacheLoadResult>[
      const SessionCacheMissing(),
      const SessionCacheRejected(SessionCacheRejectionReason.malformedJson),
      SessionCacheReadFailed(
        cause: Exception('Leitura indisponível'),
        stackTrace: StackTrace.current,
      ),
    ]) {
      test(
        'recupera silenciosamente evidência ausente: ${cache.runtimeType}',
        () async {
          final h = IdentityHarness(uid: 'user-a');

          addTearDown(h.dispose);

          final backend = Completer<InstitutionalVerification>();

          h.gateway.onVerify = () => backend.future;
          h.store.results['user-a'] = cache;

          h.controller.start();
          await flushSessionEvents();
          h.controller.refresh();
          await flushSessionEvents();

          expect(h.controller.state.access, SessionAccess.entryRequired);
          expect(h.microsoft.interactionPermissions, [false]);
          expect(h.microsoft.signInCalls, 0);
          expect(h.gateway.calls, 1);
          expect(
            (h.controller.state as SessionSignedIn).cacheLoadResult,
            same(cache),
          );

          backend.complete(testVerification());
          await flushSessionEvents();

          expect(h.controller.state.access, SessionAccess.ready);
          expect(h.store.saveCalls, 1);
        },
      );
    }

    test(
      'interação necessária preserva a conta e não gera um ciclo automático',
      () async {
        final h = IdentityHarness(uid: 'user-a');

        addTearDown(h.dispose);

        final previous = testVerification(expiresAt: h.instant);

        h.store.results['user-a'] = SessionCacheFound(previous);

        final failure = MicrosoftInteractionRequiredException(
          cause: Exception('Consent required'),
          stackTrace: StackTrace.current,
        );

        h.microsoft.onGetAccessTokenForCurrentUser = () async => throw failure;

        h.controller.start();
        await flushSessionEvents();

        expect(h.controller.state.access, SessionAccess.entryRequired);
        expect(h.auth.currentUserId, 'user-a');
        expect(h.controller.state.failureDetails?.cause, same(failure));
        expect(h.microsoft.interactionPermissions, [false]);
        expect(h.gateway.calls, 0);
        expect(h.store.saveCalls, 0);

        h.controller.refresh();
        h.controller.refresh();
        await flushSessionEvents();

        expect(h.microsoft.interactionPermissions, [false]);

        h.microsoft.onGetAccessTokenForCurrentUser = null;

        await h.controller.signIn();

        expect(h.microsoft.interactionPermissions, [false, true]);
        expect(h.controller.state.access, SessionAccess.ready);
      },
    );

    test(
      'falha de rede preserva evidência e não estende sua validade',
      () async {
        final h = IdentityHarness(uid: 'user-a');

        addTearDown(h.dispose);

        final previous = testVerification(
          expiresAt: h.instant.add(const Duration(minutes: 3)),
        );

        h.store.results['user-a'] = SessionCacheFound(previous);

        final failure = IdentityException(
          kind: IdentityFailureKind.communicationFailed,
          cause: Exception('Network unavailable'),
          stackTrace: StackTrace.current,
        );

        h.gateway.onVerify = () async => throw failure;

        h.controller.start();
        await flushSessionEvents();

        expect(h.controller.state.access, SessionAccess.ready);
        expect(
          (h.controller.state as SessionSignedIn).verification,
          same(previous),
        );
        expect(h.store.saveCalls, 0);

        h.instant = previous.expiresAt;

        h.controller.refresh();
        await flushSessionEvents();

        expect(h.controller.state.access, SessionAccess.entryRequired);
        expect(h.gateway.calls, 2);
        expect(h.auth.currentUserId, 'user-a');
      },
    );

    test('retomadas repetidas não duplicam uma renovação pendente', () async {
      final h = IdentityHarness(uid: 'user-a');

      addTearDown(h.dispose);

      final backend = Completer<InstitutionalVerification>();

      h.store.results['user-a'] = SessionCacheFound(
        testVerification(expiresAt: h.instant.add(const Duration(minutes: 3))),
      );
      h.gateway.onVerify = () => backend.future;

      h.controller.start();
      await flushSessionEvents();
      h.controller.refresh();
      h.controller.refresh();
      await flushSessionEvents();

      expect(h.gateway.calls, 1);
      expect(h.controller.state.access, SessionAccess.ready);

      h.instant = h.instant.add(const Duration(minutes: 3));

      h.controller.refresh();
      await flushSessionEvents();

      expect(h.controller.state.access, SessionAccess.entryRequired);
      expect(h.gateway.calls, 1);

      backend.complete(testVerification());
      await flushSessionEvents();

      expect(h.store.saveCalls, 1);
    });

    test(
      'resposta automática antiga não substitui a sessão após A/B/A',
      () async {
        final h = IdentityHarness(uid: 'user-a');

        addTearDown(h.dispose);

        final backend = Completer<InstitutionalVerification>();

        h.store.results['user-a'] = SessionCacheFound(
          testVerification(expiresAt: h.instant),
        );
        h.gateway.onVerify = () => backend.future;

        h.controller.start();
        await flushSessionEvents();
        h.auth.setUserId('user-b');
        await flushSessionEvents();

        final restored = testVerification(institutionAcronym: 'ATUAL');

        h.store.results['user-a'] = SessionCacheFound(restored);

        h.auth.setUserId('user-a');
        await flushSessionEvents();
        backend.complete(testVerification(institutionAcronym: 'ANTIGA'));
        await flushSessionEvents();

        expect(
          (h.controller.state as SessionSignedIn).verification,
          same(restored),
        );
        expect(h.controller.state.failure, isNull);
        expect(h.store.saveCalls, 0);
      },
    );

    test(
      'cada nova evidência permite uma futura renovação automática',
      () async {
        final h = IdentityHarness(uid: 'user-a');

        addTearDown(h.dispose);

        h.store.results['user-a'] = SessionCacheFound(
          testVerification(expiresAt: h.instant),
        );

        h.controller.start();
        await flushSessionEvents();

        expect(h.gateway.calls, 1);

        h.instant = DateTime.utc(2026, 1, 1, 12, 56);
        h.gateway.onVerify = () async =>
            testVerification(expiresAt: DateTime.utc(2026, 1, 1, 14));

        h.controller.refresh();
        await flushSessionEvents();

        expect(h.gateway.calls, 2);
        expect(h.microsoft.interactionPermissions, [false, false]);
        expect(h.controller.state.access, SessionAccess.ready);
      },
    );

    testWidgets(
      'validade muito curta não provoca um ciclo imediato de consultas',
      (tester) async {
        final h = IdentityHarness(uid: 'user-a');

        h.gateway.onVerify = () async => testVerification(
          expiresAt: h.instant.add(const Duration(seconds: 10)),
        );

        try {
          h.controller.start();
          await tester.pump();
          h.controller.refresh();
          h.controller.refresh();
          await tester.pump();

          expect(h.gateway.calls, 1);

          h.instant = h.instant.add(const Duration(seconds: 10));

          await tester.pump(const Duration(seconds: 10));

          expect(h.controller.state.access, SessionAccess.entryRequired);
          expect(h.gateway.calls, 1);

          h.instant = h.instant.add(const Duration(seconds: 20));

          await tester.pump(const Duration(seconds: 20));

          expect(h.gateway.calls, 2);
          expect(h.controller.state.access, SessionAccess.ready);
        } finally {
          await tester.runAsync(h.dispose);
        }
      },
    );
  });
}

/// Restaura a conta inicial com um perfil válido, sem interagir com Microsoft.
Future<void> _restoreCachedSession(IdentityHarness h) async {
  h.store.results['user-a'] = SessionCacheFound(testVerification());

  h.controller.start();
  await flushSessionEvents();

  expect(h.controller.state.access, SessionAccess.ready);
}

/// Falha transitória reconhecida pela política de recuperação, sem rede real.
IdentityException _networkFailure() => IdentityException(
  kind: IdentityFailureKind.communicationFailed,
  cause: Exception('Rede indisponível'),
  stackTrace: StackTrace.current,
);
