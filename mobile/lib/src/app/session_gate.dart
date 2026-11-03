/// Seleção da subárvore visual a partir do destino publicado pela sessão.
///
/// O gate separa a decisão da aplicação de sua representação em widgets.
/// SessionController calcula a prontidão; esta biblioteca traduz seu resultado
/// em um dos builders fornecidos pela composição visual. Ela pertence à camada
/// app porque escolhe entre áreas do aplicativo, sem executar o caso de uso.
///
/// ## Observação e seleção
///
/// StreamBuilder acompanha controller.states, usando controller.state na
/// primeira construção. A seleção lê a fotografia atual do controlador, evitando
/// que uma notificação assíncrona antiga reaplique um destino ultrapassado.
///
/// SessionAccess distingue restauração, entrada pendente, prontidão e falha de
/// restauração. O ramo pronto exige SessionSignedIn com verificação presente;
/// combinações incompatíveis representam erro de programação e lançam StateError.
///
/// ## Identidade das subárvores
///
/// Cada destino possui uma chave, e o ramo pronto também distingue o UID.
/// O mesmo ramo de entrada é preservado durante Microsoft e backend; uma mudança
/// de destino ou conta descarta sua subárvore. A navegação precisa ficar dentro
/// dela para participar desse descarte, como ocorre em CApp.
///
/// O gate não consulta relógio, SDK, cache ou rede. Não inicia nem encerra o
/// controlador; possui somente a inscrição visual criada por StreamBuilder.
library;

import 'package:flutter/widgets.dart';

import '../modules/identity/identity.dart';

/// Constrói widgets para uma fotografia específica da sessão.
///
/// O callback recebe contexto e estado e pode ser executado repetidamente.
/// Deve construir a interface sem iniciar operações durante o build.
typedef SessionStateWidgetBuilder<T extends SessionState> = Widget Function(
  BuildContext context,
  T state,
);

/// Apresenta o destino determinado pelo controlador.
///
/// Mantém a interface de entrada enquanto a Microsoft autentica e o backend
/// verifica. Estabelecer um UID isoladamente não seleciona a área principal.
///
/// Cada ramo possui identidade própria. Coloque seu Navigator dentro do ramo
/// para que uma troca de sessão ou expiração descarte a navegação anterior.
/// O ramo autenticado também distingue o UID, sem interpretar vínculo estudantil.
///
/// Não inicia nem encerra o controlador. StreamBuilder possui a inscrição
/// visual e a cancela quando a subárvore é descartada.
class SessionGate extends StatelessWidget {
  /// Conecta a apresentação a um controlador já iniciado.
  ///
  /// Cada builder recebe o estado associado ao destino correspondente.
  /// A restauração inclui autenticação e cache; falha de acompanhamento
  /// não é convertida em logout.
  const SessionGate({
    super.key,
    required this.controller,
    required this.restoringBuilder,
    required this.entryBuilder,
    required this.signedInBuilder,
    required this.restorationFailedBuilder,
  });

  /// Fonte do estado e das mudanças de destino.
  final SessionController controller;

  /// Interface durante a restauração da autenticação ou do cache.
  final WidgetBuilder restoringBuilder;

  /// Entrada, com ou sem autenticação Firebase já estabelecida.
  final SessionStateWidgetBuilder<SessionState> entryBuilder;

  /// Área principal com identidade e verificação válidas na política local.
  final SessionStateWidgetBuilder<SessionSignedIn> signedInBuilder;

  /// Interface da falha de acompanhamento com o diagnóstico preservado.
  final SessionStateWidgetBuilder<SessionRestorationFailed>
  restorationFailedBuilder;

  /// Observa transições usando o estado atual como fotografia inicial.
  ///
  /// A chave reinicia a observação ao substituir o controlador. Notificações
  /// assíncronas antigas não substituem o estado atual por snapshots anteriores.
  /// Erros inesperados do stream propagam com a pilha disponível.
  @override
  Widget build(BuildContext context) {
    return StreamBuilder<SessionState>(
      key: ObjectKey(controller),
      stream: controller.states,
      initialData: controller.state,
      builder: (context, snapshot) {
        if (snapshot.hasError) {
          Error.throwWithStackTrace(
            snapshot.error!,
            snapshot.stackTrace ?? StackTrace.current,
          );
        }
        return _buildForState(context, controller.state);
      },
    );
  }

  /// Seleciona o ramo sem recalcular a política da sessão.
  ///
  /// Estados incompatíveis com seu destino representam erro de programação.
  /// A mesma chave de entrada preserva a tela durante Microsoft e backend.
  Widget _buildForState(BuildContext context, SessionState state) {
    switch (state.access) {
      case SessionAccess.restoring:
        return KeyedSubtree(
          key: const ValueKey('restoring'),
          child: restoringBuilder(context),
        );
      case SessionAccess.entryRequired:
        return KeyedSubtree(
          key: const ValueKey('entry'),
          child: entryBuilder(context, state),
        );
      case SessionAccess.ready:
        if (state is! SessionSignedIn || state.verification == null) {
          throw StateError('Sessão pronta sem identidade e perfil');
        }
        return KeyedSubtree(
          key: ValueKey((SessionSignedIn, state.firebaseUid)),
          child: signedInBuilder(context, state),
        );
      case SessionAccess.restorationFailed:
        if (state is! SessionRestorationFailed) {
          throw StateError('Falha de restauração sem diagnóstico');
        }
        return KeyedSubtree(
          key: const ValueKey('restoration-failed'),
          child: restorationFailedBuilder(context, state),
        );
    }
  }
}
