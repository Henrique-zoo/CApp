/// Tela principal de consulta, busca, filtragem e gestão rápida de FAQ.
library;

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../presentation/presentation.dart';
import '../application/faq_controller.dart';
import '../application/faq_state.dart';
import '../domain/entities/faq_entry.dart';
import 'faq_entry_card.dart';
import 'faq_entry_form_dialog.dart';

/// Apresenta o FAQ do Centro Acadêmico ativo com busca, categorias e expansão inline.
class FaqPage extends StatefulWidget {
  /// Constrói a tela observando [controller].
  const FaqPage({super.key, required this.controller});

  /// Controlador que coordena consultas, filtros e operações de gestão.
  final FaqController controller;

  @override
  State<FaqPage> createState() => _FaqPageState();
}

class _FaqPageState extends State<FaqPage> {
  late final TextEditingController _searchController;
  StreamSubscription<FaqState>? _stateSubscription;

  @override
  void initState() {
    super.initState();
    _searchController = TextEditingController(
      text: widget.controller.state.searchQuery,
    );
    _subscribeToController();
    _loadIfNeeded();
  }

  @override
  void didUpdateWidget(covariant FaqPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (!identical(oldWidget.controller, widget.controller)) {
      unawaited(_stateSubscription?.cancel());
      _searchController.text = widget.controller.state.searchQuery;
      _subscribeToController();
      _loadIfNeeded();
    }
  }

  void _subscribeToController() {
    _stateSubscription = widget.controller.states.listen((state) {
      if (!mounted) return;
      if (state.searchQuery.isEmpty && _searchController.text.isNotEmpty) {
        _searchController.clear();
      }
    });
  }

  void _loadIfNeeded() {
    final state = widget.controller.state;
    if (state.hasAcademicCenter && !state.hasLoadedOnce && !state.isLoading) {
      unawaited(widget.controller.load());
    }
  }

  @override
  void dispose() {
    unawaited(_stateSubscription?.cancel());
    _searchController.dispose();
    super.dispose();
  }

  void _clearSearch() {
    _searchController.clear();
    unawaited(widget.controller.clearSearch());
  }

  void _clearFilters() {
    _searchController.clear();
    unawaited(widget.controller.clearFilters());
  }

  Future<void> _openCreateForm(FaqState state) async {
    widget.controller.clearMutationError();
    final created = await showFaqEntryFormDialog(
      context,
      existingCategories: state.categories,
      onSubmit: widget.controller.createEntry,
      readErrorMessage: () => widget.controller.state.mutationErrorMessage,
    );
    if (created == true && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Pergunta frequente cadastrada com sucesso.'),
        ),
      );
    }
  }

  Future<void> _openEditForm(FaqState state, FaqEntry entry) async {
    widget.controller.clearMutationError();
    final updated = await showFaqEntryFormDialog(
      context,
      initialEntry: entry,
      existingCategories: state.categories,
      onSubmit: (draft) => widget.controller.updateEntry(entry.id, draft),
      readErrorMessage: () => widget.controller.state.mutationErrorMessage,
    );
    if (updated == true && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Pergunta frequente atualizada com sucesso.'),
        ),
      );
    }
  }

  Future<void> _confirmDelete(FaqEntry entry) async {
    final confirmed = await confirmFaqEntryDeletion(context, entry: entry);
    if (confirmed != true || !mounted) return;

    widget.controller.clearMutationError();
    final removed = await widget.controller.deleteEntry(entry.id);
    if (!mounted) return;

    if (removed) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Pergunta frequente removida.')),
      );
    } else if (widget.controller.state.mutationErrorMessage case final error?) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text(error)));
    }
  }

  @override
  Widget build(BuildContext context) {
    return StreamBuilder<FaqState>(
      key: ObjectKey(widget.controller),
      stream: widget.controller.states,
      initialData: widget.controller.state,
      builder: (context, _) {
        final state = widget.controller.state;

        return Scaffold(
          backgroundColor: Colors.transparent,
          primary: false,
          floatingActionButton: state.hasAcademicCenter && state.canManage
              ? FloatingActionButton.extended(
                  onPressed: state.isSubmitting
                      ? null
                      : () => _openCreateForm(state),
                  icon: const Icon(Icons.add),
                  label: const Text('Nova pergunta'),
                )
              : null,
          body: !state.hasAcademicCenter
              ? _buildNoAcademicCenter(context)
              : Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    _buildControls(context, state),
                    if (state.isLoading && state.entries.isNotEmpty)
                      const LinearProgressIndicator(minHeight: 2),
                    Expanded(child: _buildBody(context, state)),
                  ],
                ),
        );
      },
    );
  }

  Widget _buildNoAcademicCenter(BuildContext context) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;

    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 420),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                Icons.school_outlined,
                size: 48,
                color: colors.onSurfaceVariant,
              ),
              const SizedBox(height: 16),
              Text(
                'Nenhum Centro Acadêmico ativo',
                style: theme.textTheme.titleMedium?.copyWith(
                  fontWeight: FontWeight.w600,
                ),
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 8),
              Text(
                'Selecione um Centro Acadêmico para consultar as perguntas frequentes.',
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: colors.onSurfaceVariant,
                ),
                textAlign: TextAlign.center,
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildControls(BuildContext context, FaqState state) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;

    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (state.academicCenterName case final caName?) ...[
            Row(
              children: [
                Icon(
                  Icons.account_balance_outlined,
                  size: 16,
                  color: colors.primary,
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    caName,
                    style: theme.textTheme.labelLarge?.copyWith(
                      color: colors.primary,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 10),
          ],
          TextField(
            controller: _searchController,
            textInputAction: TextInputAction.search,
            onChanged: widget.controller.setSearchQuery,
            onSubmitted: widget.controller.searchImmediately,
            decoration: InputDecoration(
              hintText: 'Buscar por palavra-chave no FAQ',
              prefixIcon: const Icon(Icons.search),
              suffixIcon: state.searchQuery.isNotEmpty
                  ? IconButton(
                      onPressed: _clearSearch,
                      tooltip: 'Limpar busca',
                      icon: const Icon(Icons.close),
                    )
                  : null,
              filled: true,
              fillColor: colors.surfaceContainerHighest.withValues(alpha: 0.45),
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(14),
                borderSide: BorderSide(
                  color: colors.outlineVariant.withValues(alpha: 0.5),
                ),
              ),
              contentPadding: const EdgeInsets.symmetric(
                horizontal: 16,
                vertical: 12,
              ),
            ),
          ),
          const SizedBox(height: 12),
          SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: Row(
              children: [
                _buildCategoryChip(
                  context,
                  label: 'Todas',
                  selected: state.selectedCategory == null,
                  onSelected: (_) =>
                      unawaited(widget.controller.selectCategory(null)),
                ),
                for (final category in state.categories) ...[
                  const SizedBox(width: 8),
                  _buildCategoryChip(
                    context,
                    label: category,
                    selected: state.selectedCategory == category,
                    onSelected: (selected) => unawaited(
                      widget.controller.selectCategory(
                        selected ? category : null,
                      ),
                    ),
                  ),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCategoryChip(
    BuildContext context, {
    required String label,
    required bool selected,
    required ValueChanged<bool> onSelected,
  }) {
    final colors = Theme.of(context).colorScheme;
    return ChoiceChip(
      label: Text(label),
      selected: selected,
      onSelected: onSelected,
      selectedColor: colors.primaryContainer,
      labelStyle: TextStyle(
        color: selected ? colors.onPrimaryContainer : colors.onSurfaceVariant,
        fontWeight: selected ? FontWeight.w600 : FontWeight.w500,
      ),
      side: BorderSide(
        color: selected
            ? colors.primary
            : colors.outlineVariant.withValues(alpha: 0.6),
      ),
    );
  }

  Widget _buildBody(BuildContext context, FaqState state) {
    if (state.isLoading && !state.hasLoadedOnce && state.entries.isEmpty) {
      return const Center(
        child: Padding(
          padding: EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              CircularProgressIndicator(),
              SizedBox(height: 16),
              Text('Carregando perguntas frequentes…'),
            ],
          ),
        ),
      );
    }

    if (state.errorMessage != null && state.entries.isEmpty) {
      return Center(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 460),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                WarningBox(message: state.errorMessage!),
                const SizedBox(height: 16),
                FilledButton.icon(
                  onPressed: widget.controller.refresh,
                  icon: const Icon(Icons.refresh),
                  label: const Text('Tentar novamente'),
                ),
              ],
            ),
          ),
        ),
      );
    }

    if (state.entries.isEmpty) {
      return _buildEmptyState(context, state);
    }

    return RefreshIndicator(
      onRefresh: widget.controller.refresh,
      child: ListView.separated(
        physics: const AlwaysScrollableScrollPhysics(),
        padding: EdgeInsets.fromLTRB(16, 8, 16, state.canManage ? 88 : 24),
        itemCount:
            state.entries.length + (state.errorMessage != null ? 1 : 0),
        separatorBuilder: (_, _) => const SizedBox(height: 12),
        itemBuilder: (context, index) {
          if (state.errorMessage != null && index == 0) {
            return Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                WarningBox(message: state.errorMessage!),
                Align(
                  alignment: Alignment.centerRight,
                  child: TextButton(
                    onPressed: widget.controller.refresh,
                    child: const Text('Tentar novamente'),
                  ),
                ),
              ],
            );
          }

          final entryIndex = state.errorMessage != null ? index - 1 : index;
          final entry = state.entries[entryIndex];
          final isExpanded = state.expandedEntryIds.contains(entry.id);

          return FaqEntryCard(
            entry: entry,
            isExpanded: isExpanded,
            onToggle: () => widget.controller.toggleExpanded(entry.id),
            canManage: state.canManage,
            isSubmitting: state.isSubmitting,
            onEdit: state.canManage ? () => _openEditForm(state, entry) : null,
            onDelete: state.canManage ? () => _confirmDelete(entry) : null,
          );
        },
      ),
    );
  }

  Widget _buildEmptyState(BuildContext context, FaqState state) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;
    final hasFilters = state.hasActiveFilters;

    return RefreshIndicator(
      onRefresh: widget.controller.refresh,
      child: LayoutBuilder(
        builder: (context, constraints) => SingleChildScrollView(
          physics: const AlwaysScrollableScrollPhysics(),
          padding: const EdgeInsets.all(24),
          child: ConstrainedBox(
            constraints: BoxConstraints(
              minHeight: (constraints.maxHeight - 48).clamp(0, double.infinity),
            ),
            child: Center(
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 420),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(
                      hasFilters ? Icons.search_off_rounded : Icons.quiz_outlined,
                      size: 48,
                      color: colors.onSurfaceVariant,
                    ),
                    const SizedBox(height: 16),
                    Text(
                      hasFilters
                          ? 'Nenhuma pergunta encontrada'
                          : 'Nenhuma pergunta frequente disponível',
                      style: theme.textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.w600,
                      ),
                      textAlign: TextAlign.center,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      hasFilters
                          ? 'Não encontramos resultados para os filtros atuais neste Centro Acadêmico.'
                          : 'Este Centro Acadêmico ainda não possui perguntas cadastradas no FAQ.',
                      style: theme.textTheme.bodyMedium?.copyWith(
                        color: colors.onSurfaceVariant,
                      ),
                      textAlign: TextAlign.center,
                    ),
                    if (hasFilters) ...[
                      const SizedBox(height: 16),
                      OutlinedButton.icon(
                        onPressed: _clearFilters,
                        icon: const Icon(Icons.filter_alt_off),
                        label: const Text('Limpar filtros'),
                      ),
                    ] else if (state.canManage) ...[
                      const SizedBox(height: 16),
                      FilledButton.icon(
                        onPressed: state.isSubmitting
                            ? null
                            : () => _openCreateForm(state),
                        icon: const Icon(Icons.add),
                        label: const Text('Cadastrar primeira pergunta'),
                      ),
                    ],
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
