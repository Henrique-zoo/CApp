/// Apresentação da operação única de entrada institucional.
///
/// Esta biblioteca apresenta conexão, recuperação e confirmação de conta.
/// Ela recebe um controlador e sua fotografia atual, sem criar adaptadores,
/// observar Firebase diretamente ou montar requisições HTTP.
///
/// ## Interação apresentada
///
/// Durante a restauração ou uma operação em andamento, a tela informa atividade
/// e impede uma nova tentativa. Quando a entrada está pendente, oferece uma única
/// ação: entrar com Microsoft quando não há conta; continuar com Microsoft
/// quando a obtenção silenciosa exige interação; ou tentar novamente após uma
/// falha operacional. Todas solicitam signIn ao controlador, que determina se
/// a operação admite autorização interativa. A verificação fica nos bastidores.
///
/// Uma tentativa automática pode precisar de consentimento ou falhar na rede.
/// A tela permite nova tentativa por ação explícita, além das tentativas
/// silenciosas agendadas pela aplicação. Falha de rede não pede novo login.
/// Não inicia renovação durante build nem mantém relógio próprio de repetição.
/// O gate mantém esta tela enquanto a entrada depende de uma verificação válida
/// e a substitui quando o controlador publica o destino pronto.
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
import 'package:flutter_svg/svg.dart';

import '../application/errors/identity_exception.dart';
import '../application/errors/session_cache_write_exception.dart';
import '../application/errors/session_identity_changed_exception.dart';
import '../application/session_controller.dart';
import '../application/session_state.dart';
import 'session_failure_message.dart';

/// Apresenta entrada pendente, operação em andamento e eventuais falhas.
///
/// Uma autenticação Firebase sem perfil válido usa a mesma ação de entrada.
/// O controlador escolhe entre login e renovação da verificação institucional.
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
      bottomNavigationBar: SafeArea(
        minimum: const EdgeInsets.all(24),
        child: Align(
          alignment: Alignment.bottomCenter,
          heightFactor: 1,
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(24),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 450),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Text(
                    state is SessionSignedIn
                        ? state.isBusy
                              ? 'Estamos conectando sua conta ao CApp.'
                              : state.requiresMicrosoftInteraction
                              ? 'Continue com sua conta institucional da Microsoft.'
                              : 'Sua conta continua conectada.'
                        : 'Entre com sua conta institucional da Microsoft '
                              'para acessar o CApp.',
                  ),
                  const SizedBox(height: 24),
                  _buildSignInButton(),
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

  /// Apresenta ação Microsoft ou repetição de conexão conforme o estado.
  ///
  /// O estado ocupado desabilita novas interações e altera o texto.
  /// A ação utiliza [_signIn], que mantém as verificações da sessão.
  ///
  /// O logo preserva suas cores originais. Sua descrição semântica é omitida
  /// porque o texto do botão já identifica o provedor.
  ///
  /// O arquivo deve existir e estar declarado no pubspec.yaml para que
  /// o Flutter consiga carregá-lo.
  Widget _buildSignInButton() {
    final microsoftAction =
        state is! SessionSignedIn || state.requiresMicrosoftInteraction;

    return FilledButton.icon(
      onPressed: state.isBusy ? null : _signIn,
      icon: microsoftAction
          ? SvgPicture.asset(
              'assets/branding/microsoft.svg',
              width: 20,
              height: 20,
              excludeFromSemantics: true,
            )
          : const Icon(Icons.refresh),
      label: Text(switch (state.operation) {
        SessionOperation.signingOut => 'Saindo...',
        SessionOperation.signingIn => 'Entrando...',
        SessionOperation.verifyingInstitution => 'Conectando...',
        SessionOperation.completingProfile => 'Concluindo perfil...',
        SessionOperation.restoringAuthentication ||
        SessionOperation.restoringCache => 'Restaurando...',
        SessionOperation.idle =>
          state is! SessionSignedIn
              ? 'Entrar com Microsoft'
              : state.requiresMicrosoftInteraction
              ? 'Continuar com Microsoft'
              : 'Tentar novamente',
      }),
      style: FilledButton.styleFrom(
        backgroundColor: const Color(0xFF2F2F2F),
        foregroundColor: Colors.white,
        disabledBackgroundColor: const Color(0xFF252525),
        disabledForegroundColor: const Color(0xFFAAAAAA),
        minimumSize: const Size(0, 52),
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 16),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        side: const BorderSide(color: Color(0xFF666666)),
        textStyle: const TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
      ),
    );
  }
}
