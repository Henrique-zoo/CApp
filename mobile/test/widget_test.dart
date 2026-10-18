import 'package:capp/app/app.dart';
import 'package:capp/features/authentication/domain/authenticated_identity.dart';
import 'package:capp/features/authentication/domain/authentication_repository.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

class FakeAuthenticationRepository implements AuthenticationRepository {
  bool failNextSignIn = false;
  AuthenticatedIdentity? currentIdentity;

  static const identity = AuthenticatedIdentity(
    uid: 'firebase-user-test',
    email: '232012345@aluno.unb.br',
    emailVerified: true,
  );

  @override
  Future<AuthenticatedIdentity?> restoreSession() async => currentIdentity;

  @override
  Future<AuthenticatedIdentity> signIn() async {
    if (failNextSignIn) {
      failNextSignIn = false;
      throw const AuthenticationFailure(
        AuthenticationFailureReason.serviceUnavailable,
        'O serviço de autenticação está temporariamente indisponível. Tente novamente.',
      );
    }

    currentIdentity = identity;
    return identity;
  }

  @override
  Future<void> signOut() async {
    currentIdentity = null;
  }
}

void main() {
  testWidgets('shows the institutional Microsoft login without a session', (
    tester,
  ) async {
    await tester.pumpWidget(
      CApp(authenticationRepository: FakeAuthenticationRepository()),
    );
    await tester.pumpAndSettle();

    expect(find.text('Entre com sua conta institucional'), findsOneWidget);
    expect(find.text('Entrar com a Microsoft'), findsOneWidget);
    expect(find.byKey(const ValueKey('logout-button')), findsNothing);
  });

  testWidgets('allows retry after failure and signs out successfully', (
    tester,
  ) async {
    final repository = FakeAuthenticationRepository()..failNextSignIn = true;

    await tester.pumpWidget(CApp(authenticationRepository: repository));
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const ValueKey('microsoft-login-button')));
    await tester.pumpAndSettle();

    expect(
      find.text(
        'O serviço de autenticação está temporariamente indisponível. Tente novamente.',
      ),
      findsOneWidget,
    );
    expect(
      find.byKey(const ValueKey('microsoft-login-button')),
      findsOneWidget,
    );

    await tester.tap(find.byKey(const ValueKey('microsoft-login-button')));
    await tester.pumpAndSettle();

    expect(find.text('Sessão autenticada'), findsOneWidget);
    expect(find.text('232012345@aluno.unb.br'), findsOneWidget);

    await tester.tap(find.byKey(const ValueKey('logout-button')));
    await tester.pumpAndSettle();

    expect(find.text('Entre com sua conta institucional'), findsOneWidget);
    expect(find.byKey(const ValueKey('logout-button')), findsNothing);
  });
}
