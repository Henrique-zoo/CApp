/// Composição visual dos destinos de entrada e da área principal.
///
/// Esta biblioteca pertence à camada app porque une a apresentação do módulo
/// de identidade ao tema e à navegação do aplicativo. Ela recebe o controlador
/// pronto; a escolha de SDKs e a construção de adaptadores pertencem à composição.
///
/// ## Fluxo de apresentação
///
/// CApp fornece ao SessionGate os builders de carregamento, entrada, área
/// principal e falha de restauração. O destino vem do estado publicado pelo
/// controlador. Uma conta Firebase conhecida continua na entrada enquanto não
/// houver verificação institucional válida na política local.
///
/// Cada builder monta seu próprio MaterialApp e Navigator dentro do ramo
/// identificado pelo gate. Quando o destino ou a conta muda, a navegação anterior
/// é descartada junto com sua subárvore.
///
/// ## Área principal atual
///
/// A tela inicial apresenta o perfil confirmado. NonStudentAffiliation acrescenta
/// um aviso informativo, sem transformar a resposta bem-sucedida em recusa.
/// Falha exclusiva de persistência acrescenta um aviso e preserva a entrada na
/// execução atual.
/// A ação de saída é injetada na Home. Falhas esperadas são apresentadas pelo
/// estado; a composição acompanha o Future sem depender do contexto da tela
/// depois que o logout troca o ramo da navegação.
///
/// A biblioteca não calcula validade, chama backend ou infere permissões gerais
/// a partir do vínculo estudantil. As funcionalidades principais do produto
/// podem ser compostas a partir desta área.
library;

import 'package:capp/src/presentation/presentation.dart';
import 'package:flutter/material.dart';

import '../modules/identity/identity.dart';
import '../modules/identity/identity_widgets.dart';
import 'session_gate.dart';
import 'theme.dart';

/// Apresenta a interface correspondente à situação atual da sessão.
///
/// Recebe um controlador já iniciado. Não cria adaptadores, inicia serviços
/// ou assume o encerramento da composição.
///
/// A área principal somente é montada após restauração de cache válido
/// ou conclusão bem-sucedida da verificação no backend.
class CApp extends StatelessWidget {
  /// Cria a interface com o controlador compartilhado da aplicação.
  const CApp({super.key, required this.sessionController});

  /// Fonte do estado e das operações de entrada.
  final SessionController sessionController;

  /// Solicita saída apenas para uma conta conhecida e sem outra operação.
  ///
  /// O controlador publica os diagnósticos das falhas esperadas. Contextos
  /// obsoletos e limpeza parcialmente concluída não geram Futures sem tratamento
  /// no callback do botão. Exceções sem diagnóstico publicado e Errors propagam.
  Future<void> _signOut() async {
    final current = sessionController.state;
    if (current is! SessionSignedIn || current.isBusy) return;
    try {
      await sessionController.signOut();
    } on SessionIdentityChangedException {
      // A operação deixou de pertencer à conta que originou o clique.
    } on SessionCacheRemovalException {
      // A autenticação já terminou; a entrada apresenta o aviso de limpeza.
    } on Exception catch (error, stackTrace) {
      if (!identical(sessionController.state.failureDetails?.cause, error)) {
        Error.throwWithStackTrace(error, stackTrace);
      }
    }
  }

  /// Monta o gate e a navegação correspondente ao ramo selecionado.
  ///
  /// [context] identifica a posição deste widget na árvore.
  /// Nenhuma requisição é iniciada durante a construção.
  @override
  Widget build(BuildContext context) {
    return SessionGate(
      controller: sessionController,
      restoringBuilder: (_) => _buildApplication(
        const Scaffold(body: Center(child: CircularProgressIndicator())),
      ),
      entryBuilder: (_, state) => _buildApplication(
        InstitutionalLoginPage(controller: sessionController, state: state),
      ),
      signedInBuilder: (_, state) {
        final cause = state.failureDetails?.cause;
        final warning =
            cause is SessionCacheWriteException ||
                state.failure == SessionFailure.signOutFailed
            ? cause
            : null;

        return _buildApplication(
          TemporaryWarning(
            warningId: warning,
            message: warning == null ? null : sessionFailureMessage(state),
            child: HomePage(state: state, onSignOut: _signOut),
          ),
        );
      },
      restorationFailedBuilder: (_, state) => _buildApplication(
        const Scaffold(
          body: SafeArea(
            child: Center(
              child: Padding(
                padding: EdgeInsets.all(24),
                child: Text(
                  'Não foi possível recuperar sua sessão. '
                  'Feche e abra o aplicativo para tentar novamente.',
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }

  /// Constrói tema e navegação para a interface selecionada.
  ///
  /// [home] pertence ao ramo atual do gate. Quando esse ramo muda,
  /// sua subárvore e seu Navigator são descartados.
  Widget _buildApplication(Widget home) {
    return MaterialApp(title: 'CApp', theme: createDarkTheme(), home: home);
  }
}
