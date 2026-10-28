/// Propriedade da composição durante o ciclo de vida da interface.
///
/// Esta biblioteca resolve a diferença entre recursos duradouros da aplicação
/// e widgets que podem ser removidos ou substituídos. AppRoot recebe uma composição
/// já iniciada e passa a solicitar seu encerramento quando deixa de utilizá-la.
///
/// ## Montagem e substituição
///
/// O proprietário monta SessionLifecycleObserver ao redor de CApp. O observador
/// informa retomadas ao controlador, e CApp apresenta o destino da sessão.
/// Nenhum desses passos recria os adaptadores a cada build.
///
/// Reconstruções com a mesma composição preservam a subárvore. Uma composição
/// com identidade diferente recebe outra chave, recria a interface e inicia o
/// encerramento da anterior. As duas composições devem possuir recursos cujo
/// descarte não invalide a outra.
///
/// ## Descarte assíncrono
///
/// O callback dispose de um widget é síncrono. AppRoot acompanha separadamente
/// o Future de encerramento e encaminha sua exceção e pilha a FlutterError.
/// Não acessa o estado do widget depois de aguardar esse Future.
///
/// O encerramento forçado do processo pode não chamar esse callback. A
/// persistência necessária à restauração acontece durante as operações de sessão,
/// e não depende do descarte visual. Esta biblioteca não inicia login ou logout.
library;

import 'dart:async';

import 'package:flutter/widgets.dart';

import 'app_dependencies.dart';
import 'capp.dart';
import 'session_lifecycle_observer.dart';

/// Widget que assume o encerramento das dependências da aplicação.
///
/// O chamador deve construir e iniciar [dependencies] antes de montar este
/// widget. A transferência de propriedade significa que a mesma composição
/// não deve ser compartilhada entre proprietários independentes nem reutilizada
/// depois que seu encerramento tiver começado.
///
/// Reconstruções com a mesma instância preservam as dependências e a interface.
/// A substituição por outra instância inicia o encerramento da anterior e
/// recria a subárvore de [CApp] com a nova composição.
///
/// Não inicia serviços, executa logout ou remove o cache institucional.
/// Também não observa o estado da sessão nem decide qual tela apresentar.
class AppRoot extends StatefulWidget {
  /// Cria o proprietário de uma composição já iniciada.
  ///
  /// [dependencies] deve permanecer utilizável enquanto estiver associada
  /// a este widget. Seu encerramento será solicitado na substituição da
  /// composição ou no descarte deste proprietário.
  ///
  /// [key] participa da identificação do widget na árvore Flutter.
  const AppRoot({super.key, required this.dependencies});

  /// Composição utilizada pela interface e pertencente a este proprietário.
  ///
  /// Sua identidade determina se uma atualização preserva a composição atual
  /// ou inicia seu encerramento.
  final AppDependencies dependencies;

  /// Cria o estado responsável pela montagem e pelo descarte da composição.
  @override
  State<AppRoot> createState() => _AppRootState();
}

/// Coordena a troca de composição e seu encerramento assíncrono.
///
/// Os callbacks do framework permanecem síncronos. Os Futures de encerramento
/// são acompanhados por [_disposeDependencies], que encaminha suas falhas
/// ao mecanismo de diagnóstico do Flutter.
class _AppRootState extends State<AppRoot> {
  /// Encerra a composição anterior quando sua instância é substituída.
  ///
  /// [oldWidget] contém a configuração anteriormente associada a este estado.
  /// A comparação por identidade evita descartar recursos em reconstruções
  /// que continuam usando a mesma composição.
  ///
  /// O encerramento anterior pode ocorrer enquanto a nova interface é montada.
  /// As composições devem, portanto, possuir recursos cujo descarte não invalide
  /// a composição substituta.
  ///
  /// Não chama `setState`: o framework executa [build] após esta atualização.
  @override
  void didUpdateWidget(covariant AppRoot oldWidget) {
    super.didUpdateWidget(oldWidget);

    if (!identical(oldWidget.dependencies, widget.dependencies)) {
      unawaited(_disposeDependencies(oldWidget.dependencies));
    }
  }

  /// Monta a interface com o controlador da composição atual.
  ///
  /// A identidade da composição preserva a interface em reconstruções
  /// normais e a recria quando as dependências são substituídas.
  ///
  /// O controlador já deve estar iniciado pelo chamador.
  @override
  Widget build(BuildContext context) {
    return SessionLifecycleObserver(
      key: ObjectKey(widget.dependencies),
      controller: widget.dependencies.sessionController,
      child: CApp(sessionController: widget.dependencies.sessionController),
    );
  }

  /// Solicita o encerramento da composição atual ao descartar este estado.
  ///
  /// O callback é síncrono: o framework não aguarda a conclusão do descarte
  /// das dependências. Seu Future é acompanhado separadamente, sem acessar
  /// este estado após a conclusão.
  ///
  /// Não representa um callback garantido de fechamento do processo.
  /// Dados necessários à restauração devem ser persistidos durante as
  /// operações correspondentes, sem depender deste método.
  @override
  void dispose() {
    unawaited(_disposeDependencies(widget.dependencies));

    super.dispose();
  }

  /// Aguarda o encerramento de [dependencies] e reporta eventuais falhas.
  ///
  /// O encerramento efetivo pertence a [AppDependencies.dispose], que
  /// compartilha o mesmo Future entre chamadas repetidas.
  ///
  /// Uma falha é encaminhada a [FlutterError.reportError] com a exceção
  /// e a pilha originais. Este método não executa nova tentativa nem
  /// apresenta uma mensagem na interface.
  ///
  /// Com o tratamento padrão do Flutter, a falha reportada não é relançada.
  /// Um manipulador de erros personalizado pode, entretanto, lançar sua
  /// própria exceção durante o processamento do diagnóstico.
  Future<void> _disposeDependencies(AppDependencies dependencies) async {
    try {
      await dependencies.dispose();
    } catch (error, stackTrace) {
      FlutterError.reportError(
        FlutterErrorDetails(
          exception: error,
          stack: stackTrace,
          library: 'capp',
          context: ErrorDescription('ao encerrar dependências da aplicação'),
        ),
      );
    }
  }
}
