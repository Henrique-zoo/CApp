/// Formulário modal para criação de uma nova demanda nos fóruns do CA.
library;

import 'package:flutter/material.dart';

import '../../../presentation/presentation.dart';
import '../application/demands_controller.dart';
import '../application/error/demand_exception.dart';
import '../application/models/create_demand_input.dart';
import '../application/models/demand.dart';
import '../application/models/demand_forum.dart';
import 'demand_failure_message.dart';

/// Abre o modal de criação de demanda e retorna a demanda criada, se houver.
Future<Demand?> showCreateDemandSheet(
  BuildContext context, {
  required DemandsController controller,
  DemandForum? initialForum,
}) => showModalBottomSheet<Demand>(
  context: context,
  isScrollControlled: true,
  useSafeArea: true,
  showDragHandle: true,
  builder: (_) =>
      CreateDemandSheet(controller: controller, initialForum: initialForum),
);

/// Formulário de criação de demanda com seleção de fórum e validação.
class CreateDemandSheet extends StatefulWidget {
  const CreateDemandSheet({
    super.key,
    required this.controller,
    this.initialForum,
  });

  final DemandsController controller;
  final DemandForum? initialForum;

  @override
  State<CreateDemandSheet> createState() => _CreateDemandSheetState();
}

class _CreateDemandSheetState extends State<CreateDemandSheet> {
  final _formKey = GlobalKey<FormState>();
  final _titleController = TextEditingController();
  final _descriptionController = TextEditingController();
  final _categoryController = TextEditingController();
  late final TextEditingController _courseController;
  late DemandForum _forum;
  bool _submitting = false;
  String? _errorMessage;

  @override
  void initState() {
    super.initState();
    _forum =
        widget.initialForum ??
        (widget.controller.isInstitutionalForumForbidden
            ? DemandForum.physicalSpace
            : widget.controller.selectedForum);
    _courseController = TextEditingController(
      text: widget.controller.userCourseName ?? '',
    );
  }

  @override
  void dispose() {
    _titleController.dispose();
    _descriptionController.dispose();
    _categoryController.dispose();
    _courseController.dispose();
    super.dispose();
  }

  bool get _hasConfirmedCourse =>
      widget.controller.userCourseName?.trim().isNotEmpty ?? false;

  bool get _needsInstitutionalWarning =>
      _forum == DemandForum.institutional &&
      !widget.controller.isCaManager &&
      (!widget.controller.hasStudentMembership || !_hasConfirmedCourse);

  Future<void> _submit() async {
    if (_submitting || !_formKey.currentState!.validate()) return;

    setState(() {
      _submitting = true;
      _errorMessage = null;
    });

    try {
      final input = CreateDemandInput(
        forum: _forum,
        title: _titleController.text,
        description: _descriptionController.text,
        category: _categoryController.text,
        courseName: _forum == DemandForum.institutional
            ? _courseController.text
            : null,
      );
      final created = await widget.controller.createDemand(input);
      if (!mounted) return;
      Navigator.of(context).pop(created);
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
    final courseReadOnly =
        !widget.controller.isCaManager && _hasConfirmedCourse;

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
                    'Nova demanda',
                    style: theme.textTheme.headlineSmall?.copyWith(
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                  const SizedBox(height: 6),
                  Text(
                    'Escolha o fórum adequado e descreva sua necessidade.',
                    style: theme.textTheme.bodyMedium?.copyWith(
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                  ),
                  const SizedBox(height: 16),
                  SegmentedButton<DemandForum>(
                    segments: const [
                      ButtonSegment(
                        value: DemandForum.institutional,
                        icon: Icon(Icons.school_outlined),
                        label: Text('Institucional'),
                      ),
                      ButtonSegment(
                        value: DemandForum.physicalSpace,
                        icon: Icon(Icons.meeting_room_outlined),
                        label: Text('Espaço Físico'),
                      ),
                    ],
                    selected: {_forum},
                    onSelectionChanged: _submitting
                        ? null
                        : (selection) {
                            if (selection.isEmpty) return;
                            setState(() {
                              _forum = selection.first;
                              _errorMessage = null;
                            });
                          },
                  ),
                  const SizedBox(height: 16),
                  if (_needsInstitutionalWarning) ...[
                    const WarningBox(
                      message:
                          'Para registrar uma demanda no fórum institucional, '
                          'é necessário ter vínculo estudantil com curso '
                          'representado por este CA. Você ainda pode registrar '
                          'no fórum do Espaço Físico.',
                    ),
                    const SizedBox(height: 16),
                  ],
                  if (_forum == DemandForum.institutional) ...[
                    TextFormField(
                      controller: _courseController,
                      enabled: !_submitting,
                      readOnly: courseReadOnly,
                      maxLength: maxDemandCourseNameLength,
                      decoration: InputDecoration(
                        labelText: 'Curso',
                        border: OutlineInputBorder(
                          borderRadius: BorderRadius.circular(12),
                        ),
                        suffixIcon: courseReadOnly
                            ? const Icon(Icons.lock_outline)
                            : null,
                      ),
                      validator: (value) {
                        if (_forum != DemandForum.institutional) return null;
                        if (value == null || value.trim().isEmpty) {
                          return 'Informe o curso associado à demanda.';
                        }
                        return null;
                      },
                    ),
                    const SizedBox(height: 12),
                  ],
                  TextFormField(
                    controller: _titleController,
                    enabled: !_submitting,
                    maxLength: maxDemandTitleLength,
                    textInputAction: TextInputAction.next,
                    decoration: InputDecoration(
                      labelText: 'Título (opcional)',
                      hintText: 'Resumo curto da demanda',
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(12),
                      ),
                    ),
                  ),
                  const SizedBox(height: 12),
                  TextFormField(
                    controller: _categoryController,
                    enabled: !_submitting,
                    maxLength: maxDemandCategoryLength,
                    textInputAction: TextInputAction.next,
                    decoration: InputDecoration(
                      labelText: 'Categoria (opcional)',
                      hintText: 'Ex: Infraestrutura, Pedagógico',
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(12),
                      ),
                    ),
                  ),
                  const SizedBox(height: 12),
                  TextFormField(
                    controller: _descriptionController,
                    enabled: !_submitting,
                    maxLength: maxDemandDescriptionLength,
                    minLines: 4,
                    maxLines: 7,
                    decoration: InputDecoration(
                      labelText: 'Descrição',
                      hintText: 'Detalhe o problema ou necessidade encontrada',
                      alignLabelWithHint: true,
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(12),
                      ),
                    ),
                    validator: (value) {
                      if (value == null || value.trim().isEmpty) {
                        return 'Informe a descrição da demanda.';
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
                            ? const AnimatedEllipsisText('Enviando')
                            : const Text('Registrar demanda'),
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
