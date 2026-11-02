/// Apresentação temporária de avisos recebidos pela composição visual.
///
/// Recebe a identidade de uma ocorrência e sua mensagem pública.
/// A mesma ocorrência não é reapresentada em reconstruções comuns.
///
/// O componente deve estar abaixo de um MaterialApp e envolver uma
/// subárvore que contenha um Scaffold. O ScaffoldMessenger possui
/// a apresentação, animação e duração do aviso.
///
/// Não observa controladores, interpreta exceções ou altera a sessão.
library;

import 'package:flutter/material.dart';

/// Apresenta um aviso temporário quando recebe uma nova ocorrência.
///
/// A identidade e a mensagem devem estar ambas presentes ou ambas ausentes.
/// Cada nova ocorrência deve possuir uma identidade própria e estável.
class TemporaryWarning extends StatefulWidget {
  /// Recebe a ocorrência, sua mensagem pública e a interface apresentada.
  ///
  /// [warningId] identifica a ocorrência por identidade de objeto.
  /// [message] contém somente o texto destinado ao usuário.
  /// [child] deve conter o Scaffold que apresentará o aviso.
  ///
  /// Ausência de aviso é representada pelos dois parâmetros nulos.
  const TemporaryWarning({
    super.key,
    required this.warningId,
    required this.message,
    required this.child,
  }) : assert((warningId == null) == (message == null));

  /// Identidade estável da ocorrência, ou nulo quando não há aviso.
  final Object? warningId;

  /// Mensagem pública associada à ocorrência.
  final String? message;

  /// Interface apresentada independentemente da existência de aviso.
  final Widget child;

  /// Cria o estado responsável pelo efeito visual.
  @override
  State<TemporaryWarning> createState() => _TemporaryWarningState();
}

/// Agenda os avisos depois da montagem do Scaffold.
class _TemporaryWarningState extends State<TemporaryWarning> {
  /// Invalida callbacks quando o aviso muda antes de serem executados.
  int _revision = 0;

  /// Considera também um aviso já presente na montagem inicial.
  @override
  void initState() {
    super.initState();
    _scheduleWarning();
  }

  /// Agenda outra apresentação quando a ocorrência ou seu texto muda.
  ///
  /// Reconstruções que preservam ambos não repetem o aviso.
  @override
  void didUpdateWidget(covariant TemporaryWarning oldWidget) {
    super.didUpdateWidget(oldWidget);

    if (!identical(oldWidget.warningId, widget.warningId) ||
        oldWidget.message != widget.message) {
      _scheduleWarning();
    }
  }

  /// Agenda a apresentação para depois do frame atual.
  ///
  /// Callbacks obsoletos ou posteriores ao descarte são ignorados.
  /// A ausência de ScaffoldMessenger ou Scaffold representa uso
  /// incorreto do componente e segue os diagnósticos do Flutter.
  void _scheduleWarning() {
    final revision = ++_revision;
    final message = widget.message;

    if (widget.warningId == null || message == null) return;

    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || revision != _revision) return;

      final colors = Theme.of(context).colorScheme;

      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          behavior: SnackBarBehavior.floating,
          duration: const Duration(seconds: 6),
          showCloseIcon: true,
          closeIconColor: colors.onTertiaryContainer,
          backgroundColor: colors.tertiaryContainer,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
          ),
          content: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Icon(
                Icons.warning_amber_rounded,
                color: colors.onTertiaryContainer,
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Text(
                  message,
                  style: TextStyle(color: colors.onTertiaryContainer),
                ),
              ),
            ],
          ),
        ),
      );
    });
  }

  /// Apresenta a interface recebida, sem iniciar efeitos no build.
  @override
  Widget build(BuildContext context) => widget.child;
}
