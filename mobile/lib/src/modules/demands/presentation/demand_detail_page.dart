/// Tela de detalhes, histórico cronológico e comentários de uma demanda.
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:uuid/uuid.dart';

import '../../../presentation/presentation.dart';
import '../application/demands_controller.dart';
import '../application/error/demand_exception.dart';
import '../application/models/create_demand_input.dart';
import '../application/models/demand.dart';
import '../application/models/demand_comment.dart';
import '../application/models/demand_status_history_entry.dart';
import 'demand_failure_message.dart';
import 'demand_formatting.dart';
import 'update_demand_status_sheet.dart';
import 'widgets/demand_badges.dart';

/// Exibe detalhes completos, histórico de status e interações de [demandId].
class DemandDetailPage extends StatefulWidget {
  const DemandDetailPage({
    super.key,
    required this.controller,
    required this.demandId,
    this.initialDemand,
  });

  final DemandsController controller;
  final UuidValue demandId;
  final Demand? initialDemand;

  @override
  State<DemandDetailPage> createState() => _DemandDetailPageState();
}

class _DemandDetailPageState extends State<DemandDetailPage> {
  final _commentFormKey = GlobalKey<FormState>();
  final _commentController = TextEditingController();
  bool _loadingDetail = true;
  bool _loadingComments = true;
  bool _submittingComment = false;
  bool _commentsForbidden = false;
  String? _detailError;
  String? _commentsError;
  String? _commentSubmitError;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) unawaited(_refreshAll());
    });
  }

  @override
  void dispose() {
    _commentController.dispose();
    super.dispose();
  }

  Future<void> _refreshAll() async {
    await Future.wait([_loadDetail(), _loadComments()]);
  }

  Future<void> _loadDetail() async {
    if (!mounted) return;
    setState(() {
      _loadingDetail = true;
      _detailError = null;
    });
    try {
      await widget.controller.loadDemandDetail(widget.demandId);
    } on DemandException catch (error) {
      if (mounted) setState(() => _detailError = demandFailureMessage(error));
    } finally {
      if (mounted) setState(() => _loadingDetail = false);
    }
  }

  Future<void> _loadComments() async {
    if (!mounted) return;
    setState(() {
      _loadingComments = true;
      _commentsError = null;
      _commentsForbidden = false;
    });
    try {
      await widget.controller.loadComments(widget.demandId);
    } on DemandException catch (error) {
      if (!mounted) return;
      if (error.kind == DemandFailureKind.forbiddenCaForum ||
          error.kind == DemandFailureKind.studentMembershipRequired) {
        setState(() => _commentsForbidden = true);
      } else {
        setState(() => _commentsError = demandFailureMessage(error));
      }
    } finally {
      if (mounted) setState(() => _loadingComments = false);
    }
  }

  Future<void> _submitComment() async {
    if (_submittingComment || !_commentFormKey.currentState!.validate()) return;

    setState(() {
      _submittingComment = true;
      _commentSubmitError = null;
    });

    try {
      await widget.controller.postComment(
        widget.demandId,
        _commentController.text,
      );
      if (!mounted) return;
      _commentController.clear();
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('Comentário publicado.')));
    } on FormatException catch (error) {
      if (mounted) setState(() => _commentSubmitError = error.message);
    } on DemandException catch (error) {
      if (mounted) {
        setState(() => _commentSubmitError = demandFailureMessage(error));
      }
    } finally {
      if (mounted) setState(() => _submittingComment = false);
    }
  }

  Future<void> _openStatusUpdate(Demand demand) async {
    final updated = await showUpdateDemandStatusSheet(
      context,
      controller: widget.controller,
      demand: demand,
    );
    if (updated != null && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Status atualizado com sucesso.')),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Detalhes da demanda')),
      body: SafeArea(
        child: ListenableBuilder(
          listenable: widget.controller,
          builder: (context, _) {
            final demand =
                widget.controller.demandById(widget.demandId) ??
                widget.initialDemand;
            final comments = widget.controller.commentsFor(widget.demandId);

            if (demand == null && _loadingDetail) {
              return const Center(child: CircularProgressIndicator());
            }

            if (demand == null && _detailError != null) {
              return Center(
                child: Padding(
                  padding: const EdgeInsets.all(24),
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      WarningBox(message: _detailError!),
                      const SizedBox(height: 12),
                      FilledButton(
                        onPressed: _refreshAll,
                        child: const Text('Tentar novamente'),
                      ),
                    ],
                  ),
                ),
              );
            }

            if (demand == null) {
              return const Center(child: Text('Demanda não encontrada.'));
            }

            final canComment =
                !_commentsForbidden &&
                (widget.controller.isCaManager ||
                    widget.controller.currentUserId == demand.authorId);

            return RefreshIndicator(
              onRefresh: _refreshAll,
              child: ListView(
                physics: const AlwaysScrollableScrollPhysics(),
                padding: const EdgeInsets.all(20),
                children: [
                  Center(
                    child: ConstrainedBox(
                      constraints: const BoxConstraints(maxWidth: 640),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          if (_detailError != null) ...[
                            WarningBox(message: _detailError!),
                            const SizedBox(height: 12),
                          ],
                          _DemandHeaderCard(
                            demand: demand,
                            currentUserId: widget.controller.currentUserId,
                            isCaManager: widget.controller.isCaManager,
                            onUpdateStatus: () => _openStatusUpdate(demand),
                          ),
                          const SizedBox(height: 20),
                          _StatusHistorySection(
                            history: demand.statusHistory,
                            currentUserId: widget.controller.currentUserId,
                            isLoading:
                                _loadingDetail && demand.statusHistory.isEmpty,
                          ),
                          const SizedBox(height: 20),
                          _CommentsSection(
                            comments: comments,
                            currentUserId: widget.controller.currentUserId,
                            isLoading: _loadingComments && comments == null,
                            isForbidden: _commentsForbidden,
                            errorMessage: _commentsError,
                            onRetry: _loadComments,
                            canComment: canComment,
                            formKey: _commentFormKey,
                            commentController: _commentController,
                            submitting: _submittingComment,
                            submitError: _commentSubmitError,
                            onSubmit: _submitComment,
                          ),
                        ],
                      ),
                    ),
                  ),
                ],
              ),
            );
          },
        ),
      ),
    );
  }
}

class _DemandHeaderCard extends StatelessWidget {
  const _DemandHeaderCard({
    required this.demand,
    required this.currentUserId,
    required this.isCaManager,
    required this.onUpdateStatus,
  });

  final Demand demand;
  final UuidValue? currentUserId;
  final bool isCaManager;
  final VoidCallback onUpdateStatus;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;
    final title = demand.title?.trim().isNotEmpty == true
        ? demand.title!.trim()
        : 'Demanda sem título';

    return Card(
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(16),
        side: BorderSide(color: colors.outlineVariant.withValues(alpha: 0.45)),
      ),
      child: Padding(
        padding: const EdgeInsets.all(18),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                DemandForumBadge(forum: demand.forum),
                DemandStatusBadge(status: demand.status),
              ],
            ),
            const SizedBox(height: 14),
            Text(
              title,
              style: theme.textTheme.titleLarge?.copyWith(
                fontWeight: FontWeight.w700,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              'Por ${formatDemandAuthor(demand.authorId, currentUserId)} · '
              'Criada em '
              '${formatDemandDate(demand.createdAt, includeTime: true)}',
              style: theme.textTheme.bodySmall?.copyWith(
                color: colors.onSurfaceVariant,
              ),
            ),
            if (demand.courseName != null || demand.category != null) ...[
              const SizedBox(height: 12),
              Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  if (demand.courseName case final course?)
                    Chip(
                      avatar: const Icon(Icons.menu_book_outlined, size: 16),
                      label: Text(course),
                    ),
                  if (demand.category case final category?)
                    Chip(
                      avatar: const Icon(Icons.label_outline, size: 16),
                      label: Text(category),
                    ),
                ],
              ),
            ],
            const SizedBox(height: 16),
            Text(demand.description, style: theme.textTheme.bodyLarge),
            if (isCaManager) ...[
              const SizedBox(height: 18),
              FilledButton.tonalIcon(
                onPressed: onUpdateStatus,
                icon: const Icon(Icons.sync_alt),
                label: const Text('Atualizar status'),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

class _StatusHistorySection extends StatelessWidget {
  const _StatusHistorySection({
    required this.history,
    required this.currentUserId,
    required this.isLoading,
  });

  final List<DemandStatusHistoryEntry> history;
  final UuidValue? currentUserId;
  final bool isLoading;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;
    final ordered = List<DemandStatusHistoryEntry>.of(history)
      ..sort((a, b) => a.createdAt.compareTo(b.createdAt));

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(
          'Histórico de status',
          style: theme.textTheme.titleMedium?.copyWith(
            fontWeight: FontWeight.w600,
          ),
        ),
        const SizedBox(height: 10),
        if (isLoading)
          const Padding(
            padding: EdgeInsets.symmetric(vertical: 12),
            child: Center(child: CircularProgressIndicator()),
          )
        else if (ordered.isEmpty)
          Card(
            margin: EdgeInsets.zero,
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Text(
                'Nenhum registro de alteração de status disponível.',
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: colors.onSurfaceVariant,
                ),
              ),
            ),
          )
        else
          Card(
            margin: EdgeInsets.zero,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(16),
              side: BorderSide(
                color: colors.outlineVariant.withValues(alpha: 0.45),
              ),
            ),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                children: [
                  for (var i = 0; i < ordered.length; i++)
                    _HistoryTimelineTile(
                      entry: ordered[i],
                      currentUserId: currentUserId,
                      isLast: i == ordered.length - 1,
                    ),
                ],
              ),
            ),
          ),
      ],
    );
  }
}

class _HistoryTimelineTile extends StatelessWidget {
  const _HistoryTimelineTile({
    required this.entry,
    required this.currentUserId,
    required this.isLast,
  });

  final DemandStatusHistoryEntry entry;
  final UuidValue? currentUserId;
  final bool isLast;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;
    final transitionLabel = entry.fromStatus == null
        ? 'Registro inicial: ${entry.toStatus.label}'
        : '${entry.fromStatus!.label} → ${entry.toStatus.label}';
    final roleFallback = entry.fromStatus == null ? 'Autor' : 'Gestão';
    final author = formatDemandAuthor(
      entry.changedById,
      currentUserId,
      fallbackRole: roleFallback,
    );
    final date = formatDemandDate(entry.createdAt, includeTime: true);

    return Padding(
      padding: EdgeInsets.only(bottom: isLast ? 0 : 14),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Icon(
              Icons.circle,
              size: 10,
              color: isLast ? colors.primary : colors.outline,
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Wrap(
                  spacing: 8,
                  runSpacing: 6,
                  crossAxisAlignment: WrapCrossAlignment.center,
                  children: [
                    Text(
                      transitionLabel,
                      style: theme.textTheme.bodyMedium?.copyWith(
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                    DemandStatusBadge(status: entry.toStatus),
                  ],
                ),
                const SizedBox(height: 4),
                Text(
                  '$author · $date',
                  style: theme.textTheme.bodySmall?.copyWith(
                    color: colors.onSurfaceVariant,
                  ),
                ),
                if (entry.opinion case final opinion?
                    when opinion.trim().isNotEmpty) ...[
                  const SizedBox(height: 6),
                  Text(opinion, style: theme.textTheme.bodyMedium),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _CommentsSection extends StatelessWidget {
  const _CommentsSection({
    required this.comments,
    required this.currentUserId,
    required this.isLoading,
    required this.isForbidden,
    required this.errorMessage,
    required this.onRetry,
    required this.canComment,
    required this.formKey,
    required this.commentController,
    required this.submitting,
    required this.submitError,
    required this.onSubmit,
  });

  final List<DemandComment>? comments;
  final UuidValue? currentUserId;
  final bool isLoading;
  final bool isForbidden;
  final String? errorMessage;
  final VoidCallback onRetry;
  final bool canComment;
  final GlobalKey<FormState> formKey;
  final TextEditingController commentController;
  final bool submitting;
  final String? submitError;
  final VoidCallback onSubmit;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(
          'Comentários e interações',
          style: theme.textTheme.titleMedium?.copyWith(
            fontWeight: FontWeight.w600,
          ),
        ),
        const SizedBox(height: 10),
        if (isLoading)
          const Padding(
            padding: EdgeInsets.symmetric(vertical: 12),
            child: Center(child: CircularProgressIndicator()),
          )
        else if (isForbidden)
          const WarningBox(
            message:
                'Os comentários e interações desta demanda são visíveis apenas '
                'para o autor da solicitação e para a gestão do CA.',
          )
        else if (errorMessage != null && comments == null) ...[
          WarningBox(message: errorMessage!),
          const SizedBox(height: 8),
          Align(
            alignment: Alignment.centerRight,
            child: TextButton(
              onPressed: onRetry,
              child: const Text('Tentar novamente'),
            ),
          ),
        ] else if (comments != null && comments!.isEmpty)
          Card(
            margin: EdgeInsets.zero,
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Text(
                'Nenhum comentário publicado ainda.',
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: colors.onSurfaceVariant,
                ),
              ),
            ),
          )
        else if (comments != null)
          Column(
            children: [
              for (var i = 0; i < comments!.length; i++) ...[
                _CommentTile(
                  comment: comments![i],
                  currentUserId: currentUserId,
                ),
                if (i < comments!.length - 1) const SizedBox(height: 10),
              ],
            ],
          ),
        if (canComment) ...[
          const SizedBox(height: 16),
          Form(
            key: formKey,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                TextFormField(
                  controller: commentController,
                  enabled: !submitting,
                  maxLength: maxDemandCommentLength,
                  minLines: 2,
                  maxLines: 5,
                  decoration: InputDecoration(
                    labelText: 'Adicionar comentário',
                    hintText: 'Escreva uma atualização ou resposta',
                    alignLabelWithHint: true,
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(12),
                    ),
                  ),
                  validator: (value) {
                    if (value == null || value.trim().isEmpty) {
                      return 'Informe o texto do comentário.';
                    }
                    return null;
                  },
                ),
                if (submitError case final error?) ...[
                  const SizedBox(height: 8),
                  Semantics(
                    liveRegion: true,
                    child: WarningBox(message: error),
                  ),
                ],
                const SizedBox(height: 10),
                Align(
                  alignment: Alignment.centerRight,
                  child: FilledButton.icon(
                    onPressed: submitting ? null : onSubmit,
                    icon: const Icon(Icons.send),
                    label: submitting
                        ? const AnimatedEllipsisText('Enviando')
                        : const Text('Enviar comentário'),
                  ),
                ),
              ],
            ),
          ),
        ],
      ],
    );
  }
}

class _CommentTile extends StatelessWidget {
  const _CommentTile({required this.comment, required this.currentUserId});

  final DemandComment comment;
  final UuidValue? currentUserId;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;
    final isManagement =
        comment.authorRole == DemandCommentAuthorRole.caManagement;
    final authorLabel = formatDemandAuthor(
      comment.authorId,
      currentUserId,
      fallbackRole: isManagement ? 'Gestão' : 'Estudante',
    );
    final date = formatDemandDate(comment.createdAt, includeTime: true);

    return Card(
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(14),
        side: BorderSide(
          color: isManagement
              ? colors.primary.withValues(alpha: 0.45)
              : colors.outlineVariant.withValues(alpha: 0.4),
        ),
      ),
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 8,
                    vertical: 3,
                  ),
                  decoration: BoxDecoration(
                    color: isManagement
                        ? colors.primaryContainer
                        : colors.surfaceContainerHighest,
                    borderRadius: BorderRadius.circular(999),
                  ),
                  child: Text(
                    comment.authorRole.label,
                    style: theme.textTheme.labelSmall?.copyWith(
                      color: isManagement
                          ? colors.onPrimaryContainer
                          : colors.onSurfaceVariant,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    '$authorLabel · $date',
                    style: theme.textTheme.bodySmall?.copyWith(
                      color: colors.onSurfaceVariant,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(comment.content, style: theme.textTheme.bodyMedium),
          ],
        ),
      ),
    );
  }
}
