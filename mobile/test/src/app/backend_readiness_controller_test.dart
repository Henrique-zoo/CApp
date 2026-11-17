/// Regressões da consulta inicial independente da sessão.
///
/// Callbacks controlados verificam concorrência, repetição explícita, preservação
/// de diagnósticos e descarte durante I/O sem inicializar SDKs ou sockets.
library;

import 'dart:async';

import 'package:capp/src/app/app.dart';
import 'package:flutter_test/flutter_test.dart';

/// Registra o contrato observável da coordenação de prontidão.
void main() {
  test(
    'início consulta uma vez e publica sucesso sem aguardar a rede',
    () async {
      final response = Completer<void>();
      var calls = 0;
      final controller = BackendReadinessController(
        checkReadiness: () {
          calls++;
          return response.future;
        },
      );
      addTearDown(controller.dispose);
      expect(controller.state, isA<BackendReadinessNotChecked>());

      controller.start();
      controller.start();
      expect(controller.state, isA<BackendReadinessChecking>());
      expect(calls, 1);

      final pending = controller.check();
      response.complete();
      await pending;
      expect(controller.state, isA<BackendReadinessReady>());
      controller.start();
      expect(calls, 1);
    },
  );

  test(
    'falha preserva causa e pilha e permite nova tentativa explícita',
    () async {
      final cause = Exception('backend temporariamente indisponível');
      final stackTrace = StackTrace.current;
      var calls = 0;
      final controller = BackendReadinessController(
        checkReadiness: () async {
          calls++;
          if (calls == 1) Error.throwWithStackTrace(cause, stackTrace);
        },
      );
      addTearDown(controller.dispose);

      await controller.check();
      final failure = controller.state as BackendReadinessFailed;
      expect(failure.cause, same(cause));
      expect(failure.stackTrace, same(stackTrace));
      controller.start();
      expect(calls, 1);

      await controller.check();
      expect(controller.state, isA<BackendReadinessReady>());
      expect(calls, 2);
    },
  );

  test(
    'tentativas concorrentes e reentrantes compartilham a operação',
    () async {
      final response = Completer<void>();
      var calls = 0;
      Future<void>? reentrant;
      final controller = BackendReadinessController(
        checkReadiness: () {
          calls++;
          return response.future;
        },
      );
      addTearDown(controller.dispose);
      controller.addListener(() {
        if (controller.state is BackendReadinessChecking) {
          reentrant = controller.check();
        }
      });

      final first = controller.check();
      final second = controller.check();
      expect(second, same(first));
      expect(reentrant, same(first));
      expect(calls, 1);
      response.complete();
      await first;
      expect(controller.state, isA<BackendReadinessReady>());
    },
  );

  test('descarte ignora conclusão tardia e recusa novas consultas', () async {
    final response = Completer<void>();
    var notifications = 0;
    final controller = BackendReadinessController(
      checkReadiness: () => response.future,
    );
    controller.addListener(() => notifications++);
    final pending = controller.check();
    expect(notifications, 1);

    controller.dispose();
    controller.dispose();
    response.completeError(Exception('requisição cancelada no encerramento'));
    await pending;
    expect(notifications, 1);
    expect(controller.check, throwsStateError);
    expect(controller.start, throwsStateError);
  });

  test(
    'Error inesperado propaga e não é convertido em indisponibilidade',
    () async {
      final cause = StateError('falha de programação no callback');
      var calls = 0;
      final controller = BackendReadinessController(
        checkReadiness: () async {
          calls++;
          if (calls == 1) throw cause;
        },
      );
      addTearDown(controller.dispose);

      await expectLater(controller.check(), throwsA(same(cause)));
      expect(controller.state, isNot(isA<BackendReadinessFailed>()));
      await controller.check();
      expect(controller.state, isA<BackendReadinessReady>());
    },
  );
}
