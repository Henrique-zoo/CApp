/// Diálogo modal de cadastro, edição e confirmação de remoção de FAQ.
library;

import 'package:flutter/material.dart';

import '../../../presentation/presentation.dart';
import '../domain/entities/faq_entry.dart';

/// Abre o formulário modal para cadastrar ou editar uma pergunta frequente.
///
/// Retorna `true` quando a operação foi confirmada e concluída com sucesso.
Future<bool?> showFaqEntryFormDialog(
  BuildContext context, {
  FaqEntry? initialEntry,
  List<String> existingCategories = const <String>[],
  required Future<bool> Function(FaqEntryDraft draft) onSubmit,
  String? Function()? readErrorMessage,
}) => showDialog<bool>(
  context: context,
  barrierDismissible: false,
  builder: (_) => _FaqEntryFormDialog(
    initialEntry: initialEntry,
    existingCategories: existingCategories,
    onSubmit: onSubmit,
    readErrorMessage: readErrorMessage,
  ),
);

/// Solicita confirmação explícita antes de remover uma pergunta frequente.
Future<bool?> confirmFaqEntryDeletion(
  BuildContext context, {
  required FaqEntry entry,
}) => showDialog<bool>(
  context: context,
  builder: (dialogContext) => AlertDialog(
    title: const Text('Remover pergunta frequente?'),
    content: SingleChildScrollView(
      child: Text(
        'A pergunta "${entry.question}" será removida do FAQ deste Centro Acadêmico.',
      ),
    ),
    actions: [
      TextButton(
        onPressed: () => Navigator.pop(dialogContext, false),
        child: const Text('Cancelar'),
      ),
      FilledButton(
        style: FilledButton.styleFrom(
          backgroundColor: const Color(0xFFDC2626),
          foregroundColor: Colors.white,
        ),
        onPressed: () => Navigator.pop(dialogContext, true),
        child: const Text('Remover'),
      ),
    ],
  ),
);

class _FaqEntryFormDialog extends StatefulWidget {
  const _FaqEntryFormDialog({
    required this.initialEntry,
    required this.existingCategories,
    required this.onSubmit,
    required this.readErrorMessage,
  });

  final FaqEntry? initialEntry;
  final List<String> existingCategories;
  final Future<bool> Function(FaqEntryDraft draft) onSubmit;
  final String? Function()? readErrorMessage;

  @override
  State<_FaqEntryFormDialog> createState() => _FaqEntryFormDialogState();
}

class _FaqEntryFormDialogState extends State<_FaqEntryFormDialog> {
  final _formKey = GlobalKey<FormState>();
  late final TextEditingController _questionController;
  late final TextEditingController _answerController;
  late final TextEditingController _categoryController;
  bool _submitting = false;
  String? _errorMessage;

  bool get _isEditing => widget.initialEntry != null;

  @override
  void initState() {
    super.initState();
    _questionController = TextEditingController(
      text: widget.initialEntry?.question ?? '',
    );
    _answerController = TextEditingController(
      text: widget.initialEntry?.answer ?? '',
    );
    _categoryController = TextEditingController(
      text: widget.initialEntry?.category ?? '',
    );
  }

  @override
  void dispose() {
    _questionController.dispose();
    _answerController.dispose();
    _categoryController.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    if (_submitting || !_formKey.currentState!.validate()) return;

    setState(() {
      _submitting = true;
      _errorMessage = null;
    });

    try {
      final draft = FaqEntryDraft(
        question: _questionController.text,
        answer: _answerController.text,
        category: _categoryController.text,
      );
      final ok = await widget.onSubmit(draft);
      if (!mounted) return;
      if (ok) {
        Navigator.pop(context, true);
      } else {
        setState(() {
          _submitting = false;
          _errorMessage =
              widget.readErrorMessage?.call() ??
              'Não foi possível salvar a pergunta frequente. Tente novamente.';
        });
      }
    } on FormatException catch (error) {
      if (!mounted) return;
      setState(() {
        _submitting = false;
        _errorMessage = error.message;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;

    return AlertDialog(
      title: Text(
        _isEditing ? 'Editar pergunta frequente' : 'Nova pergunta frequente',
      ),
      content: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 480),
        child: SingleChildScrollView(
          child: Form(
            key: _formKey,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                if (_errorMessage case final error?) ...[
                  WarningBox(message: error),
                  const SizedBox(height: 12),
                ],
                TextFormField(
                  controller: _questionController,
                  enabled: !_submitting,
                  maxLength: faqMaxQuestionLength,
                  maxLines: 2,
                  decoration: const InputDecoration(
                    labelText: 'Pergunta',
                    hintText: 'Ex.: Como solicito a carteirinha do CA?',
                    border: OutlineInputBorder(),
                  ),
                  validator: validateFaqQuestion,
                ),
                const SizedBox(height: 12),
                TextFormField(
                  controller: _answerController,
                  enabled: !_submitting,
                  maxLength: faqMaxAnswerLength,
                  minLines: 3,
                  maxLines: 6,
                  decoration: const InputDecoration(
                    labelText: 'Resposta',
                    hintText: 'Explique a orientação oficial do CA.',
                    border: OutlineInputBorder(),
                    alignLabelWithHint: true,
                  ),
                  validator: validateFaqAnswer,
                ),
                const SizedBox(height: 12),
                TextFormField(
                  controller: _categoryController,
                  enabled: !_submitting,
                  maxLength: faqMaxCategoryLength,
                  decoration: const InputDecoration(
                    labelText: 'Categoria (opcional)',
                    hintText: 'Ex.: Carteirinhas, Apoio Acadêmico',
                    border: OutlineInputBorder(),
                  ),
                  validator: validateFaqCategory,
                ),
                if (widget.existingCategories.isNotEmpty) ...[
                  const SizedBox(height: 6),
                  Text(
                    'Categorias existentes',
                    style: theme.textTheme.labelSmall?.copyWith(
                      color: colors.onSurfaceVariant,
                    ),
                  ),
                  const SizedBox(height: 6),
                  Wrap(
                    spacing: 8,
                    runSpacing: 6,
                    children: [
                      for (final category in widget.existingCategories)
                        ActionChip(
                          label: Text(category),
                          onPressed: _submitting
                              ? null
                              : () => _categoryController.text = category,
                        ),
                    ],
                  ),
                ],
              ],
            ),
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: _submitting ? null : () => Navigator.pop(context, false),
          child: const Text('Cancelar'),
        ),
        FilledButton(
          onPressed: _submitting ? null : _submit,
          child: Text(_submitting ? 'Salvando...' : 'Salvar'),
        ),
      ],
    );
  }
}
