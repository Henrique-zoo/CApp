/// Modal de atualização de status de demanda pela gestão do Centro Acadêmico.
library;

import 'package:flutter/material.dart';

import '../../../presentation/presentation.dart';
import '../application/demands_controller.dart';
import '../application/error/demand_exception.dart';
import '../application/models/create_demand_input.dart';
import '../application/models/demand.dart';
import '../application/models/demand_status.dart';
import 'demand_failure_message.dart';
import 'widgets/demand_badges.dart';

/// Abre o modal para a gestão atualizar o status da demanda com parecer.
Future<Demand?> showUpdateDemandStatusSheet(
  BuildContext context, {
  required DemandsController controller,
  required Demand demand,
}) => showModalBottomSheet<Demand>(
  context: context,
  isScrollControlled: true,
  useSafeArea: true,
  showDragHandle: true,
  builder: (_) =>
      UpdateDemandStatusSheet(controller: controller, demand: demand),
);

/// Formulário de transição de status restrito às regras do fórum da demanda.
class UpdateDemandStatusSheet extends StatefulWidget {
  const UpdateDemandStatusSheet({
    super.key,
    required this.controller,
    required this.demand,
  });

  final DemandsController controller;
  final Demand demand;

  @override
  State<UpdateDemandStatusSheet> createState() =>
      _UpdateDemandStatusSheetState();
}

class _UpdateDemandStatusSheetState extends State<UpdateDemandStatusSheet> {
  final _formKey = GlobalKey<FormState>();
  final _opinionController = TextEditingController();
  late final List<DemandStatus> _allowedStatuses;
  late DemandStatus _selectedStatus;
  bool _submitting = false;
  String? _errorMessage;

  @override
  void initState() {
    super.initState();
    _allowedStatuses = DemandStatus.allowedTransitions(
      forum: widget.demand.forum,
      from: widget.demand.status,
    );
    _selectedStatus = _allowedStatuses.first;
  }

  @override
  void dispose() {
    _opinionController.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    if (_submitting || !_formKey.currentState!.validate()) return;

    setState(() {
      _submitting = true;
      _errorMessage = null;
    });

    try {
      final updated = await widget.controller.updateDemandStatus(
        demandId: widget.demand.id,
        status: _selectedStatus,
        opinion: _opinionController.text,
      );
      if (!mounted) return;
      Navigator.of(context).pop(updated);
    } on FormatException catch (error) {
      if (mounted) setState(() => _errorMessage = error.message);
    } on DemandException catch (error) {
      if (mounted) setState(() => _errorMessage = demandFailureMessage(error));
    } finally {
      if (mounted) setState(() => _submitting = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final bottomInset = MediaQuery.viewInsetsOf(context).bottom;

    return Padding(
      padding: EdgeInsets.fromLTRB(20, 8, 20, bottomInset + 20),
      child: SingleChildScrollView(
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 560),
            child: Form(
              key: _formKey,
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Text(
                    'Atualizar status',
                    style: theme.textTheme.headlineSmall?.copyWith(
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                  const SizedBox(height: 10),
                  Row(
                    children: [
                      Text(
                        'Status atual:',
                        style: theme.textTheme.bodyMedium?.copyWith(
                          color: theme.colorScheme.onSurfaceVariant,
                        ),
                      ),
                      const SizedBox(width: 8),
                      DemandStatusBadge(status: widget.demand.status),
                    ],
                  ),
                  const SizedBox(height: 16),
                  Text('Novo status', style: theme.textTheme.labelLarge),
                  const SizedBox(height: 8),
                  Wrap(
                    spacing: 8,
                    runSpacing: 8,
                    children: [
                      for (final status in _allowedStatuses)
                        ChoiceChip(
                          label: Text(status.label),
                          selected: _selectedStatus == status,
                          onSelected: _submitting
                              ? null
                              : (_) => setState(() => _selectedStatus = status),
                        ),
                    ],
                  ),
                  const SizedBox(height: 16),
                  TextFormField(
                    controller: _opinionController,
                    enabled: !_submitting,
                    maxLength: maxDemandOpinionLength,
                    minLines: 3,
                    maxLines: 5,
                    decoration: InputDecoration(
                      labelText: 'Parecer / justificativa',
                      hintText: 'Explique o andamento ou decisão tomada',
                      alignLabelWithHint: true,
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(12),
                      ),
                    ),
                    validator: (value) {
                      if (value == null || value.trim().isEmpty) {
                        return 'Informe o parecer ou justificativa '
                            'da alteração.';
                      }
                      return null;
                    },
                  ),
                  if (_errorMessage case final error?) ...[
                    const SizedBox(height: 12),
                    Semantics(
                      liveRegion: true,
                      child: WarningBox(message: error),
                    ),
                  ],
                  const SizedBox(height: 16),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      TextButton(
                        onPressed: _submitting
                            ? null
                            : () => Navigator.of(context).pop(),
                        child: const Text('Cancelar'),
                      ),
                      const SizedBox(width: 12),
                      FilledButton(
                        onPressed: _submitting ? null : _submit,
                        child: _submitting
                            ? const AnimatedEllipsisText('Salvando')
                            : const Text('Confirmar alteração'),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
