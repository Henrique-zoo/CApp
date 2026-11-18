/// Indicação textual de atividade com reticências animadas.
///
/// Durante operações assíncronas, uma mensagem curta pode permanecer na tela
/// enquanto o trabalho termina. Este componente alterna a visibilidade dos
/// pontos, sem iniciar operações ou decidir quanto tempo a espera deve durar.
/// A composição e as páginas fornecem o texto e removem o widget ao concluir.
///
/// Os três pontos sempre reservam o mesmo espaço, mantendo a palavra parada.
/// Leitores de tela recebem somente a mensagem, sem anunciar cada mudança de
/// pontuação. A animação pausa quando desabilitada nas preferências do sistema
/// ou quando a subárvore está inativa; o timer é cancelado no descarte.
library;

import 'dart:async';

import 'package:flutter/widgets.dart';

/// Apresenta, por exemplo, Saindo..., Saindo., Saindo.. e repete o ciclo.
class AnimatedEllipsisText extends StatefulWidget {
  /// Recebe a mensagem sem reticências, herdando o estilo de texto do contexto.
  const AnimatedEllipsisText(this.text, {super.key});

  /// Mensagem estável, também utilizada como rótulo de acessibilidade.
  final String text;

  @override
  State<AnimatedEllipsisText> createState() => _AnimatedEllipsisTextState();
}

class _AnimatedEllipsisTextState extends State<AnimatedEllipsisText> {
  Timer? _timer;
  int _dots = 3;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _timer?.cancel();

    _timer = null;

    final disabled = MediaQuery.maybeOf(context)?.disableAnimations ?? false;

    if (disabled || !TickerMode.valuesOf(context).enabled) {
      _dots = 3;

      return;
    }

    _timer = Timer.periodic(const Duration(milliseconds: 500), (_) {
      setState(() => _dots = _dots % 3 + 1);
    });
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Text.rich(
    TextSpan(
      children: [
        TextSpan(text: widget.text),
        for (var i = 1; i <= 3; i++)
          TextSpan(
            text: '.',
            style: i <= _dots
                ? null
                : const TextStyle(color: Color(0x00000000)),
          ),
      ],
    ),
    semanticsLabel: widget.text,
  );
}
