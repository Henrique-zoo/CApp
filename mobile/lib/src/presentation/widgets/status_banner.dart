/// Apresentação compartilhada de progresso ou aviso acima de uma interface.
///
/// Este componente recebe dados visuais e um callback opcional. Não interpreta
/// estados de aplicação, observa controladores ou inicia operações externas.
/// A tradução de diagnósticos para esses dados pertence ao consumidor.
///
/// ## Organização visual
///
/// Uma mensagem presente reserva espaço no topo; isLoading seleciona progresso,
/// e a ausência de carregamento apresenta um aviso com possível nova tentativa.
/// Mensagem nula oculta o banner. A estrutura que contém child permanece estável
/// em todas essas situações, preservando formulários e demais estados locais.
///
/// O banner consome a área segura superior quando está visível e remove esse
/// padding da interface abaixo. Oculto, devolve o padding ao próprio child.
/// O proprietário deve fornecer altura limitada, normalmente no builder de um
/// MaterialApp. Não há Scaffold, Navigator ou inscrição pertencente ao componente.
library;

import 'package:flutter/material.dart';

/// Reserva espaço para uma informação sem bloquear a interface de [child].
class StatusBanner extends StatelessWidget {
  /// Recebe a mensagem pública e a interface que deve continuar disponível.
  ///
  /// [message] nula representa ausência de aviso. Nesse caso, [isLoading] deve
  /// ser false e [onRetry] nulo. [isLoading] apresenta progresso em vez de ícone
  /// de aviso. [onRetry], quando presente, oferece uma ação desabilitada durante
  /// carregamento; seu proprietário acompanha a conclusão e as falhas da operação.
  ///
  /// [child] deve caber na altura restante. Nenhum callback é executado no build.
  const StatusBanner({
    super.key,
    required this.message,
    required this.child,
    this.isLoading = false,
    this.onRetry,
  }) : assert(message != null || (!isLoading && onRetry == null));

  /// Texto público pronto, ou nulo quando o banner deve permanecer oculto.
  final String? message;

  /// Interface apresentada sem alterar sua identidade nas mudanças do banner.
  final Widget child;

  /// Seleciona o indicador de progresso e impede repetir uma ação em andamento.
  final bool isLoading;

  /// Intenção de nova tentativa; ausência do callback omite o botão.
  final VoidCallback? onRetry;

  /// Compõe o banner e reserva o restante da altura para a interface existente.
  ///
  /// A estrutura de child permanece a mesma mesmo com mensagem nula. A área
  /// segura superior é consumida uma vez, e a mensagem pode crescer verticalmente
  /// conforme a largura disponível e a escala de texto configurada pelo usuário.
  @override
  Widget build(BuildContext context) {
    final message = this.message;
    final colors = Theme.of(context).colorScheme;
    final foreground = isLoading
        ? colors.onSurface
        : colors.onTertiaryContainer;

    return Column(
      children: [
        if (message != null)
          Material(
            color: isLoading
                ? colors.surfaceContainer
                : colors.tertiaryContainer,
            child: SafeArea(
              bottom: false,
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: 16,
                  vertical: 8,
                ),
                child: Semantics(
                  liveRegion: true,
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Row(
                        children: [
                          if (isLoading)
                            const SizedBox.square(
                              dimension: 18,
                              child: CircularProgressIndicator(strokeWidth: 2),
                            )
                          else
                            Icon(
                              Icons.warning_amber_rounded,
                              color: foreground,
                            ),
                          const SizedBox(width: 12),
                          Expanded(
                            child: Text(
                              message,
                              style: Theme.of(context).textTheme.bodySmall
                                  ?.copyWith(color: foreground),
                            ),
                          ),
                        ],
                      ),
                      if (onRetry != null)
                        Align(
                          alignment: Alignment.centerRight,
                          child: TextButton(
                            onPressed: isLoading ? null : onRetry,
                            style: TextButton.styleFrom(
                              foregroundColor: foreground,
                            ),
                            child: const Text('Tentar novamente'),
                          ),
                        ),
                    ],
                  ),
                ),
              ),
            ),
          ),
        Expanded(
          child: MediaQuery.removePadding(
            context: context,
            removeTop: message != null,
            child: child,
          ),
        ),
      ],
    );
  }
}
