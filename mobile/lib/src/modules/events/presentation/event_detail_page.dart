/// Tela de detalhes de um evento publicado do Centro Acadêmico ativo.
///
/// Exibe título, informações do CA ativo, período (início e término), local,
/// situação de vagas/inscrição e descrição completa. Reserva uma barra inferior
/// limpa para a ação de inscrição/cancelamento das próximas issues (#96/#99).
library;

import 'package:flutter/material.dart';
import 'package:uuid/uuid.dart';

import '../../../presentation/presentation.dart';
import '../events.dart';

/// Formata data e hora em padrão legível (`dd/MM/yyyy às HH:mm`).
String formatEventDateTime(DateTime dateTime) {
  final local = dateTime.toUtc();
  final day = local.day.toString().padLeft(2, '0');
  final month = local.month.toString().padLeft(2, '0');
  final year = local.year.toString();
  final hour = local.hour.toString().padLeft(2, '0');
  final minute = local.minute.toString().padLeft(2, '0');
  return '$day/$month/$year às $hour:$minute';
}

/// Página que carrega e apresenta os detalhes de um evento publicado.
class EventDetailPage extends StatefulWidget {
  const EventDetailPage({
    super.key,
    required this.controller,
    required this.eventId,
    this.initialEvent,
    this.bottomActionBuilder,
  });

  /// Controlador da agenda de eventos com o contexto do CA ativo.
  final EventsScheduleController controller;

  /// Identificador do evento cujos detalhes devem ser consultados.
  final UuidValue eventId;

  /// Evento pré-carregado opcional enquanto a consulta de atualização ocorre.
  final PublishedEvent? initialEvent;

  /// Construtor opcional para área inferior de ação (ex.: inscrição/cancelamento).
  final Widget Function(BuildContext context, PublishedEvent event)?
  bottomActionBuilder;

  @override
  State<EventDetailPage> createState() => _EventDetailPageState();
}

class _EventDetailPageState extends State<EventDetailPage> {
  PublishedEvent? _event;
  EventQueryException? _error;
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _loadDetail();
  }

  Future<void> _loadDetail() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    try {
      final detail = await widget.controller.loadEventDetail(widget.eventId);
      if (!mounted) return;
      setState(() {
        _event = detail;
        _isLoading = false;
      });
    } on EventQueryException catch (failure) {
      if (!mounted) return;
      setState(() {
        _event = null;
        _error = failure;
        _isLoading = false;
      });
    } catch (cause) {
      if (!mounted) return;
      setState(() {
        _event = null;
        _error = EventQueryException(
          kind: EventQueryFailureKind.unavailable,
          message: 'Não foi possível carregar os detalhes do evento.',
          cause: cause,
        );
        _isLoading = false;
      });
    }
  }

  Widget _buildInfoTile({
    required BuildContext context,
    required IconData icon,
    required String label,
    required String value,
  }) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;

    return Padding(
      padding: const EdgeInsets.only(bottom: 16),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            padding: const EdgeInsets.all(10),
            decoration: BoxDecoration(
              color: colors.surfaceContainerHighest,
              borderRadius: BorderRadius.circular(12),
            ),
            child: Icon(icon, size: 20, color: colors.primary),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  label,
                  style: theme.textTheme.labelMedium?.copyWith(
                    color: colors.onSurfaceVariant,
                  ),
                ),
                const SizedBox(height: 4),
                Text(value, style: theme.textTheme.bodyLarge),
              ],
            ),
          ),
        ],
      ),
    );
  }

  String _formatSpotsInfo(PublishedEvent event) {
    if (!event.requiresRegistration) {
      return 'Entrada livre (não requer inscrição prévia)';
    }
    final maxCapacity = event.maxCapacity;
    if (maxCapacity == null) {
      return 'Inscrições abertas • Vagas ilimitadas (${event.confirmedRegistrations} inscritos)';
    }
    final remaining =
        event.remainingSpots ??
        (maxCapacity - event.confirmedRegistrations).clamp(0, maxCapacity);
    if (!event.hasAvailableSpots || remaining == 0) {
      return 'Evento lotado ($maxCapacity/$maxCapacity vagas preenchidas)';
    }
    return '$remaining de $maxCapacity vagas disponíveis';
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;
    final activeCa = widget.controller.activeAcademicCenter;
    final event = _event;

    return Scaffold(
      appBar: AppBar(title: const Text('Detalhes do evento')),
      body: SafeArea(
        child: _isLoading
            ? const Center(
                child: CircularProgressIndicator(
                  semanticsLabel: 'Carregando detalhes do evento',
                ),
              )
            : _error != null || event == null
            ? Center(
                child: Padding(
                  padding: const EdgeInsets.all(24),
                  child: ConstrainedBox(
                    constraints: const BoxConstraints(maxWidth: 480),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        WarningBox(
                          message:
                              _error?.kind == EventQueryFailureKind.notFound
                              ? 'Os detalhes deste evento não estão disponíveis.'
                              : (_error?.message ??
                                    'Não foi possível carregar os detalhes do evento.'),
                        ),
                        const SizedBox(height: 16),
                        if (_error?.kind != EventQueryFailureKind.notFound)
                          FilledButton.icon(
                            onPressed: _loadDetail,
                            icon: const Icon(Icons.refresh),
                            label: const Text('Tentar novamente'),
                          ),
                      ],
                    ),
                  ),
                ),
              )
            : SingleChildScrollView(
                padding: const EdgeInsets.all(24),
                child: Center(
                  child: ConstrainedBox(
                    constraints: const BoxConstraints(maxWidth: 640),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        if (activeCa != null) ...[
                          Align(
                            alignment: Alignment.centerLeft,
                            child: Chip(
                              avatar: Icon(
                                Icons.school_outlined,
                                size: 18,
                                color: colors.primary,
                              ),
                              label: Text(activeCa.displayLabel),
                            ),
                          ),
                          const SizedBox(height: 12),
                        ],
                        Text(
                          event.title,
                          style: theme.textTheme.headlineSmall?.copyWith(
                            fontWeight: FontWeight.w700,
                          ),
                        ),
                        const SizedBox(height: 24),
                        Card(
                          color: colors.surfaceContainer,
                          child: Padding(
                            padding: const EdgeInsets.all(20),
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.stretch,
                              children: [
                                _buildInfoTile(
                                  context: context,
                                  icon: Icons.calendar_today_outlined,
                                  label: 'Início',
                                  value: formatEventDateTime(event.startsAt),
                                ),
                                _buildInfoTile(
                                  context: context,
                                  icon: Icons.schedule_outlined,
                                  label: 'Término',
                                  value: formatEventDateTime(event.endsAt),
                                ),
                                _buildInfoTile(
                                  context: context,
                                  icon: Icons.place_outlined,
                                  label: 'Local',
                                  value: event.location,
                                ),
                                _buildInfoTile(
                                  context: context,
                                  icon: Icons.people_outline,
                                  label: 'Participação e vagas',
                                  value: _formatSpotsInfo(event),
                                ),
                                if (event.registrationDeadline != null)
                                  _buildInfoTile(
                                    context: context,
                                    icon: Icons.timer_outlined,
                                    label: 'Prazo de inscrição',
                                    value: formatEventDateTime(
                                      event.registrationDeadline!,
                                    ),
                                  ),
                              ],
                            ),
                          ),
                        ),
                        const SizedBox(height: 24),
                        Text(
                          'Sobre o evento',
                          style: theme.textTheme.titleMedium?.copyWith(
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                        const SizedBox(height: 12),
                        Card(
                          color: colors.surfaceContainerLow,
                          child: Padding(
                            padding: const EdgeInsets.all(20),
                            child: Text(
                              event.description,
                              style: theme.textTheme.bodyLarge?.copyWith(
                                height: 1.5,
                              ),
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
      ),
      bottomNavigationBar:
          event != null && widget.bottomActionBuilder != null
          ? SafeArea(
              child: Padding(
                padding: const EdgeInsets.fromLTRB(24, 12, 24, 16),
                child: widget.bottomActionBuilder!(context, event),
              ),
            )
          : null,
    );
  }
}
