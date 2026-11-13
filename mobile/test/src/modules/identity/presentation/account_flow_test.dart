/// Interações do provisionamento e da entrada com contas salvas.
///
/// Renderiza widgets reais sobre o controlador real, usando contratos controlados.
/// Verifica a ordem das duas etapas, a validação antes de avançar e a preservação
/// do rascunho ao voltar ou repetir uma conclusão. Selecionar uma conta protegida
/// encaminha a senha ao UID escolhido, sem iniciar Microsoft.
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:capp/src/modules/identity/identity.dart';
import 'package:capp/src/modules/identity/identity_widgets.dart';

import '../../../../support/account_identity_fakes.dart';

void main() {
  Widget profilePage(AccountIdentityHarness h) => MaterialApp(
    home: StreamBuilder<SessionState>(
      stream: h.controller.states,
      initialData: h.controller.state,
      builder: (context, _) => ProfileCompletionPage(
        controller: h.controller,
        state: h.controller.state as SessionSignedIn,
        now: () => DateTime(2026, 11, 12),
      ),
    ),
  );

  void profileTest(
    String description,
    Future<void> Function(WidgetTester, AccountIdentityHarness) body, {
    AccountIdentityHarness Function()? create,
  }) {
    testWidgets(description, (tester) async {
      final h =
          create?.call() ??
          AccountIdentityHarness(institutionalEmail: 'aluno@aluno.unb.br');

      h.gateway.user = testInternalUser(complete: false);
      h.accounts.hasPassword = false;

      try {
        h.controller.start();
        await tester.pump();
        await tester.pumpWidget(profilePage(h));
        await body(tester, h);
      } finally {
        await tester.pumpWidget(const SizedBox());
        await tester.runAsync(h.dispose);
      }
    });
  }

  Future<void> continueToCredentials(WidgetTester tester) async {
    await tester.enterText(
      find.widgetWithText(TextFormField, 'Apelido'),
      'Meu apelido',
    );

    await tester.enterText(
      find.widgetWithText(TextFormField, 'Período atual do curso'),
      '4',
    );

    await tester.enterText(
      find.widgetWithText(TextFormField, 'Semestre de ingresso na instituição'),
      '2023.2',
    );

    final button = find.widgetWithText(FilledButton, 'Continuar');

    await tester.ensureVisible(button);
    await tester.tap(button);
    await tester.pumpAndSettle();
  }

  Future<void> submitCredentials(WidgetTester tester) async {
    final fields = find.byType(TextFormField);

    await tester.enterText(fields.at(1), 'senha-transitoria');
    await tester.enterText(fields.at(2), 'senha-transitoria');

    final button = find.widgetWithText(FilledButton, 'Concluir cadastro');

    await tester.ensureVisible(button);
    await tester.tap(button);
    await tester.pumpAndSettle();
  }

  TextField input(WidgetTester tester, String label) =>
      tester.widget<TextField>(
        find.descendant(
          of: find.widgetWithText(TextFormField, label),
          matching: find.byType(TextField),
        ),
      );

  profileTest(
    'dados não nulos do Graph são somente leitura, incluindo texto vazio',
    (tester, h) async {
      expect(input(tester, 'Nome completo').readOnly, isTrue);
      expect(input(tester, 'Nome completo').controller!.text, 'Pessoa Exemplo');
      expect(input(tester, 'E-mail institucional').readOnly, isTrue);
      expect(input(tester, 'Curso').readOnly, isTrue);
      expect(input(tester, 'Curso').controller!.text, isEmpty);
      expect(input(tester, 'Instituição').readOnly, isTrue);

      await continueToCredentials(tester);
      await submitCredentials(tester);

      expect(h.gateway.completedProfile!.name, isNull);
      expect(h.gateway.completedProfile!.institutionalEmail, isNull);
      expect(h.gateway.completedProfile!.courseName, isNull);
    },
    create: () => AccountIdentityHarness(
      institutionalEmail: 'aluno@aluno.unb.br',
      student: true,
      courseName: '',
    ),
  );

  profileTest(
    'dados nulos do Graph podem ser completados sem mudar a evidência',
    (tester, h) async {
      for (final label in ['Nome completo', 'E-mail institucional', 'Curso']) {
        expect(input(tester, label).readOnly, isFalse);
      }

      await tester.enterText(
        find.widgetWithText(TextFormField, 'Nome completo'),
        'Nome declarado',
      );

      await tester.enterText(
        find.widgetWithText(TextFormField, 'E-mail institucional'),
        'declarado@aluno.unb.br',
      );

      await tester.enterText(
        find.widgetWithText(TextFormField, 'Curso'),
        'Curso declarado',
      );

      await continueToCredentials(tester);

      expect(
        input(tester, 'E-mail de entrada').controller!.text,
        'declarado@aluno.unb.br',
      );

      await submitCredentials(tester);

      expect(h.gateway.completedProfile!.name, 'Nome declarado');
      expect(
        h.gateway.completedProfile!.institutionalEmail,
        'declarado@aluno.unb.br',
      );
      expect(h.gateway.completedProfile!.courseName, 'Curso declarado');
      expect(h.proof.profile.name, isNull);
      expect(h.proof.profile.email, isNull);
      expect(
        (h.proof.profile.affiliation as StudentAffiliation).courseName,
        isNull,
      );
      expect(
        (h.controller.state as SessionSignedIn).user!.name,
        'Nome declarado',
      );
    },
    create: () =>
        AccountIdentityHarness(institutionalName: null, student: true),
  );

  profileTest(
    'sugestão de ingresso conta o período corrente e preserva edição manual',
    (tester, h) async {
      final semester = find.widgetWithText(
        TextFormField,
        'Período atual do curso',
      );
      final entry = find.widgetWithText(
        TextFormField,
        'Semestre de ingresso na instituição',
      );

      await tester.enterText(semester, '1');

      expect(
        input(tester, 'Semestre de ingresso na instituição').controller!.text,
        '2026.2',
      );

      await tester.enterText(semester, '4');

      expect(
        input(tester, 'Semestre de ingresso na instituição').controller!.text,
        '2025.1',
      );

      await tester.enterText(entry, '2021.2');
      await tester.enterText(semester, '5');

      expect(
        input(tester, 'Semestre de ingresso na instituição').controller!.text,
        '2021.2',
      );
    },
  );

  profileTest(
    'pede dados acadêmicos antes das credenciais e só conclui ao final',
    (tester, h) async {
      expect(find.byType(TextFormField), findsNWidgets(6));

      final formState = tester.state(find.byType(ProfileCompletionPage));
      final navigator = Navigator.of(
        tester.element(find.byType(ProfileCompletionPage)),
      );

      expect(find.text('Apelido'), findsOneWidget);
      expect(find.text('Período atual do curso'), findsOneWidget);
      expect(find.text('Semestre de ingresso na instituição'), findsOneWidget);
      expect(find.text('E-mail de entrada'), findsNothing);
      expect(find.text('Criar senha'), findsNothing);

      await continueToCredentials(tester);

      expect(find.text('Dados de acesso'), findsOneWidget);
      expect(tester.state(find.byType(ProfileCompletionPage)), same(formState));
      expect(
        Navigator.of(tester.element(find.byType(ProfileCompletionPage))),
        same(navigator),
      );
      expect(navigator.canPop(), isFalse);
      expect(find.text('Apelido'), findsNothing);
      expect(find.text('Período atual do curso'), findsNothing);
      expect(h.accounts.linkEmail, isNull);
      expect(h.gateway.completedProfile, isNull);
      expect(h.controller.state.access, SessionAccess.profileRequired);
      expect(h.gateway.user.status, InternalUserStatus.incomplete);

      final fields = find.byType(TextFormField);

      expect(fields, findsNWidgets(3));
      expect(
        tester.widget<TextFormField>(fields.first).controller!.text,
        h.proof.profile.email,
      );

      await tester.enterText(fields.at(0), 'escolhido@example.test');
      await submitCredentials(tester);

      expect(h.accounts.linkEmail, 'escolhido@example.test');
      expect(h.gateway.completedProfile?.nickname, 'Meu apelido');
      expect(h.gateway.completedProfile?.currentSemester, 4);
      expect(h.gateway.completedProfile?.institutionEntrySemester, '2023.2');
      expect(h.controller.state.access, SessionAccess.ready);
      expect(h.ms.currentUserTokenCalls, 0);
    },
  );

  profileTest(
    'dados acadêmicos inválidos impedem avançar e não vinculam credenciais',
    (tester, h) async {
      await tester.enterText(
        find.widgetWithText(TextFormField, 'Período atual do curso'),
        '0',
      );

      await tester.enterText(
        find.widgetWithText(
          TextFormField,
          'Semestre de ingresso na instituição',
        ),
        '2023.3',
      );

      final button = find.widgetWithText(FilledButton, 'Continuar');

      await tester.ensureVisible(button);
      await tester.tap(button);
      await tester.pump();

      expect(find.text('Informe um apelido.'), findsOneWidget);
      expect(find.text('Informe um período a partir de 1.'), findsOneWidget);
      expect(find.text('Use o formato AAAA.1 ou AAAA.2.'), findsOneWidget);
      expect(find.text('Dados de acesso'), findsNothing);
      expect(h.accounts.linkEmail, isNull);
      expect(h.gateway.completedProfile, isNull);
    },
  );

  profileTest(
    'voltar preserva o rascunho e permite revisar os dados acadêmicos',
    (tester, h) async {
      await continueToCredentials(tester);

      var fields = find.byType(TextFormField);

      await tester.enterText(fields.first, 'escolhido@example.test');
      await tester.enterText(fields.at(1), 'senha-que-nao-sera-retida');
      await tester.pageBack();
      await tester.pumpAndSettle();

      expect(
        tester
            .widget<TextFormField>(
              find.widgetWithText(TextFormField, 'Apelido'),
            )
            .controller!
            .text,
        'Meu apelido',
      );
      expect(
        tester
            .widget<TextFormField>(
              find.widgetWithText(TextFormField, 'Período atual do curso'),
            )
            .controller!
            .text,
        '4',
      );
      expect(
        tester
            .widget<TextFormField>(
              find.widgetWithText(
                TextFormField,
                'Semestre de ingresso na instituição',
              ),
            )
            .controller!
            .text,
        '2023.2',
      );

      await tester.enterText(
        find.widgetWithText(TextFormField, 'Período atual do curso'),
        '5',
      );

      final button = find.widgetWithText(FilledButton, 'Continuar');

      await tester.ensureVisible(button);
      await tester.tap(button);
      await tester.pumpAndSettle();

      fields = find.byType(TextFormField);

      expect(
        tester.widget<TextFormField>(fields.first).controller!.text,
        'escolhido@example.test',
      );
      expect(
        tester.widget<TextFormField>(fields.at(1)).controller!.text,
        isEmpty,
      );
      expect(
        tester.widget<TextFormField>(fields.at(2)).controller!.text,
        isEmpty,
      );

      await submitCredentials(tester);

      expect(h.gateway.completedProfile?.currentSemester, 5);
    },
  );

  profileTest('credenciais inválidas não enviam a conclusão do perfil', (
    tester,
    h,
  ) async {
    await continueToCredentials(tester);

    final fields = find.byType(TextFormField);

    await tester.enterText(fields.first, 'email-invalido');

    final button = find.widgetWithText(FilledButton, 'Concluir cadastro');

    await tester.ensureVisible(button);
    await tester.tap(button);
    await tester.pump();

    expect(find.text('Informe um e-mail válido.'), findsOneWidget);
    expect(find.text('Informe sua senha.'), findsOneWidget);

    await tester.enterText(fields.at(1), 'uma-senha');
    await tester.enterText(fields.at(2), 'outra-senha');
    await tester.ensureVisible(button);
    await tester.tap(button);
    await tester.pump();

    expect(find.text('As senhas devem ser iguais.'), findsOneWidget);
    expect(h.accounts.linkEmail, isNull);
    expect(h.gateway.completedProfile, isNull);
    expect(h.controller.state.access, SessionAccess.profileRequired);
  });

  profileTest(
    'falha na conclusão permite repetir sem refazer a primeira etapa',
    (tester, h) async {
      h.gateway.completeFailure = Exception('indisponível');

      await continueToCredentials(tester);
      await submitCredentials(tester);

      expect(find.text('Dados de acesso'), findsOneWidget);
      expect(
        find.textContaining('Não foi possível concluir o perfil'),
        findsOneWidget,
      );
      expect(h.controller.state.access, SessionAccess.profileRequired);
      expect(h.gateway.user.status, InternalUserStatus.incomplete);

      final fields = find.byType(TextFormField);

      expect(
        tester.widget<TextFormField>(fields.at(1)).controller!.text,
        isEmpty,
      );
      expect(
        tester.widget<TextFormField>(fields.at(2)).controller!.text,
        isEmpty,
      );

      h.gateway.completeFailure = null;

      await submitCredentials(tester);

      expect(h.controller.state.access, SessionAccess.ready);
      expect(h.gateway.completedProfile?.nickname, 'Meu apelido');
      expect(h.gateway.completedProfile?.currentSemester, 4);
      expect(h.gateway.completedProfile?.institutionEntrySemester, '2023.2');
    },
  );

  profileTest(
    'conclusão em curso bloqueia envio repetido e navegação de volta',
    (tester, h) async {
      final pending = Completer<InternalUser>();
      var calls = 0;

      h.gateway.onComplete = () {
        calls++;

        return pending.future;
      };

      await continueToCredentials(tester);

      final fields = find.byType(TextFormField);

      await tester.enterText(fields.at(1), 'senha-transitoria');
      await tester.enterText(fields.at(2), 'senha-transitoria');

      final button = find.widgetWithText(FilledButton, 'Concluir cadastro');

      await tester.ensureVisible(button);
      await tester.tap(button);
      await tester.tap(button);
      await tester.pump();

      expect(calls, 1);
      expect(
        tester
            .widget<FilledButton>(
              find.widgetWithText(FilledButton, 'Salvando...'),
            )
            .onPressed,
        isNull,
      );

      await tester.pageBack();
      await tester.pumpAndSettle();

      expect(find.text('Dados de acesso'), findsOneWidget);

      pending.complete(testInternalUser());
      await tester.pumpAndSettle();

      expect(h.controller.state.access, SessionAccess.ready);
    },
  );

  testWidgets('seleção protegida pede senha e preserva a conta escolhida', (
    tester,
  ) async {
    final h = AccountIdentityHarness(uid: null);

    h.accounts.accounts = [
      const SavedAccount(
        firebaseUid: 'a',

        requiresPassword: true,
        name: 'Minha conta',
        email: 'pessoal@example.test',
      ),
    ];
    h.accounts.requirePassword = true;

    h.controller.start();
    await tester.pump();
    await tester.pumpWidget(
      MaterialApp(
        home: StreamBuilder<SessionState>(
          stream: h.controller.states,
          initialData: h.controller.state,
          builder: (context, _) => AccountLoginPage(
            controller: h.controller,
            state: h.controller.state,
          ),
        ),
      ),
    );

    await tester.pump();
    await tester.tap(find.text('Minha conta'));
    await tester.pump();

    expect(find.text('Informe a senha interna desta conta.'), findsOneWidget);
    expect(h.auth.currentUserId, isNull);

    await tester.enterText(find.byType(TextField).last, 'senha-transitoria');
    await tester.ensureVisible(find.text('Entrar'));
    await tester.tap(find.text('Entrar'));
    await tester.pump();

    expect(h.accounts.selectedUid, 'a');
    expect(h.accounts.passwordCalls, 0);
    expect(h.auth.currentUserId, 'a');
    expect(h.ms.signInCalls, 0);

    await tester.pumpWidget(const SizedBox());
    await tester.runAsync(h.dispose);
  });

  testWidgets('saída confirma esquecer antes de encaminhar a intenção', (
    tester,
  ) async {
    final h = AccountIdentityHarness();

    h.controller.start();
    await tester.pump();

    AccountSignOutMode? selected;

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: AccountPage(
            state: h.controller.state as SessionSignedIn,
            onSignOut: (mode) async {
              selected = mode;
            },
          ),
        ),
      ),
    );

    await tester.ensureVisible(
      find.widgetWithText(FilledButton, 'Sair da conta'),
    );

    await tester.tap(find.widgetWithText(FilledButton, 'Sair da conta'));
    await tester.pumpAndSettle();

    expect(selected, isNull);
    expect(find.text('Tem certeza que deseja sair?'), findsOneWidget);

    await tester.tap(find.text('Esquecer conta'));

    expect(selected, isNull);

    await tester.tap(find.widgetWithText(FilledButton, 'Sair'));
    await tester.pumpAndSettle();

    expect(selected, AccountSignOutMode.forgetAccount);

    await tester.pumpWidget(const SizedBox());
    await tester.runAsync(h.dispose);
  });
}
