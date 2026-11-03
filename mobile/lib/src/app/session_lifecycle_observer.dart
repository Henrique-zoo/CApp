/// Adaptação do ciclo de vida Flutter à coordenação da sessão.
///
/// Timers e notificações podem deixar de acompanhar o tempo real enquanto o
/// processo está suspenso. Esta biblioteca encaminha a retomada ao controlador,
/// que reavalia a identidade e a validade usando sua própria política.
///
/// ## Fluxo do observador
///
/// O widget registra um WidgetsBindingObserver ao entrar na árvore. Ao receber
/// AppLifecycleState.resumed, chama refresh no controlador atualmente associado.
/// O controlador pode publicar outro destino, que será apresentado pelo gate.
///
/// Pausas e estados intermediários não são interpretados como logout. Ao sair
/// da árvore, o widget remove apenas seu registro de observação e mantém o child
/// sem alteração durante a apresentação.
///
/// ## Fronteira arquitetural
///
/// O observador pertence à camada app por conhecer eventos Flutter e a API pública
/// do módulo de identidade. A aplicação permanece independente do framework.
/// A biblioteca não possui o controlador, não abre login, não consulta o backend
/// e não calcula datas de expiração por conta própria.
library;

import 'package:flutter/widgets.dart';

import '../modules/identity/identity.dart';

/// Notifica o controlador quando a aplicação retorna ao primeiro plano.
///
/// Não possui a composição e não executa descarte ao pausar o aplicativo.
/// O observador é removido quando este widget deixa a árvore.
class SessionLifecycleObserver extends StatefulWidget {
  /// Recebe o controlador já iniciado e a interface que será apresentada.
  const SessionLifecycleObserver({
    super.key,
    required this.controller,
    required this.child,
  });

  /// Coordenador que reavalia identidade e validade na retomada.
  final SessionController controller;

  /// Subárvore apresentada sem alterações pelo observador.
  final Widget child;

  /// Cria o registro de observação do binding Flutter.
  @override
  State<SessionLifecycleObserver> createState() =>
      _SessionLifecycleObserverState();
}

/// Mantém exclusivamente o registro no binding da plataforma.
class _SessionLifecycleObserverState extends State<SessionLifecycleObserver>
    with WidgetsBindingObserver {
  /// Registra as notificações sem iniciar a sessão ou consultar o backend.
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  /// Encaminha a retomada ao controlador atualmente recebido pelo widget.
  ///
  /// Pausas e estados intermediários não representam logout ou descarte.
  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      widget.controller.refresh();
    }
  }

  /// Apresenta a subárvore recebida, sem operações durante a construção.
  @override
  Widget build(BuildContext context) => widget.child;

  /// Remove o registro sem encerrar o controlador ou alterar a autenticação.
  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }
}
