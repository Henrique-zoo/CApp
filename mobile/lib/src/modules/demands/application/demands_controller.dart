/// Coordenação de estado dos fóruns, filtros, detalhes e interações de demandas.
library;

import 'dart:collection';

import 'package:flutter/foundation.dart';
import 'package:uuid/uuid.dart';

import 'error/demand_exception.dart';
import 'models/create_demand_input.dart';
import 'models/demand.dart';
import 'models/demand_comment.dart';
import 'models/demand_forum.dart';
import 'models/demand_status.dart';
import 'ports/demands_gateway.dart';

/// Controlador reativo para consulta e acompanhamento dos fóruns de demandas.
final class DemandsController extends ChangeNotifier {
  DemandsController({
    required this._gateway,
    required this.academicCenterId,
    this.academicCenterName,
    this.currentUserId,
    this.userCourseName,
    this.hasStudentMembership = true,
    this.isCaManager = false,
    DemandForum initialForum = DemandForum.institutional,
  }) : _selectedForum = initialForum;

  final DemandsGateway _gateway;

  /// Centro Acadêmico ativo na consulta.
  final UuidValue academicCenterId;

  /// Nome de exibição do Centro Acadêmico ativo, quando conhecido.
  final String? academicCenterName;

  /// Identificador interno do usuário autenticado, usado para destacar autoria.
  final UuidValue? currentUserId;

  /// Curso confirmado institucionalmente para o estudante autenticado.
  final String? userCourseName;

  /// Indica se a conta possui vínculo estudantil confirmado.
  final bool hasStudentMembership;

  /// Indica se o usuário possui papel de gestão neste Centro Acadêmico.
  final bool isCaManager;

  DemandForum _selectedForum;
  DemandStatus? _selectedStatus;
  bool _onlyMine = false;
  String _searchQuery = '';
  List<Demand> _demands = const [];
  final Map<UuidValue, Demand> _detailedDemands = {};
  final Map<UuidValue, List<DemandComment>> _commentsByDemand = {};
  bool _isLoadingList = false;
  bool _isCreatingDemand = false;
  DemandException? _listFailure;
  int _listRequestSerial = 0;
  bool _disposed = false;

  /// Fórum atualmente selecionado nas abas.
  DemandForum get selectedForum => _selectedForum;

  /// Filtro de status aplicado na listagem, ou `null` para todos.
  DemandStatus? get selectedStatus => _selectedStatus;

  /// Indica se o filtro de demandas próprias (`mine`) está ativo.
  bool get onlyMine => _onlyMine;

  /// Termo de busca local aplicado sobre título, descrição, categoria e curso.
  String get searchQuery => _searchQuery;

  /// Indica se a listagem do fórum está em carregamento.
  bool get isLoadingList => _isLoadingList;

  /// Indica se há uma criação de demanda em andamento.
  bool get isCreatingDemand => _isCreatingDemand;

  /// Falha mais recente da listagem atual, se houver.
  DemandException? get listFailure => _listFailure;

  /// Indica se o fórum institucional foi negado por ausência de vínculo/curso (403).
  bool get isInstitutionalForumForbidden =>
      _selectedForum == DemandForum.institutional &&
      (_listFailure?.kind == DemandFailureKind.forbiddenCaForum ||
          _listFailure?.kind == DemandFailureKind.studentMembershipRequired);

  /// Indica se o usuário pode abrir criação diretamente no fórum selecionado.
  bool get canCreateInSelectedForum {
    if (_selectedForum == DemandForum.physicalSpace) {
      return hasStudentMembership || isCaManager;
    }
    if (isInstitutionalForumForbidden) return false;
    return isCaManager ||
        (hasStudentMembership && (userCourseName?.trim().isNotEmpty ?? false));
  }

  /// Lista bruta carregada para o fórum e filtros remotos atuais.
  List<Demand> get demands => UnmodifiableListView(_demands);

  /// Lista filtrada pela busca textual local, preservando a ordem do backend.
  List<Demand> get visibleDemands {
    final query = _searchQuery.trim().toLowerCase();
    if (query.isEmpty) return demands;
    return UnmodifiableListView(
      _demands.where((demand) {
        final title = demand.title?.toLowerCase() ?? '';
        final description = demand.description.toLowerCase();
        final category = demand.category?.toLowerCase() ?? '';
        final course = demand.courseName?.toLowerCase() ?? '';
        return title.contains(query) ||
            description.contains(query) ||
            category.contains(query) ||
            course.contains(query);
      }),
    );
  }

  /// Retorna a versão mais completa conhecida da demanda (detalhe ou lista).
  Demand? demandById(UuidValue demandId) {
    final detailed = _detailedDemands[demandId];
    if (detailed != null) return detailed;
    for (final demand in _demands) {
      if (demand.id == demandId) return demand;
    }
    return null;
  }

  /// Comentários já carregados para [demandId], se disponíveis.
  List<DemandComment>? commentsFor(UuidValue demandId) {
    final comments = _commentsByDemand[demandId];
    return comments == null ? null : UnmodifiableListView(comments);
  }

  /// Contagem de comentários conhecida para [demandId], se disponível.
  int? commentCountFor(UuidValue demandId) =>
      _commentsByDemand[demandId]?.length ?? demandById(demandId)?.commentCount;

  /// Carrega as demandas do fórum e filtros ativos.
  Future<void> loadDemands() async {
    if (_disposed) return;

    final requestId = ++_listRequestSerial;
    _isLoadingList = true;
    _listFailure = null;
    _notify();

    try {
      final result = await _gateway.listDemands(
        academicCenterId: academicCenterId,
        forum: _selectedForum,
        status: _selectedStatus,
        mine: _onlyMine,
      );
      if (_disposed || requestId != _listRequestSerial) return;
      _demands = result.map(_mergeCachedCount).toList();
      _listFailure = null;
    } on DemandException catch (error) {
      if (_disposed || requestId != _listRequestSerial) return;
      _demands = const [];
      _listFailure = error;
    } finally {
      if (!_disposed && requestId == _listRequestSerial) {
        _isLoadingList = false;
        _notify();
      }
    }
  }

  /// Alterna entre o fórum institucional e o fórum do espaço físico.
  Future<void> selectForum(DemandForum forum) async {
    if (_selectedForum == forum && _listFailure == null) return;
    _selectedForum = forum;
    if (forum == DemandForum.physicalSpace &&
        _selectedStatus == DemandStatus.forwardedToCoordination) {
      _selectedStatus = null;
    }
    await loadDemands();
  }

  /// Aplica ou limpa o filtro de status e recarrega a listagem.
  Future<void> selectStatus(DemandStatus? status) async {
    if (_selectedStatus == status) return;
    _selectedStatus = status;
    await loadDemands();
  }

  /// Alterna o filtro de demandas registradas pelo próprio usuário (`mine`).
  Future<void> toggleOnlyMine([bool? value]) async {
    final next = value ?? !_onlyMine;
    if (_onlyMine == next) return;
    _onlyMine = next;
    await loadDemands();
  }

  /// Atualiza o termo de busca local sem disparar requisição HTTP.
  void setSearchQuery(String query) {
    if (_searchQuery == query) return;
    _searchQuery = query;
    _notify();
  }

  /// Limpa filtros de status, minhas demandas e busca textual.
  Future<void> clearFilters() async {
    final hadRemoteFilter = _selectedStatus != null || _onlyMine;
    _selectedStatus = null;
    _onlyMine = false;
    _searchQuery = '';
    if (hadRemoteFilter) {
      await loadDemands();
    } else {
      _notify();
    }
  }

  /// Cria uma nova demanda no fórum escolhido e atualiza a listagem.
  Future<Demand> createDemand(CreateDemandInput input) async {
    if (_disposed) {
      throw StateError('DemandsController já foi encerrado');
    }
    _isCreatingDemand = true;
    _notify();

    try {
      final created = await _gateway.createDemand(
        academicCenterId: academicCenterId,
        input: input,
      );
      if (_disposed) return created;

      _detailedDemands[created.id] = created;
      if (input.forum != _selectedForum) {
        _selectedForum = input.forum;
        _selectedStatus = null;
        await loadDemands();
      } else if (_selectedStatus == null || _selectedStatus == created.status) {
        _demands = [created, ..._demands.where((d) => d.id != created.id)];
      }
      return created;
    } finally {
      if (!_disposed) {
        _isCreatingDemand = false;
        _notify();
      }
    }
  }

  /// Consulta os detalhes atualizados e o histórico completo de uma demanda.
  Future<Demand> loadDemandDetail(UuidValue demandId) async {
    final detail = await _gateway.getDemand(
      academicCenterId: academicCenterId,
      demandId: demandId,
    );
    if (_disposed) return detail;

    final merged = _mergeCachedCount(detail);
    _detailedDemands[demandId] = merged;
    _replaceInList(merged);
    _notify();
    return merged;
  }

  /// Carrega os comentários cronológicos de uma demanda.
  Future<List<DemandComment>> loadComments(UuidValue demandId) async {
    final comments = await _gateway.listComments(
      academicCenterId: academicCenterId,
      demandId: demandId,
    );
    if (_disposed) return comments;

    _commentsByDemand[demandId] = List<DemandComment>.of(comments);
    _updateCachedCommentCount(demandId, comments.length);
    _notify();
    return UnmodifiableListView(comments);
  }

  /// Publica um novo comentário ou resposta oficial na demanda.
  Future<DemandComment> postComment(UuidValue demandId, String content) async {
    final trimmed = content.trim();
    if (trimmed.isEmpty) {
      throw const FormatException('Informe o texto do comentário.');
    }
    if (trimmed.length > maxDemandCommentLength) {
      throw const FormatException(
        'O comentário excede o limite de $maxDemandCommentLength caracteres.',
      );
    }

    final created = await _gateway.createComment(
      academicCenterId: academicCenterId,
      demandId: demandId,
      content: trimmed,
    );
    if (_disposed) return created;

    final current = List<DemandComment>.of(_commentsByDemand[demandId] ?? []);
    current.add(created);
    _commentsByDemand[demandId] = current;
    _updateCachedCommentCount(demandId, current.length);
    _notify();
    return created;
  }

  /// Atualiza o status da demanda pela gestão do CA com parecer obrigatório.
  Future<Demand> updateDemandStatus({
    required UuidValue demandId,
    required DemandStatus status,
    required String opinion,
  }) async {
    final trimmedOpinion = opinion.trim();
    if (trimmedOpinion.isEmpty) {
      throw const FormatException(
        'Informe o parecer ou justificativa da alteração.',
      );
    }
    if (trimmedOpinion.length > maxDemandOpinionLength) {
      throw const FormatException(
        'O parecer excede o limite de $maxDemandOpinionLength caracteres.',
      );
    }

    final existing = demandById(demandId);
    if (existing != null) {
      final allowed = DemandStatus.allowedTransitions(
        forum: existing.forum,
        from: existing.status,
      );
      if (!allowed.contains(status)) {
        throw const FormatException(
          'Esta transição de status não é permitida para o fórum da demanda.',
        );
      }
    }

    final updated = await _gateway.updateDemandStatus(
      academicCenterId: academicCenterId,
      demandId: demandId,
      status: status,
      opinion: trimmedOpinion,
    );
    if (_disposed) return updated;

    final merged = _mergeCachedCount(updated);
    _detailedDemands[demandId] = merged;
    if (_selectedStatus != null && _selectedStatus != merged.status) {
      _demands = _demands.where((d) => d.id != demandId).toList();
    } else {
      _replaceInList(merged);
    }
    _notify();
    return merged;
  }

  Demand _mergeCachedCount(Demand demand) {
    final knownComments = _commentsByDemand[demand.id];
    if (knownComments != null) {
      return demand.copyWith(commentCount: knownComments.length);
    }
    final previousCount = _detailedDemands[demand.id]?.commentCount;
    if (demand.commentCount == null && previousCount != null) {
      return demand.copyWith(commentCount: previousCount);
    }
    return demand;
  }

  void _replaceInList(Demand updated) {
    final index = _demands.indexWhere((d) => d.id == updated.id);
    if (index == -1) return;
    final copy = List<Demand>.of(_demands);
    copy[index] = updated;
    _demands = copy;
  }

  void _updateCachedCommentCount(UuidValue demandId, int count) {
    final detail = _detailedDemands[demandId];
    if (detail != null) {
      _detailedDemands[demandId] = detail.copyWith(commentCount: count);
    }
    final index = _demands.indexWhere((d) => d.id == demandId);
    if (index != -1) {
      final copy = List<Demand>.of(_demands);
      copy[index] = copy[index].copyWith(commentCount: count);
      _demands = copy;
    }
  }

  void _notify() {
    if (!_disposed) notifyListeners();
  }

  @override
  void dispose() {
    _disposed = true;
    super.dispose();
  }
}
