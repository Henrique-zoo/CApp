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
/// SessionAccess distingue restauração, entrada pendente, conclusão de perfil,
/// prontidão e falha de restauração. O ramo pronto exige SessionSignedIn com verificação presente;
/// combinações incompatíveis representam erro de programação e lançam StateError.
/// Depois de perder a autenticação durante uma saída, a operação signingOut
/// seleciona uma interface de espera até concluir a tentativa de limpeza. Essa
/// atividade não concede acesso nem mantém uma identidade que o SDK já encerrou.
///
/// ## Identidade das subárvores
///
/// Cada destino possui uma chave, e o ramo pronto também distingue o UID.
/// O mesmo ramo de entrada é preservado durante Microsoft e backend; uma mudança
/// de destino ou conta descarta sua subárvore. A navegação precisa ficar dentro
/// dela para participar desse descarte, como ocorre em CApp.
/// A espera de saída possui um ramo próprio: descarta a área anterior e impede
/// montar o seletor com uma lista ainda em processo de remoção.
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
    required this.signingOutBuilder,
    required this.entryBuilder,
    this.profileBuilder,
    required this.signedInBuilder,
    required this.restorationFailedBuilder,
  });

  /// Fonte do estado e das mudanças de destino.
  final SessionController controller;

  /// Interface durante a restauração da autenticação ou do cache.
  final WidgetBuilder restoringBuilder;

  /// Espera da saída após encerrar a autenticação, antes de concluir a limpeza.
  ///
  /// O controlador mantém signingOut até a tentativa terminar. Não monta a
  /// entrada nem reapresenta os dados da área autenticada nesse intervalo.
  final SessionStateWidgetBuilder<SessionSignedOut> signingOutBuilder;

  /// Entrada, com ou sem autenticação Firebase já estabelecida.
  final SessionStateWidgetBuilder<SessionState> entryBuilder;

  /// Conclusão do cadastro depois da confirmação institucional.
  final SessionStateWidgetBuilder<SessionSignedIn>? profileBuilder;

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
  /// A operação de saída tem precedência sobre a entrada quando o UID já saiu;
  /// a apresentação acompanha a atividade sem recalcular prontidão ou validade.
  Widget _buildForState(BuildContext context, SessionState state) {
    if (state is SessionSignedOut &&
        state.operation == SessionOperation.signingOut) {
      return KeyedSubtree(
        key: const ValueKey('signing-out'),
        child: signingOutBuilder(context, state),
      );
    }

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
      case SessionAccess.profileRequired:
        if (state is! SessionSignedIn) {
          throw StateError('Perfil sem identidade');
        }
        return KeyedSubtree(
          key: ValueKey(('profile', state.firebaseUid)),
          child:
              profileBuilder?.call(context, state) ??
              entryBuilder(context, state),
        );
      case SessionAccess.ready:
        if (state is! SessionSignedIn ||
            state.verification == null ||
            state.user?.status != InternalUserStatus.active) {
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
