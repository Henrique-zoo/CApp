import 'dart:async';

import 'package:capp/features/authentication/application/authentication_controller.dart';
import 'package:capp/features/authentication/domain/authenticated_identity.dart';
import 'package:capp/features/authentication/domain/authentication_repository.dart';
import 'package:flutter_test/flutter_test.dart';

const _identity = AuthenticatedIdentity(
  uid: 'firebase-user-test',
  email: '232012345@aluno.unb.br',
  emailVerified: true,
);

class _FakeAuthenticationRepository implements AuthenticationRepository {
  AuthenticatedIdentity? restoredIdentity;
  AuthenticatedIdentity signInIdentity = _identity;

  AuthenticationFailure? restoreFailure;
  AuthenticationFailure? signInFailure;
  AuthenticationFailure? signOutFailure;

  Object? restoreError;
  Object? signInError;
  Object? signOutError;

  Completer<AuthenticatedIdentity>? pendingSignIn;

  int signInCalls = 0;
  int signOutCalls = 0;

  @override
  Future<AuthenticatedIdentity?> restoreSession() async {
    final failure = restoreFailure;
    if (failure != null) throw failure;

    final error = restoreError;
    if (error != null) throw error;

    return restoredIdentity;
  }

  @override
  Future<AuthenticatedIdentity> signIn() async {
    signInCalls++;

    final failure = signInFailure;
    if (failure != null) throw failure;

    final error = signInError;
    if (error != null) throw error;

    final pending = pendingSignIn;
    if (pending != null) return pending.future;

    return signInIdentity;
  }

  @override
  Future<void> signOut() async {
    signOutCalls++;

    final failure = signOutFailure;
    if (failure != null) throw failure;

    final error = signOutError;
    if (error != null) throw error;
  }
}

void main() {
  late _FakeAuthenticationRepository repository;
  late AuthenticationController controller;

  setUp(() {
    repository = _FakeAuthenticationRepository();
    controller = AuthenticationController(repository);
  });

  tearDown(() {
    controller.dispose();
  });

  group('restoreSession', () {
    test('moves to signed out when no session exists', () async {
      await controller.restoreSession();

      expect(controller.state.status, AuthenticationStatus.signedOut);
      expect(controller.state.identity, isNull);
      expect(controller.state.message, isNull);
    });

    test('restores identity confirmed by the repository', () async {
      repository.restoredIdentity = _identity;

      await controller.restoreSession();

      expect(controller.state.status, AuthenticationStatus.authenticated);
      expect(controller.state.identity, _identity);
    });

    test('shows a safe message when restoration fails', () async {
      repository.restoreFailure = const AuthenticationFailure(
        AuthenticationFailureReason.serviceUnavailable,
        'Serviço indisponível.',
      );

      await controller.restoreSession();

      expect(controller.state.status, AuthenticationStatus.signedOut);
      expect(controller.state.message, 'Serviço indisponível.');
    });

    test('handles unexpected restoration errors', () async {
      repository.restoreError = Exception('Internal error');

      await controller.restoreSession();

      expect(controller.state.status, AuthenticationStatus.signedOut);
      expect(
        controller.state.message,
        'Não foi possível restaurar a sessão. Tente entrar novamente.',
      );
    });
  });

  group('signIn', () {
    test('authenticates only after the repository confirms identity', () async {
      await controller.restoreSession();
      await controller.signIn();

      expect(controller.state.status, AuthenticationStatus.authenticated);
      expect(controller.state.identity, _identity);
      expect(controller.state.message, isNull);
    });

    test('returns to signed out after an authentication failure', () async {
      await controller.restoreSession();
      repository.signInFailure = const AuthenticationFailure(
        AuthenticationFailureReason.serviceUnavailable,
        'Serviço indisponível.',
      );

      await controller.signIn();

      expect(controller.state.status, AuthenticationStatus.signedOut);
      expect(controller.state.identity, isNull);
      expect(controller.state.message, 'Serviço indisponível.');
    });

    test('does not show an error when login is cancelled', () async {
      await controller.restoreSession();
      repository.signInFailure = const AuthenticationFailure(
        AuthenticationFailureReason.cancelled,
        'Autenticação cancelada.',
      );

      await controller.signIn();

      expect(controller.state.status, AuthenticationStatus.signedOut);
      expect(controller.state.message, isNull);
    });

    test('handles unexpected login errors safely', () async {
      await controller.restoreSession();
      repository.signInError = Exception('Internal error');

      await controller.signIn();

      expect(controller.state.status, AuthenticationStatus.signedOut);
      expect(
        controller.state.message,
        'Não foi possível concluir o login institucional. Tente novamente.',
      );
    });

    test('does not start another login while one is in progress', () async {
      await controller.restoreSession();
      final pending = Completer<AuthenticatedIdentity>();
      repository.pendingSignIn = pending;

      final firstAttempt = controller.signIn();

      expect(controller.state.status, AuthenticationStatus.signingIn);

      await controller.signIn();

      expect(repository.signInCalls, 1);
      expect(controller.state.status, AuthenticationStatus.signingIn);

      pending.complete(_identity);
      await firstAttempt;

      expect(controller.state.status, AuthenticationStatus.authenticated);
      expect(controller.state.identity, _identity);
    });
  });

  group('signOut', () {
    test('clears the authenticated state after successful logout', () async {
      await controller.restoreSession();
      await controller.signIn();
      await controller.signOut();

      expect(repository.signOutCalls, 1);
      expect(controller.state.status, AuthenticationStatus.signedOut);
      expect(controller.state.identity, isNull);
    });

    test('preserves the authenticated state when logout fails', () async {
      await controller.restoreSession();
      await controller.signIn();

      repository.signOutError = Exception('Internal error');

      await controller.signOut();

      expect(controller.state.status, AuthenticationStatus.authenticated);
      expect(controller.state.identity, _identity);
      expect(
        controller.state.message,
        'Não foi possível encerrar a sessão. Tente novamente.',
      );
    });
  });
}
