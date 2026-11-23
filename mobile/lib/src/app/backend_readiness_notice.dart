/// Ligação entre o diagnóstico global de prontidão e a apresentação compartilhada.
///
/// Esta biblioteca pertence à composição visual de app: observa o controlador e
/// traduz suas variantes em mensagem pública, progresso e intenção de repetir.
/// A aparência e a organização visual pertencem a StatusBanner, consumido pela
/// fachada de apresentação; seus componentes não conhecem o backend ou app.
///
/// CApp monta esta ligação no builder do MaterialApp, acima de sua navegação.
/// Estado inicial ou sucesso ocultam o aviso. Uma consulta em andamento informa
/// progresso; uma falha oferece nova tentativa sem expor causa e pilha técnicas.
///
/// A composição possui o controlador e inicia a primeira consulta. Este widget
/// apenas possui a inscrição visual criada por ListenableBuilder; não consulta
/// durante o build, encerra recursos ou decide o destino da sessão.
library;

import 'dart:async';

import 'package:flutter/widgets.dart';

import '../presentation/presentation.dart';
import 'backend_readiness_controller.dart';
import 'backend_readiness_state.dart';

/// Apresenta um diagnóstico de prontidão sem condicionar o uso de [child].
class BackendReadinessNotice extends StatelessWidget {
  /// Conecta o controlador iniciado à navegação pertencente ao MaterialApp.
  ///
  /// [controller] conserva coordenação e diagnósticos sob propriedade da
  /// composição. [child] recebe o espaço disponível; sua árvore é preservada
  /// quando o aviso aparece, muda de conteúdo ou é ocultado.
  const BackendReadinessNotice({
    super.key,
    required this.controller,
    required this.child,
  });

  /// Fonte do diagnóstico e da operação de nova tentativa.
  final BackendReadinessController controller;

  /// Interface apresentada independentemente da prontidão do servidor.
  final Widget child;

  /// Converte as variantes do diagnóstico em dados para o componente genérico.
  ///
  /// A mensagem não depende do SDK ou do texto da exceção original. Exceptions
  /// operacionais da nova tentativa são publicadas pelo controlador; Errors
  /// inesperados seguem seu contrato de propagação assíncrona.
  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller,
      child: child,
      builder: (context, child) {
        final state = controller.state;
        final message = switch (state) {
          BackendReadinessNotChecked() || BackendReadinessReady() => null,
          BackendReadinessChecking() =>
            'Verificando disponibilidade do servidor...',
          BackendReadinessFailed() =>
            'Não foi possível confirmar a disponibilidade do servidor. '
                'Algumas funcionalidades podem estar indisponíveis.',
        };

        return StatusBanner(
          message: message,
          isLoading: state is BackendReadinessChecking,
          onRetry: state is BackendReadinessFailed
              ? () => unawaited(controller.check())
              : null,
          child: child!,
        );
      },
    );
  }
}
