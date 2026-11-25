/// Badges semânticos de status e fórum alinhados ao tema escuro do CApp.
library;

import 'package:flutter/material.dart';

import '../../application/models/demand_forum.dart';
import '../../application/models/demand_status.dart';

/// Selo semântico indicando o status atual da demanda.
class DemandStatusBadge extends StatelessWidget {
  const DemandStatusBadge({super.key, required this.status});

  final DemandStatus status;

  @override
  Widget build(BuildContext context) {
    final (background, foreground, icon) = switch (status) {
      DemandStatus.open => (
        const Color(0xFF0E2A47),
        const Color(0xFF7DD3FC),
        Icons.radio_button_checked,
      ),
      DemandStatus.inAnalysis => (
        const Color(0xFF3A2807),
        const Color(0xFFFCD34D),
        Icons.manage_search_rounded,
      ),
      DemandStatus.forwardedToCoordination => (
        const Color(0xFF2D1947),
        const Color(0xFFD8B4FE),
        Icons.forward_to_inbox_rounded,
      ),
      DemandStatus.inExecution => (
        const Color(0xFF13264A),
        const Color(0xFF93C5FD),
        Icons.engineering_outlined,
      ),
      DemandStatus.resolved || DemandStatus.completed => (
        const Color(0xFF0D3321),
        const Color(0xFF86EFAC),
        Icons.check_circle_outline,
      ),
      DemandStatus.rejected || DemandStatus.cancelled => (
        const Color(0xFF3F1316),
        const Color(0xFFFCA5A5),
        Icons.cancel_outlined,
      ),
    };

    return Semantics(
      label: 'Status: ${status.label}',
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
        decoration: BoxDecoration(
          color: background,
          borderRadius: BorderRadius.circular(999),
          border: Border.all(color: foreground.withValues(alpha: 0.35)),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 14, color: foreground),
            const SizedBox(width: 5),
            Text(
              status.label,
              style: Theme.of(context).textTheme.labelSmall?.copyWith(
                color: foreground,
                fontWeight: FontWeight.w600,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Selo compacto identificando o fórum da demanda.
class DemandForumBadge extends StatelessWidget {
  const DemandForumBadge({super.key, required this.forum});

  final DemandForum forum;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    final isInstitutional = forum == DemandForum.institutional;
    final background = isInstitutional
        ? colors.primaryContainer.withValues(alpha: 0.55)
        : colors.secondaryContainer.withValues(alpha: 0.55);
    final foreground = isInstitutional
        ? colors.onPrimaryContainer
        : colors.onSecondaryContainer;
    final icon = isInstitutional
        ? Icons.school_outlined
        : Icons.meeting_room_outlined;

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
      decoration: BoxDecoration(
        color: background,
        borderRadius: BorderRadius.circular(999),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 14, color: foreground),
          const SizedBox(width: 5),
          Text(
            forum.shortLabel,
            style: Theme.of(context).textTheme.labelSmall?.copyWith(
              color: foreground,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }
}
