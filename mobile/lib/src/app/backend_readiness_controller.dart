/// Coordenação da consulta inicial de prontidão e de novas tentativas explícitas.
///
/// Esta biblioteca pertence à camada app: acompanha um diagnóstico global,
/// separado das decisões de autenticação e acesso do módulo de identidade.
/// Recebe a consulta como callback, mantendo transporte e protocolo na composição.
/// ChangeNotifier integra a observação visual desta camada; a biblioteca não
/// pertence à application de um módulo do domínio, que permanece independente
/// de Flutter.
///
/// ## Início e observação
///
/// AppDependencies inicia o controlador uma vez, em paralelo à restauração da
/// sessão. A interface acompanha state e as notificações de ChangeNotifier.
/// start não aguarda a rede; check permite uma nova tentativa pelo usuário e
/// compartilha a operação em curso entre chamadas concorrentes.
///
/// ## Falhas e encerramento
///
/// Exceptions da consulta são publicadas com causa e pilha e concluem a tentativa
/// normalmente. Errors e outros objetos inesperados propagam pelo Future.
/// dispose interrompe publicações, sem aguardar ou cancelar o I/O; o transporte
/// pertence à composição, que também o encerra.
///
/// Não executa polling, reage a retomadas, remove cache ou concede acesso ao app.
/// Sucesso ou falha desta consulta não altera o estado da sessão.
library;

import 'dart:async';

import 'package:flutter/foundation.dart';

import 'backend_readiness_state.dart';

/// Publica o diagnóstico da consulta sem bloquear a entrada na aplicação.
final class BackendReadinessController extends ChangeNotifier {
  /// Recebe a operação assíncrona que confirma prontidão ou lança uma falha.
  ///
  /// [checkReadiness] deve concluir somente quando o endpoint confirmar sucesso.
  /// A construção não executa o callback nem inicia uma consulta.
  BackendReadinessController({required this._checkReadiness});

  /// Operação fornecida pela raiz de composição, sem dependência de um SDK aqui.
  final Future<void> Function() _checkReadiness;

  /// Fotografia atual da consulta nesta execução.
  BackendReadinessState _state = const BackendReadinessNotChecked();

  /// Resultado atual, incluindo o diagnóstico quando a consulta falha.
  BackendReadinessState get state => _state;

  /// Impede que reconstruções ou chamadas repetidas iniciem consultas extras.
  bool _started = false;

  /// Invalida publicações depois que o proprietário solicita encerramento.
  bool _disposed = false;

  /// Future compartilhado enquanto a operação atual permanece em andamento.
  Future<void>? _pendingCheck;

  /// Solicita uma única verificação inicial sem aguardar sua conclusão.
  ///
  /// Chamadas posteriores não repetem a consulta, inclusive depois de uma falha.
  /// Novas tentativas explícitas usam [check]. Lança [StateError] após descarte.
  /// Errors inesperados da operação seguem o tratamento assíncrono do chamador.
  void start() {
    _requireOpen();
    if (_started) return;
    unawaited(check());
  }

  /// Solicita prontidão ou retorna o Future da consulta já em andamento.
  ///
  /// Publica estado ocupado antes de executar a operação. A conclusão representa
  /// sucesso ou uma Exception registrada em [BackendReadinessFailed]; o chamador
  /// consulta [state] para distinguir os resultados. Errors inesperados propagam.
  ///
  /// Uma consulta explícita antes de [start] também satisfaz a consulta inicial.
  /// Lança [StateError] depois de [dispose]. Não autentica nem altera a sessão.
  Future<void> check() {
    _requireOpen();
    final pending = _pendingCheck;
    if (pending != null) return pending;

    _started = true;
    final completion = Completer<void>();
    _pendingCheck = completion.future;
    unawaited(_runCheck(completion));
    return completion.future;
  }

  /// Executa a tentativa e conserva seu diagnóstico sem publicar após descarte.
  ///
  /// O Future público já existe antes da primeira notificação; callbacks que
  /// solicitam outra tentativa durante a publicação compartilham a mesma operação.
  /// O encerramento não aguarda este método nem garante cancelar sua requisição.
  Future<void> _runCheck(Completer<void> completion) async {
    try {
      _publish(const BackendReadinessChecking());
      if (_disposed) return;
      await _checkReadiness();
      if (!_disposed) _publish(const BackendReadinessReady());
    } on Exception catch (error, stackTrace) {
      if (!_disposed) {
        _publish(BackendReadinessFailed(cause: error, stackTrace: stackTrace));
      }
    } catch (error, stackTrace) {
      completion.completeError(error, stackTrace);
    } finally {
      _pendingCheck = null;
      if (!completion.isCompleted) completion.complete();
    }
  }

  /// Substitui a fotografia antes de notificar os consumidores visuais.
  void _publish(BackendReadinessState state) {
    _state = state;
    notifyListeners();
  }

  /// Recusa operações sobre um controlador que já pertence ao encerramento.
  void _requireOpen() {
    if (_disposed) throw StateError('A consulta de prontidão foi encerrada');
  }

  /// Interrompe notificações e ignora conclusões tardias, sem aguardar a rede.
  ///
  /// Chamadas repetidas são inertes. Não fecha o transporte recebido indiretamente
  /// pelo callback; esse recurso continua pertencendo à raiz de composição.
  @override
  void dispose() {
    if (_disposed) return;
    _disposed = true;
    super.dispose();
  }
}
