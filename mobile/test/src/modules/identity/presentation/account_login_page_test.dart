/// Entrada pelo seletor de contas e escolha explícita de credenciais internas.
///
/// A composição real do aplicativo verifica a passagem do seletor à sessão
/// pronta. Portas controladas representam contas lembradas e autenticação, sem
/// inicializar SDKs. Os cenários cobrem a ausência inicial de campos, a marca
/// Microsoft, a indicação de criação de conta, o retorno ao seletor e a diferença
/// entre uma conta com entrada rápida e outra que exige senha.
/// A saída suspende separadamente a limpeza do índice, da evidência e do perfil
/// após o evento Firebase. A entrada só deve ser montada quando todas terminarem,
/// sem reapresentar uma referência que ainda esteja sendo removida.
library;

import 'dart:async';

import 'package:capp/src/app/app.dart';
import 'package:capp/src/modules/identity/identity.dart';
import 'package:capp/src/modules/identity/identity_widgets.dart';
import 'package:flutter/material.dart';
import 'package:flutter_svg/svg.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../../../support/account_identity_fakes.dart';

const _savedAccount = SavedAccount(
  firebaseUid: 'a',

  requiresPassword: false,
  name: 'Minha conta',
  email: 'pessoal@example.test',
);

void main() {
  /// Monta o gate real para também descartar a entrada após autenticar.
  void entryTest(
    String description,
    Future<void> Function(WidgetTester, AccountIdentityHarness) body, {
    void Function(AccountIdentityHarness)? configure,
  }) {
    testWidgets(description, (tester) async {
      final h = AccountIdentityHarness(uid: null);
      final readiness = BackendReadinessController(checkReadiness: () async {});

      configure?.call(h);

      try {
        h.controller.start();
        await tester.pumpWidget(
          CApp(
            sessionController: h.controller,
            backendReadinessController: readiness,
          ),
        );

        await tester.pumpAndSettle();
        await body(tester, h);
      } finally {
        await tester.pumpWidget(const SizedBox());
        await tester.runAsync(h.dispose);
        readiness.dispose();
      }
    });
  }

  entryTest(
    'entrada mostra contas e opções abaixo delas sem abrir os campos internos',
    (tester, h) async {
      expect(find.text('Contas salvas'), findsNothing);
      expect(find.byType(AppBar), findsNothing);
      expect(find.text('Minha conta'), findsOneWidget);
      expect(find.text('pessoal@example.test'), findsOneWidget);
      expect(find.byType(TextField), findsNothing);
      expect(find.text('Esqueci minha senha'), findsNothing);

      final email = find.widgetWithText(
        OutlinedButton,
        'Entrar com e-mail e senha',
      );
      final microsoft = find.widgetWithText(
        FilledButton,
        'Entrar com Microsoft',
      );

      expect(email, findsOneWidget);
      expect(microsoft, findsOneWidget);
      expect(
        find.text('Primeiro acesso? Entre com Microsoft para criar sua conta.'),
        findsOneWidget,
      );

      final logo = tester.widget<SvgPicture>(
        find.descendant(of: microsoft, matching: find.byType(SvgPicture)),
      );

      expect(
        (logo.bytesLoader as SvgAssetLoader).assetName,
        'assets/branding/microsoft.svg',
      );
      expect(
        tester.getTopLeft(email).dy,
        greaterThan(tester.getBottomLeft(find.byType(ListTile)).dy),
      );
      expect(
        tester.getTopLeft(microsoft).dy,
        greaterThan(tester.getBottomLeft(email).dy),
      );
      expect(h.accounts.passwordCalls, 0);
      expect(h.ms.signInCalls, 0);
    },
    configure: (h) => h.accounts.accounts = [_savedAccount],
  );

  entryTest('sem contas salvas mantém as duas opções sem pedir credenciais', (
    tester,
    h,
  ) async {
    expect(find.text('Nenhuma conta salva neste dispositivo.'), findsNothing);
    expect(find.text('Contas salvas'), findsNothing);
    expect(find.byType(ListTile), findsNothing);
    expect(find.byType(AppBar), findsNothing);
    expect(find.text('Entrar com e-mail e senha'), findsOneWidget);
    expect(find.text('Entrar com Microsoft'), findsOneWidget);
    expect(find.byType(TextField), findsNothing);
    expect(h.accounts.passwordCalls, 0);
    expect(h.ms.signInCalls, 0);
  });

  entryTest(
    'formulário interno abre por escolha e voltar limpa as credenciais',
    (tester, h) async {
      await tester.tap(find.text('Entrar com e-mail e senha'));
      await tester.pumpAndSettle();

      expect(find.byType(TextField), findsNWidgets(2));
      expect(find.text('E-mail da conta CApp'), findsOneWidget);
      expect(find.text('Senha'), findsOneWidget);
      expect(find.text('Entrar com Microsoft'), findsNothing);

      final fields = find.byType(TextField);

      await tester.enterText(fields.first, 'escolhido@example.test');
      await tester.enterText(fields.last, 'senha-transitoria');

      await tester.binding.handlePopRoute();
      await tester.pumpAndSettle();

      expect(find.byType(AppBar), findsNothing);
      expect(find.text('Entrar com Microsoft'), findsOneWidget);
      expect(find.byType(TextField), findsNothing);

      await tester.tap(find.text('Entrar com e-mail e senha'));
      await tester.pumpAndSettle();

      for (final field in tester.widgetList<TextField>(
        find.byType(TextField),
      )) {
        expect(field.controller!.text, isEmpty);
      }

      expect(h.accounts.passwordCalls, 0);
    },
  );

  entryTest('entrada interna explícita autentica sem iniciar Microsoft', (
    tester,
    h,
  ) async {
    await tester.tap(find.text('Entrar com e-mail e senha'));
    await tester.pumpAndSettle();

    final fields = find.byType(TextField);

    await tester.enterText(fields.first, 'pessoal@example.test');
    await tester.enterText(fields.last, 'senha-transitoria');
    await tester.tap(find.widgetWithText(FilledButton, 'Entrar'));
    await tester.pumpAndSettle();

    expect(h.accounts.passwordCalls, 1);
    expect(h.ms.signInCalls, 0);
    expect(find.byType(MainPage), findsOneWidget);
    expect(h.controller.state.access, SessionAccess.ready);
  });

  entryTest('conta salva com sessão utilizável entra sem abrir formulário', (
    tester,
    h,
  ) async {
    await tester.tap(find.text('Minha conta'));
    await tester.pumpAndSettle();

    expect(h.accounts.selectedUid, 'a');
    expect(h.accounts.passwordCalls, 0);
    expect(h.ms.signInCalls, 0);
    expect(find.byType(TextField), findsNothing);
    expect(find.byType(MainPage), findsOneWidget);
  }, configure: (h) => h.accounts.accounts = [_savedAccount]);

  entryTest(
    'conta protegida só abre o formulário após selecionar e mantém o UID',
    (tester, h) async {
      expect(find.byType(TextField), findsNothing);

      await tester.tap(find.text('Minha conta'));
      await tester.pumpAndSettle();

      expect(find.text('Informe a senha interna desta conta.'), findsOneWidget);
      expect(find.byType(TextField), findsNWidgets(2));

      final fields = find.byType(TextField);

      expect(tester.widget<TextField>(fields.first).enabled, isFalse);

      await tester.enterText(fields.last, 'senha-transitoria');
      await tester.tap(find.widgetWithText(FilledButton, 'Entrar'));
      await tester.pumpAndSettle();

      expect(h.accounts.selectedUid, 'a');
      expect(h.accounts.passwordCalls, 0);
      expect(h.ms.signInCalls, 0);
      expect(find.byType(MainPage), findsOneWidget);
    },
    configure: (h) {
      h.accounts.accounts = [
        const SavedAccount(
          firebaseUid: 'a',

          requiresPassword: true,
          name: 'Minha conta',
          email: 'pessoal@example.test',
        ),
      ];
      h.accounts.requirePassword = true;
    },
  );

  entryTest(
    'logout com esquecer aguarda índice, evidência e perfil antes da entrada',
    (tester, h) async {
      final cleanup = Completer<void>();
      final proofCleanup = Completer<void>();
      final profileCleanup = Completer<void>();

      h.accounts.beforeSignOutCompletes = cleanup.future;
      h.store.onRemove = (_) => proofCleanup.future;
      h.users.onRemove = (_) => profileCleanup.future;

      void expectWaitingForCleanup() {
        expect(h.auth.currentUserId, isNull);
        expect(h.controller.state.isBusy, isTrue);
        expect(find.text('Saindo...'), findsOneWidget);
        expect(find.byType(AccountLoginPage), findsNothing);
        expect(find.text('Entrar com Microsoft'), findsNothing);
        expect(find.byType(MainPage), findsNothing);
      }

      try {
        await tester.tap(find.text('Minha conta'));
        await tester.pumpAndSettle();
        await tester.tap(find.byIcon(Icons.person_outline));
        await tester.pumpAndSettle();

        final signOut = find.widgetWithText(FilledButton, 'Sair da conta');

        await tester.ensureVisible(signOut);
        await tester.tap(signOut);
        await tester.pumpAndSettle();
        await tester.tap(find.text('Esquecer conta'));
        await tester.tap(find.widgetWithText(FilledButton, 'Sair'));
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));

        // Firebase já saiu, mas o índice ainda contém a conta que será removida.
        expectWaitingForCleanup();

        expect(h.accounts.accounts.map((a) => a.firebaseUid), ['a', 'b']);

        await tester.binding.handlePopRoute();
        await tester.pump();
        expectWaitingForCleanup();

        cleanup.complete();
        await tester.pump();
        expectWaitingForCleanup();

        expect(h.accounts.accounts.map((a) => a.firebaseUid), ['b']);
        expect(h.store.results.containsKey('a'), isTrue);

        proofCleanup.complete();
        await tester.pump();
        expectWaitingForCleanup();

        expect(h.store.results.containsKey('a'), isFalse);
        expect(h.users.values.containsKey('a'), isTrue);

        profileCleanup.complete();
        await tester.pumpAndSettle();

        expect(h.controller.state.isBusy, isFalse);
        expect(find.text('Saindo...'), findsNothing);
        expect(find.byType(AccountLoginPage), findsOneWidget);
        expect(find.text('Minha conta'), findsNothing);
        expect(find.text('Outra conta'), findsOneWidget);
        expect(h.accounts.accounts.map((a) => a.firebaseUid), ['b']);
        expect(h.users.values.containsKey('a'), isFalse);
      } finally {
        if (!cleanup.isCompleted) cleanup.complete();

        if (!proofCleanup.isCompleted) proofCleanup.complete();

        if (!profileCleanup.isCompleted) profileCleanup.complete();

        await tester.pumpAndSettle();
      }
    },
    configure: (h) => h.accounts.accounts = [
      _savedAccount,
      const SavedAccount(
        firebaseUid: 'b',

        requiresPassword: false,
        name: 'Outra conta',
      ),
    ],
  );

  entryTest(
    'lista longa rola sem esconder as opções de entrada no celular',
    (tester, h) async {
      tester.view.devicePixelRatio = 1;
      tester.view.physicalSize = const Size(360, 640);

      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      await tester.pumpAndSettle();

      final microsoft = find.widgetWithText(
        FilledButton,
        'Entrar com Microsoft',
      );

      expect(tester.getBottomLeft(microsoft).dy, lessThan(640));

      await tester.ensureVisible(find.text('Conta 19'));
      await tester.pumpAndSettle();

      expect(find.text('Conta 19').hitTestable(), findsOneWidget);
      expect(microsoft.hitTestable(), findsOneWidget);
      expect(
        find
            .widgetWithText(OutlinedButton, 'Entrar com e-mail e senha')
            .hitTestable(),
        findsOneWidget,
      );
      expect(tester.takeException(), isNull);
    },
    configure: (h) => h.accounts.accounts = [
      for (var i = 0; i < 20; i++)
        SavedAccount(
          firebaseUid: 'uid-$i',

          requiresPassword: false,
          name: 'Conta $i',
        ),
    ],
  );
}
