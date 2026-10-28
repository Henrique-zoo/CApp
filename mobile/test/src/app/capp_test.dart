/// Testes da composição visual completa do CApp.
///
/// As responsabilidades de gate e tela de entrada são verificadas em suas
/// suítes específicas. Aqui permanecem o sucesso da entrada até a área principal
/// e o aviso de persistência, usando contratos sem SDK ou rede.
library;

import 'package:capp/src/app/app.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../support/identity_fakes.dart';

/// Registra os cenários que atravessam a composição de widgets da aplicação.
void main() {
  testWidgets('não estudante entra sem uma etapa manual de verificação', (
    tester,
  ) async {
    final h = IdentityHarness();
    try {
      h.controller.start();
      await tester.pumpWidget(CApp(sessionController: h.controller));
      await tester.pump();
      await tester.tap(find.text('Entrar com Microsoft'));
      await tester.pump();
      expect(find.text('Olá, Pessoa Exemplo'), findsOneWidget);
      expect(
        find.textContaining('Você não foi reconhecido como estudante'),
        findsOneWidget,
      );
      expect(find.text('Verificar instituição'), findsNothing);
      expect(find.text('Atualizar verificação'), findsNothing);
      expect(h.store.saveCalls, 1);
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
      await tester.pumpWidget(CApp(sessionController: h.controller));
      await tester.pump();
      await tester.tap(find.text('Entrar com Microsoft'));
      await tester.pump();
      expect(find.text('Olá, Pessoa Exemplo'), findsOneWidget);
      expect(
        find.textContaining('Não foi possível salvar seus dados'),
        findsOneWidget,
      );
      expect(tester.takeException(), isNull);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });
}
