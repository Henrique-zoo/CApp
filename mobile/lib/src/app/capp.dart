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
///
/// A biblioteca não calcula validade, chama backend ou infere permissões gerais
/// a partir do vínculo estudantil. As funcionalidades principais do produto
/// podem ser compostas a partir desta área.
library;

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
      signedInBuilder: (_, state) => _buildApplication(_buildHome(state)),
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
    return MaterialApp(
      title: 'CApp',
      theme: createDarkTheme(),
      home: home,
    );
  }

  /// Constrói o ponto inicial da área principal autenticada.
  ///
  /// [state] é fornecido pelo ramo do gate que exige verificação válida.
  /// Por esse contrato, seu perfil está disponível nesta construção.
  ///
  /// O vínculo não estudantil gera uma mensagem informativa e não impede
  /// a montagem desta área. As funcionalidades principais podem ser
  /// acrescentadas a partir desta tela.
  ///
  /// Falha exclusiva de persistência é apresentada sem desfazer a entrada
  /// já permitida pela resposta válida do backend.
  Widget _buildHome(SessionSignedIn state) {
    final profile = state.verification!.profile;

    return Scaffold(
      appBar: AppBar(title: const Text('CApp')),
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.all(24),
          children: [
            Text(
              profile.name == null
                  ? 'Bem-vindo ao CApp'
                  : 'Olá, ${profile.name}',
            ),
            if (profile.affiliation is NonStudentAffiliation) ...[
              const SizedBox(height: 16),
              const Text(
                'Você não foi reconhecido como estudante da sua '
                'instituição. Isso significa que você não terá acesso '
                'a diversas funcionalidades exclusivas de estudantes.',
              ),
            ],
            if (state.failure == SessionFailure.cacheWriteFailed) ...[
              const SizedBox(height: 16),
              const Text(
                'Não foi possível salvar seus dados neste dispositivo. '
                'Pode ser necessário entrar novamente ao reabrir o app.',
              ),
            ],
          ],
        ),
      ),
    );
  }
}
