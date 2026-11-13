/// Testes da composição visual completa do CApp.
///
/// As responsabilidades de gate e tela de entrada são verificadas em suas
/// suítes específicas. Esta composição verifica entrada, avisos de persistência
/// e saída pela página Conta, incluindo falhas do provedor e da limpeza local.
/// Os contratos controlados não inicializam SDKs ou conexões externas.
library;

import 'dart:async';

import 'package:capp/src/app/app.dart';
import 'package:capp/src/modules/identity/identity.dart';
import 'package:capp/src/modules/identity/identity_widgets.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../support/identity_fakes.dart';
import '../../support/account_identity_fakes.dart';

/// Registra os cenários que atravessam a composição de widgets da aplicação.
void main() {
  late BackendReadinessController readiness;

  setUp(() {
    readiness = BackendReadinessController(checkReadiness: () async {});
  });

  tearDown(() => readiness.dispose());

  Future<void> openAccount(WidgetTester tester) async {
    await tester.tap(find.byIcon(Icons.person_outline));
    await tester.pumpAndSettle();
  }

  Future<void> openSignOut(WidgetTester tester) async {
    final button = find.widgetWithText(FilledButton, 'Sair da conta');

    await tester.ensureVisible(button);
    await tester.tap(button);
    await tester.pumpAndSettle();
  }

  testWidgets('não estudante entra sem uma etapa manual de verificação', (
    tester,
  ) async {
    final h = IdentityHarness();

    try {
      h.controller.start();
      await tester.pumpWidget(
        CApp(
          sessionController: h.controller,
          backendReadinessController: readiness,
        ),
      );

      await tester.pumpAndSettle();
      await tester.tap(find.text('Entrar com Microsoft'));
      await tester.pumpAndSettle();

      expect(find.byType(MainPage), findsOneWidget);
      expect(find.byType(AccountPage), findsNothing);
      expect(find.byType(NavigationBar), findsOneWidget);

      await openAccount(tester);

      expect(find.text('Pessoa Exemplo'), findsOneWidget);
      expect(find.text('Instituição'), findsOneWidget);
      expect(find.text('Instituição Exemplo (IEX)'), findsOneWidget);
      expect(
        find.textContaining('Você não foi reconhecido como estudante'),
        findsOneWidget,
      );
      expect(find.text('Verificar instituição'), findsNothing);
      expect(find.text('Atualizar verificação'), findsNothing);
      expect(h.store.saveCalls, 1);

      await tester.tap(find.byIcon(Icons.home_outlined));
      await tester.pumpAndSettle();

      expect(find.byType(AccountPage), findsNothing);
      expect(find.text('Dados da conta'), findsNothing);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });

  testWidgets('instituição sem sigla apresenta apenas o nome', (tester) async {
    final h = IdentityHarness(uid: 'user-a');

    h.store.results['user-a'] = SessionCacheFound(
      testVerification(institutionAcronym: null),
    );

    try {
      h.controller.start();
      await tester.pumpWidget(
        CApp(
          sessionController: h.controller,
          backendReadinessController: readiness,
        ),
      );

      await tester.pumpAndSettle();
      await openAccount(tester);

      expect(find.text('Instituição Exemplo'), findsOneWidget);
      expect(find.textContaining('Instituição Exemplo ('), findsNothing);
      expect(tester.takeException(), isNull);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });

  testWidgets('falha exclusiva de cache não desfaz a entrada', (tester) async {
    final h = IdentityHarness();

    try {
      h.store.onSave = (_, _) async => throw Exception('gravação indisponível');

      h.controller.start();
      await tester.pumpWidget(
        CApp(
          sessionController: h.controller,
          backendReadinessController: readiness,
        ),
      );

      await tester.pumpAndSettle();
      await tester.tap(find.text('Entrar com Microsoft'));
      await tester.pumpAndSettle();

      expect(find.byType(MainPage), findsOneWidget);
      expect(
        find.textContaining('não foi possível salvar os dados'),
        findsOneWidget,
      );
      expect(find.byType(SnackBar), findsOneWidget);
      expect(tester.takeException(), isNull);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });

  testWidgets('botão Sair encerra a sessão e retorna à tela de entrada', (
    tester,
  ) async {
    final h = IdentityHarness(uid: 'user-a');

    h.store.results['user-a'] = SessionCacheFound(testVerification());

    try {
      h.controller.start();
      await tester.pumpWidget(
        CApp(
          sessionController: h.controller,
          backendReadinessController: readiness,
        ),
      );

      await tester.pumpAndSettle();
      await openAccount(tester);

      expect(find.text('Pessoa Exemplo'), findsOneWidget);

      await openSignOut(tester);
      await tester.pumpAndSettle();
      await tester.tap(find.text('Exigir login'));
      await tester.tap(find.widgetWithText(FilledButton, 'Sair'));
      await tester.pumpAndSettle();

      expect(find.text('Entrar com Microsoft'), findsOneWidget);
      expect(find.text('Dados institucionais'), findsNothing);
      expect(h.auth.currentUserId, isNull);
      expect(h.auth.signOutCalls, 1);
      expect(h.store.results.containsKey('user-a'), isTrue);
      expect(tester.takeException(), isNull);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });

  testWidgets(
    'saída impede cliques repetidos e apresenta falha sem perder a conta',
    (tester) async {
      final h = IdentityHarness(uid: 'user-a');

      h.store.results['user-a'] = SessionCacheFound(testVerification());

      final provider = Completer<void>();
      final cause = IdentityException(
        kind: IdentityFailureKind.credentialsRejected,
        cause: Exception('falha do provedor ao sair'),
        stackTrace: StackTrace.current,
      );

      h.auth.onSignOut = () async {
        await provider.future;
        throw cause;
      };

      try {
        h.controller.start();
        await tester.pumpWidget(
          CApp(
            sessionController: h.controller,
            backendReadinessController: readiness,
          ),
        );

        await tester.pumpAndSettle();
        await openAccount(tester);
        await tester.ensureVisible(
          find.widgetWithText(FilledButton, 'Sair da conta'),
        );

        final button = tester.widget<FilledButton>(
          find.widgetWithText(FilledButton, 'Sair da conta'),
        );

        button.onPressed!();
        button.onPressed!();
        await tester.pumpAndSettle();

        expect(find.byType(AlertDialog), findsOneWidget);

        await tester.tap(find.text('Exigir login'));
        await tester.tap(find.widgetWithText(FilledButton, 'Sair'));
        await tester.pumpAndSettle();

        expect(h.auth.signOutCalls, 1);

        final busy = tester.widget<FilledButton>(
          find.widgetWithText(FilledButton, 'Saindo...'),
        );

        expect(busy.onPressed, isNull);

        provider.complete();
        await tester.pumpAndSettle();

        expect(find.text('Pessoa Exemplo'), findsOneWidget);
        expect(
          find.text('Não foi possível sair da conta. Tente novamente.'),
          findsOneWidget,
        );
        expect(find.byType(SnackBar), findsOneWidget);
        expect(h.controller.state.failureDetails?.cause, same(cause));
        expect(
          tester
              .widget<FilledButton>(
                find.widgetWithText(FilledButton, 'Sair da conta'),
              )
              .onPressed,
          isNotNull,
        );
        expect(tester.takeException(), isNull);
      } finally {
        if (!provider.isCompleted) provider.complete();

        await tester.pumpWidget(const SizedBox.shrink());
        await tester.runAsync(h.dispose);
      }
    },
  );

  testWidgets('falha de limpeza é informada na entrada depois do logout', (
    tester,
  ) async {
    final h = IdentityHarness(uid: 'user-a');

    h.store.results['user-a'] = SessionCacheFound(testVerification());
    h.store.onRemove = (_) async => throw Exception('remoção indisponível');

    try {
      h.controller.start();
      await tester.pumpWidget(
        CApp(
          sessionController: h.controller,
          backendReadinessController: readiness,
        ),
      );

      await tester.pumpAndSettle();
      await openAccount(tester);
      await openSignOut(tester);
      await tester.tap(find.text('Esquecer conta'));
      await tester.tap(find.widgetWithText(FilledButton, 'Sair'));
      await tester.pumpAndSettle();

      expect(h.auth.currentUserId, isNull);
      expect(find.text('Entrar com Microsoft'), findsOneWidget);
      expect(
        find.textContaining('Você saiu da conta, mas não foi possível remover'),
        findsOneWidget,
      );
      expect(
        h.controller.state.failureDetails?.cause,
        isA<SessionCacheRemovalException>(),
      );
      expect(tester.takeException(), isNull);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });

  testWidgets(
    'cancelar a saída preserva a conta e confirmar exige login por padrão',
    (tester) async {
      final h = AccountIdentityHarness();

      try {
        h.controller.start();
        await tester.pumpWidget(
          CApp(
            sessionController: h.controller,
            backendReadinessController: readiness,
          ),
        );

        await tester.pumpAndSettle();

        expect(find.byType(AccountPage), findsNothing);
        expect(
          tester
              .widget<NavigationBar>(find.byType(NavigationBar))
              .selectedIndex,
          0,
        );

        await openAccount(tester);
        await openSignOut(tester);

        expect(h.accounts.mode, isNull);

        await tester.tap(find.text('Cancelar'));
        await tester.pumpAndSettle();

        expect(h.auth.currentUserId, 'a');
        expect(h.accounts.mode, isNull);
        expect(find.byType(AccountPage), findsOneWidget);

        await openSignOut(tester);
        await tester.tap(find.widgetWithText(FilledButton, 'Sair'));
        await tester.pumpAndSettle();

        expect(h.accounts.mode, AccountSignOutMode.requirePassword);
        expect(h.auth.currentUserId, isNull);
        expect(find.text('Entrar com Microsoft'), findsOneWidget);
        expect(find.byType(AlertDialog), findsNothing);
      } finally {
        await tester.pumpWidget(const SizedBox.shrink());
        await tester.runAsync(h.dispose);
      }
    },
  );

  testWidgets(
    'perfil incompleto fica fora da área principal até o backend aceitar a conclusão',
    (tester) async {
      final h = AccountIdentityHarness();

      h.gateway.user = testInternalUser(complete: false);
      h.accounts.hasPassword = false;

      try {
        h.controller.start();
        await tester.pumpWidget(
          CApp(
            sessionController: h.controller,
            backendReadinessController: readiness,
          ),
        );

        await tester.pumpAndSettle();

        expect(find.byType(ProfileCompletionPage), findsOneWidget);
        expect(find.byType(MainPage), findsNothing);

        await tester.enterText(
          find.widgetWithText(TextFormField, 'Apelido'),
          'Apelido',
        );

        await tester.enterText(
          find.widgetWithText(TextFormField, 'Período atual do curso'),
          '4',
        );

        await tester.enterText(
          find.widgetWithText(
            TextFormField,
            'Semestre de ingresso na instituição',
          ),
          '2023.2',
        );

        final next = find.widgetWithText(FilledButton, 'Continuar');

        await tester.ensureVisible(next);
        await tester.tap(next);
        await tester.pumpAndSettle();

        expect(find.text('Dados de acesso'), findsOneWidget);
        expect(find.byType(MainPage), findsNothing);

        final fields = find.byType(TextFormField);

        await tester.enterText(fields.at(1), 'transient');
        await tester.enterText(fields.at(2), 'transient');

        final submit = find.widgetWithText(FilledButton, 'Concluir cadastro');

        await tester.ensureVisible(submit);
        await tester.tap(submit);
        await tester.pumpAndSettle();

        expect(find.byType(MainPage), findsOneWidget);
        expect(find.byType(ProfileCompletionPage), findsNothing);
        expect(find.text('Dados de acesso'), findsNothing);
        expect(
          Navigator.of(tester.element(find.byType(MainPage))).canPop(),
          isFalse,
        );
        expect(h.ms.signInCalls, 0);
      } finally {
        await tester.pumpWidget(const SizedBox.shrink());
        await tester.runAsync(h.dispose);
      }
    },
  );
}
