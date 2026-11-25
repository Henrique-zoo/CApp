/// Tela de listagem da agenda pública de eventos publicados por Centro Acadêmico ativo.
///
/// Apresenta o contexto do CA ativo (com seletor quando múltiplos CAs estiverem
/// disponíveis), estados de carregamento, erro com botão de nova tentativa,
/// lista vazia e cartões resumidos que navegam para [EventDetailPage].
library;

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../presentation/presentation.dart';
import '../events.dart';
import 'event_detail_page.dart';

/// Página principal da agenda de eventos publicados do Centro Acadêmico ativo.
class EventsSchedulePage extends StatefulWidget {
  const EventsSchedulePage({
    super.key,
    required this.controller,
    this.autoLoadOnInit = true,
  });

  /// Controlador que coordena o CA ativo e a busca dos eventos publicados.
  final EventsScheduleController controller;

  /// Inicia automaticamente a consulta dos eventos ao montar a página.
  final bool autoLoadOnInit;

  @override
  State<EventsSchedulePage> createState() => _EventsSchedulePageState();
}

class _EventsSchedulePageState extends State<EventsSchedulePage> {
  @override
  void initState() {
    super.initState();
    if (widget.autoLoadOnInit) {
      unawaited(widget.controller.loadEvents());
    }
  }

  void _openEventDetail(PublishedEvent event) {
    Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => EventDetailPage(
          controller: widget.controller,
          eventId: event.id,
          initialEvent: event,
        ),
      ),
    );
  }

  Widget _buildActiveCaHeader(BuildContext context) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;
    final centers = widget.controller.academicCenters;
    final activeCa = widget.controller.activeAcademicCenter;

    if (activeCa == null) {
      return const SizedBox.shrink();
    }

    return Padding(
      padding: const EdgeInsets.fromLTRB(24, 20, 24, 8),
      child: Row(
        children: [
          Icon(Icons.account_balance_outlined, color: colors.primary, size: 20),
          const SizedBox(width: 10),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Centro Acadêmico ativo',
                  style: theme.textTheme.labelSmall?.copyWith(
                    color: colors.onSurfaceVariant,
                  ),
                ),
                const SizedBox(height: 2),
                if (centers.length > 1)
                  DropdownButtonHideUnderline(
                    child: DropdownButton<AcademicCenterContext>(
                      key: const ValueKey('active-ca-selector'),
                      isDense: true,
                      value: activeCa,
                      items: centers
                          .map(
                            (ca) => DropdownMenuItem<AcademicCenterContext>(
                              value: ca,
                              child: Text(
                                ca.displayLabel,
                                style: theme.textTheme.titleMedium?.copyWith(
                                  fontWeight: FontWeight.w600,
                                ),
                              ),
                            ),
                          )
                          .toList(growable: false),
                      onChanged: widget.controller.isLoading
                          ? null
                          : (selected) {
                              if (selected != null && selected != activeCa) {
                                unawaited(
                                  widget.controller.selectAcademicCenter(
                                    selected,
                                  ),
                                );
                              }
                            },
                    ),
                  )
                else
                  Text(
                    activeCa.displayLabel,
                    style: theme.textTheme.titleMedium?.copyWith(
                      fontWeight: FontWeight.w600,
                    ),
                  ),
              ],
            ),
          ),
          IconButton(
            tooltip: 'Atualizar eventos',
            onPressed: widget.controller.isLoading
                ? null
                : () => unawaited(widget.controller.loadEvents()),
            icon: const Icon(Icons.refresh),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        final controller = widget.controller;
        final events = controller.events;
        final error = controller.error;

        return SafeArea(
          child: Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 640),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  _buildActiveCaHeader(context),
                  Expanded(
                    child: controller.isLoading
                        ? const Center(
                            child: CircularProgressIndicator(
                              semanticsLabel: 'Carregando agenda de eventos',
                            ),
                          )
                        : error != null
                        ? Center(
                            child: Padding(
                              padding: const EdgeInsets.all(24),
                              child: Column(
                                mainAxisSize: MainAxisSize.min,
                                children: [
                                  WarningBox(message: error.message),
                                  const SizedBox(height: 16),
                                  FilledButton.icon(
                                    onPressed: () =>
                                        unawaited(controller.loadEvents()),
                                    icon: const Icon(Icons.refresh),
                                    label: const Text('Tentar novamente'),
                                  ),
                                ],
                              ),
                            ),
                          )
                        : events.isEmpty
                        ? Center(
                            child: Padding(
                              padding: const EdgeInsets.all(24),
                              child: Column(
                                mainAxisSize: MainAxisSize.min,
                                children: [
                                  Icon(
                                    Icons.event_busy_outlined,
                                    size: 48,
                                    color: Theme.of(context)
                                        .colorScheme
                                        .onSurfaceVariant,
                                  ),
                                  const SizedBox(height: 16),
                                  Text(
                                    'Não há eventos publicados.',
                                    textAlign: TextAlign.center,
                                    style: Theme.of(context)
                                        .textTheme
                                        .titleMedium
                                        ?.copyWith(fontWeight: FontWeight.w600),
                                  ),
                                  const SizedBox(height: 8),
                                  Text(
                                    'Quando o Centro Acadêmico publicar novas atividades, elas aparecerão nesta agenda.',
                                    textAlign: TextAlign.center,
                                    style: Theme.of(context)
                                        .textTheme
                                        .bodyMedium
                                        ?.copyWith(
                                          color: Theme.of(context)
                                              .colorScheme
                                              .onSurfaceVariant,
                                        ),
                                  ),
                                ],
                              ),
                            ),
                          )
                        : RefreshIndicator(
                            onRefresh: controller.loadEvents,
                            child: ListView.separated(
                              padding: const EdgeInsets.fromLTRB(
                                24,
                                12,
                                24,
                                24,
                              ),
                              itemCount: events.length,
                              separatorBuilder: (_, _) =>
                                  const SizedBox(height: 12),
                              itemBuilder: (context, index) {
                                final event = events[index];
                                return EventSummaryCard(
                                  key: ValueKey('event-card-${event.id.uuid}'),
                                  event: event,
                                  isRegistered: controller.isRegisteredIn(
                                    event.id,
                                  ),
                                  onTap: () => _openEventDetail(event),
                                );
                              },
                            ),
                          ),
                  ),
                ],
              ),
            ),
          ),
        );
      },
    );
  }
}

/// Cartão de resumo de um evento publicado na listagem da agenda do CA.
class EventSummaryCard extends StatelessWidget {
  const EventSummaryCard({
    super.key,
    required this.event,
    required this.onTap,
    this.isRegistered = false,
  });

  final PublishedEvent event;
  final VoidCallback onTap;
  final bool isRegistered;

  String? _buildSpotsLabel() {
    if (!event.requiresRegistration) {
      return null;
    }
    final maxCapacity = event.maxCapacity;
    if (maxCapacity == null) {
      return 'Vagas ilimitadas';
    }
    if (event.isFull) {
      return 'Lotado';
    }
    final remaining = event.effectiveRemainingSpots ?? maxCapacity;
    return '$remaining de $maxCapacity vagas disponíveis';
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;
    final spotsLabel = _buildSpotsLabel();

    return Card(
      color: colors.surfaceContainer,
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: onTap,
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Expanded(
                    child: Text(
                      event.title,
                      style: theme.textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
                  Icon(Icons.chevron_right, color: colors.onSurfaceVariant),
                ],
              ),
              const SizedBox(height: 12),
              Row(
                children: [
                  Icon(
                    Icons.calendar_today_outlined,
                    size: 16,
                    color: colors.primary,
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      formatEventDateTime(event.startsAt),
                      style: theme.textTheme.bodyMedium?.copyWith(
                        color: colors.onSurfaceVariant,
                      ),
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 6),
              Row(
                children: [
                  Icon(Icons.place_outlined, size: 16, color: colors.primary),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      event.location,
                      style: theme.textTheme.bodyMedium?.copyWith(
                        color: colors.onSurfaceVariant,
                      ),
                    ),
                  ),
                ],
              ),
              if (spotsLabel != null || isRegistered) ...[
                const SizedBox(height: 10),
                Wrap(
                  spacing: 12,
                  runSpacing: 6,
                  crossAxisAlignment: WrapCrossAlignment.center,
                  children: [
                    if (spotsLabel != null)
                      Text(
                        spotsLabel,
                        style: theme.textTheme.labelMedium?.copyWith(
                          color: event.isFull ? colors.error : colors.primary,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    if (isRegistered)
                      Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Icon(
                            Icons.check_circle_outline,
                            size: 14,
                            color: colors.primary,
                          ),
                          const SizedBox(width: 4),
                          Text(
                            'Inscrito',
                            style: theme.textTheme.labelMedium?.copyWith(
                              color: colors.primary,
                              fontWeight: FontWeight.w600,
                            ),
                          ),
                        ],
                      ),
                  ],
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
