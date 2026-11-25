/// Tela de detalhes de um evento publicado do Centro Acadêmico ativo.
///
/// Exibe título, informações do CA ativo, período (início e término), local,
/// situação de vagas/inscrição, descrição completa e a barra de ação para
/// inscrição e cancelamento de inscrição (#96 e #99).
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
    this.referenceTime,
  });

  /// Controlador da agenda de eventos com o contexto do CA ativo.
  final EventsScheduleController controller;

  /// Identificador do evento cujos detalhes devem ser consultados.
  final UuidValue eventId;

  /// Evento pré-carregado opcional enquanto a consulta de atualização ocorre.
  final PublishedEvent? initialEvent;

  /// Construtor opcional para substituir a barra inferior padrão de inscrição/cancelamento.
  final Widget Function(BuildContext context, PublishedEvent event)?
  bottomActionBuilder;

  /// Instante de referência opcional para avaliação do prazo de inscrição (útil em testes).
  final DateTime? referenceTime;

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

  PublishedEvent? get _effectiveEvent =>
      widget.controller.findCachedEvent(widget.eventId) ?? _event;

  Future<void> _handleRegister(PublishedEvent event) async {
    try {
      final record = await widget.controller.registerInEvent(
        event.id,
        currentEvent: event,
      );
      if (!mounted) return;
      setState(() {
        final cached = widget.controller.findCachedEvent(event.id);
        if (cached != null) {
          _event = cached;
        } else {
          final nextConfirmed = event.confirmedRegistrations + 1;
          final cap = event.maxCapacity;
          final nextRemaining = cap == null
              ? null
              : (cap - nextConfirmed).clamp(0, cap);
          _event = event.copyWith(
            confirmedRegistrations: nextConfirmed,
            remainingSpots: nextRemaining,
            hasAvailableSpots: cap == null || (nextRemaining ?? 0) > 0,
          );
        }
      });
      _showFeedbackSnackBar(
        record.message.isNotEmpty
            ? record.message
            : 'Inscrição realizada com sucesso.',
      );
    } on EventQueryException catch (failure) {
      if (!mounted) return;
      setState(() {
        final cached = widget.controller.findCachedEvent(event.id);
        if (cached != null) {
          _event = cached;
        } else if (failure.kind == EventQueryFailureKind.eventFull &&
            event.maxCapacity != null) {
          _event = event.copyWith(
            confirmedRegistrations: event.maxCapacity,
            remainingSpots: 0,
            hasAvailableSpots: false,
          );
        }
      });
      _showFeedbackSnackBar(failure.message, isError: true);
    }
  }

  Future<void> _confirmAndCancelRegistration(PublishedEvent event) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Cancelar inscrição?'),
        content: Text(
          'Deseja realmente cancelar sua inscrição em "${event.title}"?',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(false),
            child: const Text('Manter inscrição'),
          ),
          FilledButton(
            style: FilledButton.styleFrom(
              backgroundColor: Theme.of(dialogContext).colorScheme.error,
              foregroundColor: Theme.of(dialogContext).colorScheme.onError,
            ),
            onPressed: () => Navigator.of(dialogContext).pop(true),
            child: const Text('Confirmar cancelamento'),
          ),
        ],
      ),
    );

    if (confirmed != true || !mounted) {
      return;
    }

    try {
      final record = await widget.controller.cancelRegistration(
        event.id,
        currentEvent: event,
      );
      if (!mounted) return;
      setState(() {
        final cached = widget.controller.findCachedEvent(event.id);
        if (cached != null) {
          _event = cached;
        } else {
          final nextConfirmed = (event.confirmedRegistrations - 1).clamp(
            0,
            1 << 30,
          );
          final cap = event.maxCapacity;
          final nextRemaining = cap == null
              ? null
              : (cap - nextConfirmed).clamp(0, cap);
          _event = event.copyWith(
            confirmedRegistrations: nextConfirmed,
            remainingSpots: nextRemaining,
            hasAvailableSpots: cap == null || (nextRemaining ?? 0) > 0,
          );
        }
      });
      _showFeedbackSnackBar(
        record.message.isNotEmpty
            ? record.message
            : 'Inscrição cancelada com sucesso.',
      );
    } on EventQueryException catch (failure) {
      if (!mounted) return;
      _showFeedbackSnackBar(failure.message, isError: true);
    }
  }

  void _showFeedbackSnackBar(String message, {bool isError = false}) {
    final messenger = ScaffoldMessenger.maybeOf(context);
    if (messenger == null) return;
    final colors = Theme.of(context).colorScheme;
    messenger
      ..hideCurrentSnackBar()
      ..showSnackBar(
        SnackBar(
          content: Text(
            message,
            style: isError ? TextStyle(color: colors.onErrorContainer) : null,
          ),
          backgroundColor: isError ? colors.errorContainer : null,
        ),
      );
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
    final remaining = event.effectiveRemainingSpots ?? 0;
    if (event.isFull) {
      return 'Lotado ($maxCapacity/$maxCapacity vagas preenchidas)';
    }
    return '$remaining de $maxCapacity vagas disponíveis';
  }

  Widget? _buildDefaultBottomAction(
    BuildContext context,
    PublishedEvent event,
  ) {
    if (!event.requiresRegistration) {
      return null;
    }

    final theme = Theme.of(context);
    final colors = theme.colorScheme;
    final controller = widget.controller;
    final isRegistered = controller.isRegisteredIn(event.id);
    final isSubmitting = controller.isSubmittingRegistration;

    if (isRegistered) {
      return Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
            decoration: BoxDecoration(
              color: colors.primaryContainer,
              borderRadius: BorderRadius.circular(12),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(
                  Icons.check_circle_outline,
                  size: 18,
                  color: colors.onPrimaryContainer,
                ),
                const SizedBox(width: 8),
                Text(
                  'Inscrição confirmada',
                  style: theme.textTheme.labelLarge?.copyWith(
                    color: colors.onPrimaryContainer,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 10),
          OutlinedButton.icon(
            key: const ValueKey('cancel-event-registration-button'),
            style: OutlinedButton.styleFrom(
              foregroundColor: colors.error,
              side: BorderSide(color: colors.error),
            ),
            onPressed: isSubmitting
                ? null
                : () => _confirmAndCancelRegistration(event),
            icon: isSubmitting
                ? const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Icon(Icons.cancel_outlined),
            label: const Text('Cancelar inscrição'),
          ),
        ],
      );
    }

    if (event.isRegistrationDeadlinePassed(
      referenceTime: widget.referenceTime,
    )) {
      return const FilledButton(
        key: ValueKey('registration-closed-button'),
        onPressed: null,
        child: Text('Inscrições encerradas'),
      );
    }

    if (event.isFull) {
      return const FilledButton(
        key: ValueKey('event-full-button'),
        onPressed: null,
        child: Text('Vagas esgotadas'),
      );
    }

    return FilledButton.icon(
      key: const ValueKey('register-event-button'),
      onPressed: isSubmitting ? null : () => _handleRegister(event),
      icon: isSubmitting
          ? const SizedBox(
              width: 16,
              height: 16,
              child: CircularProgressIndicator(strokeWidth: 2),
            )
          : const Icon(Icons.how_to_reg_outlined),
      label: const Text('Inscrever-se'),
    );
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        final theme = Theme.of(context);
        final colors = theme.colorScheme;
        final activeCa = widget.controller.activeAcademicCenter;
        final event = _effectiveEvent;

        final bottomAction = event == null
            ? null
            : (widget.bottomActionBuilder != null
                  ? widget.bottomActionBuilder!(context, event)
                  : _buildDefaultBottomAction(context, event));

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
                                  crossAxisAlignment:
                                      CrossAxisAlignment.stretch,
                                  children: [
                                    _buildInfoTile(
                                      context: context,
                                      icon: Icons.calendar_today_outlined,
                                      label: 'Início',
                                      value: formatEventDateTime(
                                        event.startsAt,
                                      ),
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
          bottomNavigationBar: bottomAction != null
              ? SafeArea(
                  child: Padding(
                    padding: const EdgeInsets.fromLTRB(24, 12, 24, 16),
                    child: Center(
                      heightFactor: 1,
                      child: ConstrainedBox(
                        constraints: const BoxConstraints(maxWidth: 640),
                        child: bottomAction,
                      ),
                    ),
                  ),
                )
              : null,
        );
      },
    );
  }
}
