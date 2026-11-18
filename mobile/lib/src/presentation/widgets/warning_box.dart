/// Aviso incorporado ao layout e independente dos módulos do domínio.
///
/// Recebe uma mensagem pública pronta e utiliza as cores do tema local.
/// Não interpreta falhas ou inicia operações; o consumidor determina quando
/// apresentar o aviso e por quanto tempo ele permanece na árvore.
library;

import 'package:flutter/material.dart';

/// Apresenta uma mensagem informativa com ícone e contraste do tema.
class WarningBox extends StatelessWidget {
  /// Recebe o texto público de [message], sem interpretar seu significado.
  const WarningBox({super.key, required this.message});

  /// Texto destinado ao usuário, produzido pelo consumidor do componente.
  final String message;

  /// Constrói o aviso respeitando a largura disponível e o esquema de cores.
  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;

    return Card(
      color: colors.tertiaryContainer,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
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
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: colors.onTertiaryContainer,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
