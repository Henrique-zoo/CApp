/// Cartão expansível que apresenta uma pergunta e resposta do FAQ do CA.
library;

import 'package:flutter/material.dart';

import '../domain/entities/faq_entry.dart';

/// Exibe uma entrada de FAQ com expansão inline para leitura completa da resposta.
class FaqEntryCard extends StatelessWidget {
  /// Cria o cartão expansível para [entry].
  const FaqEntryCard({
    super.key,
    required this.entry,
    required this.isExpanded,
    required this.onToggle,
    this.canManage = false,
    this.isSubmitting = false,
    this.onEdit,
    this.onDelete,
  });

  /// Pergunta frequente exibida neste cartão.
  final FaqEntry entry;

  /// Controla se a resposta completa e metadados estão visíveis.
  final bool isExpanded;

  /// Alterna a expansão deste cartão ao tocar no cabeçalho.
  final VoidCallback onToggle;

  /// Exibe ações rápidas de edição e remoção para gestão do CA.
  final bool canManage;

  /// Desabilita ações de gestão enquanto outra operação está em andamento.
  final bool isSubmitting;

  /// Callback acionado para editar esta pergunta frequente.
  final VoidCallback? onEdit;

  /// Callback acionado para solicitar a remoção desta pergunta frequente.
  final VoidCallback? onDelete;

  String _formatUpdatedAt(BuildContext context, DateTime date) {
    final local = date.toLocal();
    final strings = MaterialLocalizations.of(context);
    final time = strings.formatTimeOfDay(
      TimeOfDay.fromDateTime(local),
      alwaysUse24HourFormat: true,
    );
    final day = local.day.toString().padLeft(2, '0');
    final month = local.month.toString().padLeft(2, '0');
    return '$day/$month/${local.year} às $time';
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;
    final category = entry.category;

    return Card(
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(16),
        side: BorderSide(
          color: isExpanded
              ? colors.primary
              : colors.outlineVariant.withValues(alpha: 0.6),
        ),
      ),
      clipBehavior: Clip.antiAlias,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Semantics(
            button: true,
            expanded: isExpanded,
            child: InkWell(
              onTap: onToggle,
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          if (category != null && category.isNotEmpty) ...[
                            Container(
                              padding: const EdgeInsets.symmetric(
                                horizontal: 10,
                                vertical: 4,
                              ),
                              decoration: BoxDecoration(
                                color: colors.secondaryContainer,
                                borderRadius: BorderRadius.circular(8),
                              ),
                              child: Text(
                                category,
                                style: theme.textTheme.labelSmall?.copyWith(
                                  color: colors.onSecondaryContainer,
                                  fontWeight: FontWeight.w600,
                                ),
                              ),
                            ),
                            const SizedBox(height: 8),
                          ],
                          Text(
                            entry.question,
                            style: theme.textTheme.titleMedium?.copyWith(
                              fontWeight: FontWeight.w600,
                            ),
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(width: 12),
                    Icon(
                      isExpanded ? Icons.expand_less : Icons.expand_more,
                      color: colors.onSurfaceVariant,
                    ),
                  ],
                ),
              ),
            ),
          ),
          if (isExpanded) ...[
            const Divider(height: 1),
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 14, 16, 16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    entry.answer,
                    style: theme.textTheme.bodyMedium?.copyWith(
                      height: 1.5,
                      color: colors.onSurface,
                    ),
                  ),
                  const SizedBox(height: 14),
                  Wrap(
                    spacing: 12,
                    runSpacing: 6,
                    crossAxisAlignment: WrapCrossAlignment.center,
                    children: [
                      if (category != null && category.isNotEmpty)
                        Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(
                              Icons.label_outline,
                              size: 16,
                              color: colors.onSurfaceVariant,
                            ),
                            const SizedBox(width: 4),
                            Text(
                              category,
                              style: theme.textTheme.bodySmall?.copyWith(
                                color: colors.onSurfaceVariant,
                              ),
                            ),
                          ],
                        ),
                      Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Icon(
                            Icons.schedule,
                            size: 16,
                            color: colors.onSurfaceVariant,
                          ),
                          const SizedBox(width: 4),
                          Text(
                            'Atualizado em ${_formatUpdatedAt(context, entry.updatedAt)}',
                            style: theme.textTheme.bodySmall?.copyWith(
                              color: colors.onSurfaceVariant,
                            ),
                          ),
                        ],
                      ),
                    ],
                  ),
                  if (canManage) ...[
                    const SizedBox(height: 12),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.end,
                      children: [
                        TextButton.icon(
                          onPressed: isSubmitting ? null : onEdit,
                          icon: const Icon(Icons.edit_outlined, size: 18),
                          label: const Text('Editar'),
                        ),
                        const SizedBox(width: 8),
                        TextButton.icon(
                          onPressed: isSubmitting ? null : onDelete,
                          style: TextButton.styleFrom(
                            foregroundColor: colors.error,
                          ),
                          icon: const Icon(Icons.delete_outline, size: 18),
                          label: const Text('Excluir'),
                        ),
                      ],
                    ),
                  ],
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }
}
