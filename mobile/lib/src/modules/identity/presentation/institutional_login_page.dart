/// Apresentação da operação única de entrada institucional.
///
/// Esta biblioteca traduz o estado de aplicação em uma experiência de login.
/// Ela recebe um controlador e sua fotografia atual, sem criar adaptadores,
/// observar Firebase diretamente ou montar requisições HTTP.
///
/// ## Interação apresentada
///
/// Durante a restauração ou uma operação em andamento, a tela informa atividade
/// e impede uma nova tentativa. Quando a entrada está pendente, oferece uma única
/// ação: entrar com Microsoft. Essa ação solicita signIn ao controlador, que
/// escolhe login ou reautenticação e também realiza a verificação no backend.
///
/// Não há etapa manual de verificação posterior ao login. O gate mantém esta
/// tela enquanto a operação está pendente e a substitui quando o controlador
/// publica o destino pronto.
///
/// ## Estado, falhas e concorrência
///
/// sessionFailureMessage converte o diagnóstico em mensagem pública.
/// A página usa o estado recebido para renderizar e confere o estado atual do
/// controlador ao interagir, impedindo cliques repetidos antes de outro frame.
///
/// As falhas esperadas já representadas pelo fluxo são consumidas no callback
/// assíncrono; a mensagem vem do estado, preservando o diagnóstico original.
/// Falhas inesperadas sem o tratamento correspondente continuam propagando.
///
/// A página pertence à apresentação do módulo. Tema global, escolha do destino
/// e propriedade do controlador pertencem à camada app.
library;

import 'package:flutter/material.dart';

import '../application/errors/identity_exception.dart';
import '../application/errors/session_cache_write_exception.dart';
import '../application/errors/session_identity_changed_exception.dart';
import '../application/session_controller.dart';
import '../application/session_state.dart';
import 'session_failure_message.dart';

/// Apresenta entrada pendente, operação em andamento e eventuais falhas.
///
/// Uma autenticação Firebase sem perfil válido usa a mesma ação de entrada.
/// O controlador escolhe entre login e reautenticação Microsoft.
///
/// O chamador fornece as atualizações e substitui a tela quando a sessão
/// está pronta. Não navega, mantém assinatura própria ou encerra o controlador.
class InstitutionalLoginPage extends StatelessWidget {
  /// Recebe o controlador iniciado e sua fotografia de entrada pendente.
  ///
  /// [state] deve pertencer ao ramo SessionAccess.entryRequired.
  const InstitutionalLoginPage({
    super.key,
    required this.controller,
    required this.state,
  });

  /// Coordenador compartilhado da composição.
  final SessionController controller;

  /// Fotografia com operação e diagnóstico da última falha.
  final SessionState state;

  /// Solicita entrada somente enquanto necessária e ociosa.
  ///
  /// O controlador marca a operação antes do primeiro await, impedindo cliques
  /// repetidos antes da reconstrução da tela.
  ///
  /// Falhas reconhecidas ficam preservadas no estado. Esta fronteira consome
  /// seu Future para apresentá-las declarativamente. Tentativas obsoletas
  /// não substituem os dados da conta atual.
  ///
  /// Exceções inesperadas sem diagnóstico publicado e objetos da hierarquia
  /// Error continuam propagando com sua pilha.
  Future<void> _signIn() async {
    final current = controller.state;
    if (current.access != SessionAccess.entryRequired || current.isBusy) return;
    try {
      await controller.signIn();
    } on SessionIdentityChangedException {
      // A tentativa deixou de pertencer à identidade atual.
    } on SessionCacheWriteException {
      // O resultado válido permite entrada e o estado preserva o aviso.
    } on IdentityException {
      // A causa da tentativa atual está preservada no estado da sessão.
    } on Exception catch (error, stackTrace) {
      if (!identical(controller.state.failureDetails?.cause, error)) {
        Error.throwWithStackTrace(error, stackTrace);
      }
    }
  }

  /// Apresenta ação e mensagem pública sem iniciar operações no build.
  ///
  /// [context] fornece tema e configuração visual. O bloqueio acompanha
  /// o estado compartilhado; somente o callback do botão solicita entrada.
  @override
  Widget build(BuildContext context) {
    final message = sessionFailureMessage(state);
    return Scaffold(
      appBar: AppBar(title: const Text('Entrar no CApp')),
      body: SafeArea(
        child: Center(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(24),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 450),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  const Text(
                    'Entre com sua conta Microsoft para acessar o CApp.',
                  ),
                  const SizedBox(height: 24),
                  FilledButton(
                    onPressed: state.isBusy ? null : _signIn,
                    child: Text(
                      state.isBusy ? 'Entrando...' : 'Entrar com Microsoft',
                    ),
                  ),
                  if (message != null) ...[
                    const SizedBox(height: 16),
                    Semantics(
                      liveRegion: true,
                      child: Text(
                        message,
                        style: TextStyle(
                          color: Theme.of(context).colorScheme.error,
                        ),
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
