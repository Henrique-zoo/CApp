/// Intenções de saída antes de qualquer alteração na autenticação ou no cache.
///
/// Verifica a proteção padrão, a precedência de esquecer sobre exigir login,
/// cancelamento e a restrição de contas que ainda não configuraram uma senha.
/// A janela real é montada sem SDKs; seu resultado representa a intenção que a
/// página Conta encaminha ao controlador somente depois da confirmação.
library;

import 'package:capp/src/modules/identity/identity.dart';
import 'package:capp/src/modules/identity/presentation/account_sign_out_dialog.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  Future<AccountSignOutMode?>? result;

  Future<void> open(
    WidgetTester tester, {
    bool canRequirePassword = true,
  }) async {
    result = null;

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: TextButton(
            onPressed: () => result = confirmAccountSignOut(
              tester.element(find.byType(TextButton)),
              canRequirePassword: canRequirePassword,
            ),
            child: const Text('Abrir confirmação'),
          ),
        ),
      ),
    );

    await tester.tap(find.text('Abrir confirmação'));
    await tester.pumpAndSettle();
  }

  CheckboxListTile choice(WidgetTester tester, String label) => tester
      .widget<CheckboxListTile>(find.widgetWithText(CheckboxListTile, label));

  Future<void> confirm(WidgetTester tester) async {
    await tester.tap(find.widgetWithText(FilledButton, 'Sair'));
    await tester.pumpAndSettle();
  }

  testWidgets('padrão conserva a referência e exige novo login', (
    tester,
  ) async {
    await open(tester);

    expect(find.text('Tem certeza que deseja sair?'), findsOneWidget);
    expect(choice(tester, 'Esquecer conta').value, isFalse);
    expect(choice(tester, 'Exigir login').value, isTrue);
    expect(choice(tester, 'Exigir login').onChanged, isNotNull);

    await confirm(tester);

    expect(await result, AccountSignOutMode.requirePassword);
  });

  testWidgets('desmarcar exigir login conserva a entrada rápida', (
    tester,
  ) async {
    await open(tester);
    await tester.tap(find.text('Exigir login'));
    await tester.pump();

    expect(choice(tester, 'Exigir login').value, isFalse);

    await confirm(tester);

    expect(await result, AccountSignOutMode.keepSession);
  });

  testWidgets('esquecer força e bloqueia exigir login antes da confirmação', (
    tester,
  ) async {
    await open(tester);
    await tester.tap(find.text('Exigir login'));
    await tester.tap(find.text('Esquecer conta'));
    await tester.pump();

    expect(choice(tester, 'Esquecer conta').value, isTrue);
    expect(choice(tester, 'Exigir login').value, isTrue);
    expect(choice(tester, 'Exigir login').onChanged, isNull);

    await tester.tap(find.text('Exigir login'));
    await tester.pump();

    expect(choice(tester, 'Exigir login').value, isTrue);

    await confirm(tester);

    expect(await result, AccountSignOutMode.forgetAccount);
  });

  testWidgets(
    'desmarcar esquecer devolve a edição mantendo exigir login marcado',
    (tester) async {
      await open(tester);
      await tester.tap(find.text('Esquecer conta'));
      await tester.pump();
      await tester.tap(find.text('Esquecer conta'));
      await tester.pump();

      expect(choice(tester, 'Exigir login').value, isTrue);
      expect(choice(tester, 'Exigir login').onChanged, isNotNull);

      await tester.tap(find.text('Exigir login'));
      await confirm(tester);

      expect(await result, AccountSignOutMode.keepSession);
    },
  );

  testWidgets(
    'cancelar ou voltar não devolve intenção e reabrir restaura os padrões',
    (tester) async {
      await open(tester);
      await tester.tap(find.text('Esquecer conta'));
      await tester.tap(find.text('Cancelar'));
      await tester.pumpAndSettle();

      expect(await result, isNull);

      await tester.tap(find.text('Abrir confirmação'));
      await tester.pumpAndSettle();

      expect(choice(tester, 'Esquecer conta').value, isFalse);
      expect(choice(tester, 'Exigir login').value, isTrue);

      await tester.binding.handlePopRoute();
      await tester.pumpAndSettle();

      expect(await result, isNull);
    },
  );

  testWidgets('sem senha configurada não solicita uma credencial inexistente', (
    tester,
  ) async {
    await open(tester, canRequirePassword: false);

    expect(choice(tester, 'Exigir login').value, isFalse);
    expect(choice(tester, 'Exigir login').onChanged, isNull);

    await confirm(tester);

    expect(await result, AccountSignOutMode.keepSession);

    await tester.tap(find.text('Abrir confirmação'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Esquecer conta'));
    await tester.pump();

    expect(choice(tester, 'Exigir login').value, isTrue);
    expect(choice(tester, 'Exigir login').onChanged, isNull);

    await confirm(tester);

    expect(await result, AccountSignOutMode.forgetAccount);
  });
}
