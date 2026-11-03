/// Regressões do componente visual compartilhado, sem serviços ou controladores.
///
/// Verifica preservação de estado local nas transições, encaminhamento da ação
/// e respeito à área segura com textos ampliados em uma tela estreita.
library;

import 'package:capp/src/presentation/presentation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

/// Monta o componente sobre um formulário sem chave global ou controlador externo.
///
/// Os parâmetros representam apenas apresentação; o campo possui seu próprio
/// estado para detectar descarte indevido quando a mensagem aparece ou desaparece.
Widget _host({String? message, bool isLoading = false, VoidCallback? onRetry}) {
  return MaterialApp(
    home: StatusBanner(
      message: message,
      isLoading: isLoading,
      onRetry: onRetry,
      child: const Scaffold(body: TextField()),
    ),
  );
}

/// Registra os contratos de composição visual do banner genérico.
void main() {
  testWidgets('mostrar, atualizar e ocultar preserva o estado do formulário', (
    tester,
  ) async {
    await tester.pumpWidget(_host());
    await tester.enterText(find.byType(TextField), 'Rascunho preservado');
    final fieldState = tester.state<EditableTextState>(
      find.byType(EditableText),
    );

    await tester.pumpWidget(_host(message: 'Operação indisponível'));
    expect(find.text('Rascunho preservado'), findsOneWidget);
    expect(
      tester.state<EditableTextState>(find.byType(EditableText)),
      same(fieldState),
    );

    await tester.pumpWidget(
      _host(message: 'Consultando serviço', isLoading: true),
    );
    expect(find.byType(CircularProgressIndicator), findsOneWidget);
    expect(find.text('Rascunho preservado'), findsOneWidget);

    await tester.pumpWidget(_host());
    expect(find.text('Consultando serviço'), findsNothing);
    expect(find.text('Rascunho preservado'), findsOneWidget);
    expect(
      tester.state<EditableTextState>(find.byType(EditableText)),
      same(fieldState),
    );
    expect(tester.takeException(), isNull);
  });

  testWidgets(
    'nova tentativa é explícita e fica desabilitada no carregamento',
    (tester) async {
      var attempts = 0;
      void retry() => attempts++;
      await tester.pumpWidget(
        _host(message: 'Falha temporária', onRetry: retry),
      );
      expect(attempts, 0);
      await tester.tap(find.text('Tentar novamente'));
      expect(attempts, 1);

      await tester.pumpWidget(
        _host(message: 'Consultando serviço', isLoading: true, onRetry: retry),
      );
      final button = tester.widget<TextButton>(
        find.widgetWithText(TextButton, 'Tentar novamente'),
      );
      expect(button.onPressed, isNull);
      await tester.tap(find.text('Tentar novamente'));
      expect(attempts, 1);
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets(
    'texto ampliado cabe e a área segura superior é consumida uma vez',
    (tester) async {
      tester.view.physicalSize = const Size(320, 640);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      Widget host(String? message) => MaterialApp(
        home: MediaQuery(
          data: const MediaQueryData(
            size: Size(320, 640),
            padding: EdgeInsets.only(top: 24),
            viewPadding: EdgeInsets.only(top: 24),
            textScaler: TextScaler.linear(2),
          ),
          child: StatusBanner(
            message: message,
            onRetry: message == null ? null : () {},
            child: Scaffold(
              body: Builder(
                builder: (context) =>
                    Text('padding:${MediaQuery.paddingOf(context).top}'),
              ),
            ),
          ),
        ),
      );

      const message =
          'Não foi possível concluir a operação. Tente novamente em instantes.';
      await tester.pumpWidget(host(message));
      expect(find.text(message), findsOneWidget);
      expect(find.text('padding:0.0'), findsOneWidget);
      expect(
        tester.getTopLeft(find.text(message)).dy,
        greaterThanOrEqualTo(24),
      );
      expect(tester.takeException(), isNull);

      await tester.pumpWidget(host(null));
      expect(find.text('padding:24.0'), findsOneWidget);
      expect(tester.takeException(), isNull);
    },
  );
}
