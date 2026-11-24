/// Tela principal de consulta, filtragem e criação nos dois fóruns de demandas.
library;

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../presentation/presentation.dart';
import '../application/demands_controller.dart';
import '../application/models/demand.dart';
import '../application/models/demand_forum.dart';
import '../application/models/demand_status.dart';
import 'create_demand_sheet.dart';
import 'demand_detail_page.dart';
import 'demand_failure_message.dart';
import 'widgets/demand_card.dart';

/// Apresenta os fóruns Institucional e Espaço Físico com filtros e navegação.
class DemandsHome extends StatefulWidget {
  const DemandsHome({
    super.key,
    required this.controller,
    this.onChangeAcademicCenter,
  });

  final DemandsController controller;

  /// Callback opcional para selecionar outro Centro Acadêmico ativo.
  final VoidCallback? onChangeAcademicCenter;

  @override
  State<DemandsHome> createState() => _DemandsHomeState();
}

class _DemandsHomeState extends State<DemandsHome> {
  final _searchController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _searchController.text = widget.controller.searchQuery;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) unawaited(widget.controller.loadDemands());
    });
  }

  @override
  void didUpdateWidget(covariant DemandsHome oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (!identical(oldWidget.controller, widget.controller)) {
      _searchController.text = widget.controller.searchQuery;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) unawaited(widget.controller.loadDemands());
      });
    }
  }

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  Future<void> _openCreateSheet() async {
    final created = await showCreateDemandSheet(
      context,
      controller: widget.controller,
    );
    if (created != null && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Demanda registrada com sucesso.')),
      );
    }
  }

  Future<void> _openDetail(Demand demand) async {
    await Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => DemandDetailPage(
          controller: widget.controller,
          demandId: demand.id,
          initialDemand: demand,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final controller = widget.controller;
    final theme = Theme.of(context);

    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        final statusOptions = DemandStatus.filterOptionsFor(
          controller.selectedForum,
        );
        final visibleDemands = controller.visibleDemands;

        return Scaffold(
          backgroundColor: Colors.transparent,
          floatingActionButton:
              controller.hasStudentMembership || controller.isCaManager
              ? FloatingActionButton.extended(
                  onPressed: _openCreateSheet,
                  icon: const Icon(Icons.add),
                  label: const Text('Nova demanda'),
                )
              : null,
          body: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    if (controller.academicCenterName != null ||
                        widget.onChangeAcademicCenter != null) ...[
                      Row(
                        children: [
                          Icon(
                            Icons.account_balance_outlined,
                            size: 18,
                            color: theme.colorScheme.primary,
                          ),
                          const SizedBox(width: 8),
                          Expanded(
                            child: Text(
                              controller.academicCenterName ??
                                  'Centro Acadêmico ativo',
                              style: theme.textTheme.titleSmall?.copyWith(
                                fontWeight: FontWeight.w600,
                              ),
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                          if (widget.onChangeAcademicCenter case final change?)
                            TextButton(
                              onPressed: change,
                              child: const Text('Trocar CA'),
                            ),
                        ],
                      ),
                      const SizedBox(height: 10),
                    ],
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
                      selected: {controller.selectedForum},
                      onSelectionChanged: (selection) {
                        if (selection.isNotEmpty) {
                          unawaited(controller.selectForum(selection.first));
                        }
                      },
                    ),
                    const SizedBox(height: 12),
                    TextField(
                      controller: _searchController,
                      onChanged: controller.setSearchQuery,
                      decoration: InputDecoration(
                        hintText: 'Buscar por título, descrição ou categoria',
                        prefixIcon: const Icon(Icons.search),
                        suffixIcon: controller.searchQuery.isNotEmpty
                            ? IconButton(
                                tooltip: 'Limpar busca',
                                onPressed: () {
                                  _searchController.clear();
                                  controller.setSearchQuery('');
                                },
                                icon: const Icon(Icons.close),
                              )
                            : null,
                        isDense: true,
                        border: OutlineInputBorder(
                          borderRadius: BorderRadius.circular(12),
                        ),
                      ),
                    ),
                    const SizedBox(height: 10),
                    SingleChildScrollView(
                      scrollDirection: Axis.horizontal,
                      child: Row(
                        children: [
                          FilterChip(
                            avatar: const Icon(Icons.person_outline, size: 16),
                            label: const Text('Minhas demandas'),
                            selected: controller.onlyMine,
                            onSelected: (value) =>
                                unawaited(controller.toggleOnlyMine(value)),
                          ),
                          const SizedBox(width: 8),
                          ChoiceChip(
                            label: const Text('Todos'),
                            selected: controller.selectedStatus == null,
                            onSelected: (_) =>
                                unawaited(controller.selectStatus(null)),
                          ),
                          for (final status in statusOptions) ...[
                            const SizedBox(width: 8),
                            ChoiceChip(
                              label: Text(status.label),
                              selected: controller.selectedStatus == status,
                              onSelected: (_) => unawaited(
                                controller.selectStatus(
                                  controller.selectedStatus == status
                                      ? null
                                      : status,
                                ),
                              ),
                            ),
                          ],
                        ],
                      ),
                    ),
                  ],
                ),
              ),
              Expanded(
                child: RefreshIndicator(
                  onRefresh: controller.loadDemands,
                  child: _buildBody(context, controller, visibleDemands),
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildBody(
    BuildContext context,
    DemandsController controller,
    List<Demand> visibleDemands,
  ) {
    if (controller.isLoadingList && controller.demands.isEmpty) {
      return ListView(
        physics: const AlwaysScrollableScrollPhysics(),
        children: const [
          SizedBox(height: 120),
          Center(child: CircularProgressIndicator()),
        ],
      );
    }

    if (controller.isInstitutionalForumForbidden) {
      return ListView(
        physics: const AlwaysScrollableScrollPhysics(),
        padding: const EdgeInsets.fromLTRB(16, 12, 16, 88),
        children: [
          Semantics(
            liveRegion: true,
            child: WarningBox(
              message: demandFailureMessage(controller.listFailure!),
            ),
          ),
          const SizedBox(height: 12),
          Wrap(
            spacing: 12,
            runSpacing: 8,
            alignment: WrapAlignment.end,
            children: [
              TextButton(
                onPressed: () => unawaited(controller.loadDemands()),
                child: const Text('Tentar novamente'),
              ),
              FilledButton.icon(
                onPressed: () =>
                    unawaited(controller.selectForum(DemandForum.physicalSpace)),
                icon: const Icon(Icons.meeting_room_outlined),
                label: const Text('Ir para Espaço Físico'),
              ),
            ],
          ),
        ],
      );
    }

    if (controller.listFailure != null && controller.demands.isEmpty) {
      return ListView(
        physics: const AlwaysScrollableScrollPhysics(),
        padding: const EdgeInsets.fromLTRB(16, 12, 16, 88),
        children: [
          WarningBox(message: demandFailureMessage(controller.listFailure!)),
          const SizedBox(height: 12),
          Align(
            alignment: Alignment.centerRight,
            child: FilledButton(
              onPressed: () => unawaited(controller.loadDemands()),
              child: const Text('Tentar novamente'),
            ),
          ),
        ],
      );
    }

    if (visibleDemands.isEmpty) {
      return ListView(
        physics: const AlwaysScrollableScrollPhysics(),
        padding: const EdgeInsets.fromLTRB(20, 48, 20, 88),
        children: [
          _EmptyDemandsState(
            controller: controller,
            onClearSearch: () {
              _searchController.clear();
              controller.setSearchQuery('');
            },
            onCreateDemand: _openCreateSheet,
          ),
        ],
      );
    }

    return ListView.separated(
      physics: const AlwaysScrollableScrollPhysics(),
      padding: const EdgeInsets.fromLTRB(16, 8, 16, 88),
      itemCount: visibleDemands.length,
      separatorBuilder: (_, _) => const SizedBox(height: 12),
      itemBuilder: (context, index) {
        final demand = visibleDemands[index];
        return DemandCard(
          demand: demand,
          currentUserId: controller.currentUserId,
          commentCount: controller.commentCountFor(demand.id),
          onTap: () => unawaited(_openDetail(demand)),
        );
      },
    );
  }
}

class _EmptyDemandsState extends StatelessWidget {
  const _EmptyDemandsState({
    required this.controller,
    required this.onClearSearch,
    required this.onCreateDemand,
  });

  final DemandsController controller;
  final VoidCallback onClearSearch;
  final VoidCallback onCreateDemand;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;

    if (controller.searchQuery.trim().isNotEmpty) {
      return Column(
        children: [
          Icon(Icons.search_off, size: 44, color: colors.onSurfaceVariant),
          const SizedBox(height: 12),
          Text(
            'Nenhuma demanda corresponde à busca.',
            style: theme.textTheme.titleMedium,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 12),
          TextButton(
            onPressed: onClearSearch,
            child: const Text('Limpar busca'),
          ),
        ],
      );
    }

    if (controller.onlyMine) {
      return Column(
        children: [
          Icon(Icons.inbox_outlined, size: 44, color: colors.onSurfaceVariant),
          const SizedBox(height: 12),
          Text(
            'Você ainda não registrou demandas neste fórum.',
            style: theme.textTheme.titleMedium,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 6),
          Text(
            'Você pode remover o filtro para acompanhar as demandas coletivas.',
            style: theme.textTheme.bodyMedium?.copyWith(
              color: colors.onSurfaceVariant,
            ),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 14),
          FilledButton.tonal(
            onPressed: () => unawaited(controller.toggleOnlyMine(false)),
            child: const Text('Ver todas do fórum'),
          ),
        ],
      );
    }

    if (controller.selectedStatus != null) {
      return Column(
        children: [
          Icon(
            Icons.filter_list_off,
            size: 44,
            color: colors.onSurfaceVariant,
          ),
          const SizedBox(height: 12),
          Text(
            'Nenhuma demanda com status "${controller.selectedStatus!.label}".',
            style: theme.textTheme.titleMedium,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 12),
          TextButton(
            onPressed: () => unawaited(controller.clearFilters()),
            child: const Text('Limpar filtros'),
          ),
        ],
      );
    }

    return Column(
      children: [
        Icon(Icons.forum_outlined, size: 44, color: colors.onSurfaceVariant),
        const SizedBox(height: 12),
        Text(
          'Nenhuma demanda registrada neste fórum.',
          style: theme.textTheme.titleMedium,
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 6),
        Text(
          'Registre a primeira necessidade para acompanhar com o CA.',
          style: theme.textTheme.bodyMedium?.copyWith(
            color: colors.onSurfaceVariant,
          ),
          textAlign: TextAlign.center,
        ),
        if (controller.canCreateInSelectedForum) ...[
          const SizedBox(height: 14),
          FilledButton.icon(
            onPressed: onCreateDemand,
            icon: const Icon(Icons.add),
            label: const Text('Criar demanda'),
          ),
        ],
      ],
    );
  }
}
