/// Implementação controlada de [DemandsGateway] para testes unitários e de widget.
library;

import 'package:capp/src/modules/demands/demands.dart';
import 'package:uuid/uuid.dart';

const testCaId = UuidValue.raw('00000000-0000-0000-0000-000000000001');
const testUserId = UuidValue.raw('00000000-0000-0000-0000-000000000010');
const testOtherUserId = UuidValue.raw('00000000-0000-0000-0000-000000000011');

Demand sampleDemand({
  UuidValue? id,
  UuidValue? academicCenterId,
  UuidValue? authorId,
  DemandForum forum = DemandForum.physicalSpace,
  String? title = 'Reparo da porta',
  String description = 'A porta da sala do CA está danificada.',
  String? category = 'Infraestrutura',
  String? courseName,
  DemandStatus status = DemandStatus.open,
  DateTime? createdAt,
  DateTime? updatedAt,
  List<DemandStatusHistoryEntry> statusHistory = const [],
  int? commentCount,
}) {
  final demandId =
      id ?? const UuidValue.raw('00000000-0000-0000-0000-000000000020');
  final created = createdAt ?? DateTime.utc(2026, 11, 20, 12);
  return Demand(
    id: demandId,
    academicCenterId: academicCenterId ?? testCaId,
    authorId: authorId ?? testUserId,
    forum: forum,
    title: title,
    description: description,
    category: category,
    courseName: courseName,
    status: status,
    createdAt: created,
    updatedAt: updatedAt ?? created,
    statusHistory: statusHistory,
    commentCount: commentCount,
  );
}

final class FakeDemandsGateway implements DemandsGateway {
  FakeDemandsGateway({
    List<Demand> initialDemands = const [],
    Map<UuidValue, List<DemandComment>>? initialComments,
    this.forbidInstitutional = false,
    this.forbidComments = false,
    this.actorId = testUserId,
    this.actorIsManager = false,
  }) : _demands = List<Demand>.of(initialDemands),
       _comments = {
         if (initialComments != null)
           for (final entry in initialComments.entries)
             entry.key: List<DemandComment>.of(entry.value),
       };

  final List<Demand> _demands;
  final Map<UuidValue, List<DemandComment>> _comments;
  bool forbidInstitutional;
  bool forbidComments;
  UuidValue actorId;
  bool actorIsManager;
  int _nextId = 100;

  Future<List<Demand>> Function({
    DemandForum? forum,
    DemandStatus? status,
    bool mine,
  })?
  onListDemands;

  DemandException? listError;
  DemandException? createError;
  DemandException? commentError;
  DemandException? updateStatusError;

  List<Demand> defaultList({
    DemandForum? forum,
    DemandStatus? status,
    bool mine = false,
  }) {
    return _demands.where((d) {
      if (forum != null && d.forum != forum) return false;
      if (status != null && d.status != status) return false;
      if (mine && d.authorId != actorId) return false;
      return true;
    }).toList();
  }

  @override
  Future<List<Demand>> listDemands({
    required UuidValue academicCenterId,
    DemandForum? forum,
    DemandStatus? status,
    bool mine = false,
    int? limit,
    int? offset,
  }) async {
    if (listError != null) throw listError!;
    if (onListDemands != null) {
      return onListDemands!(forum: forum, status: status, mine: mine);
    }
    if (forbidInstitutional && forum == DemandForum.institutional) {
      throw DemandException(
        kind: DemandFailureKind.forbiddenCaForum,
        cause: Exception('403'),
        stackTrace: StackTrace.current,
        backendCode: 'user_not_permitted_in_ca',
      );
    }
    return defaultList(forum: forum, status: status, mine: mine)
        .map(
          (d) => d.copyWith(
            commentCount: _comments[d.id]?.length ?? d.commentCount,
          ),
        )
        .toList();
  }

  @override
  Future<Demand> createDemand({
    required UuidValue academicCenterId,
    required CreateDemandInput input,
  }) async {
    if (createError != null) throw createError!;
    final idSeq = (_nextId++).toString().padLeft(12, '0');
    final historySeq = (_nextId++).toString().padLeft(12, '0');
    final id = UuidValue.fromString('00000000-0000-0000-0000-$idSeq');
    final now = DateTime.utc(2026, 11, 24, 15);
    final created = Demand(
      id: id,
      academicCenterId: academicCenterId,
      authorId: actorId,
      forum: input.forum,
      title: input.title,
      description: input.description,
      category: input.category,
      courseName: input.courseName,
      status: DemandStatus.open,
      createdAt: now,
      updatedAt: now,
      statusHistory: [
        DemandStatusHistoryEntry(
          id: UuidValue.fromString('00000000-0000-0000-0001-$historySeq'),
          demandId: id,
          fromStatus: null,
          toStatus: DemandStatus.open,
          changedById: actorId,
          opinion: null,
          createdAt: now,
        ),
      ],
      commentCount: 0,
    );
    _demands.insert(0, created);
    return created;
  }

  @override
  Future<Demand> getDemand({
    required UuidValue academicCenterId,
    required UuidValue demandId,
  }) async {
    Demand? demand;
    for (final item in _demands) {
      if (item.id == demandId) {
        demand = item;
        break;
      }
    }
    if (demand == null || demand.academicCenterId != academicCenterId) {
      throw DemandException(
        kind: DemandFailureKind.demandNotFound,
        cause: Exception('404'),
        stackTrace: StackTrace.current,
        backendCode: 'demand_not_found',
      );
    }
    return demand;
  }

  @override
  Future<Demand> updateDemandStatus({
    required UuidValue academicCenterId,
    required UuidValue demandId,
    required DemandStatus status,
    required String opinion,
  }) async {
    if (updateStatusError != null) throw updateStatusError!;
    final index = _demands.indexWhere((d) => d.id == demandId);
    if (index == -1) {
      throw DemandException(
        kind: DemandFailureKind.demandNotFound,
        cause: Exception('404'),
        stackTrace: StackTrace.current,
        backendCode: 'demand_not_found',
      );
    }
    final current = _demands[index];
    final now = DateTime.utc(2026, 11, 24, 16);
    final entry = DemandStatusHistoryEntry(
      id: UuidValue.fromString(
        '00000000-0000-0000-0002-${(_nextId++).toString().padLeft(12, '0')}',
      ),
      demandId: current.id,
      fromStatus: current.status,
      toStatus: status,
      changedById: actorId,
      opinion: opinion,
      createdAt: now,
    );
    final updated = current.copyWith(
      status: status,
      updatedAt: now,
      statusHistory: [...current.statusHistory, entry],
    );
    _demands[index] = updated;
    return updated;
  }

  @override
  Future<List<DemandComment>> listComments({
    required UuidValue academicCenterId,
    required UuidValue demandId,
  }) async {
    if (forbidComments) {
      throw DemandException(
        kind: DemandFailureKind.forbiddenCaForum,
        cause: Exception('403'),
        stackTrace: StackTrace.current,
        backendCode: 'user_not_permitted_in_ca',
      );
    }
    return List<DemandComment>.of(_comments[demandId] ?? const []);
  }

  @override
  Future<DemandComment> createComment({
    required UuidValue academicCenterId,
    required UuidValue demandId,
    required String content,
  }) async {
    if (commentError != null) throw commentError!;
    final created = DemandComment(
      id: UuidValue.fromString(
        '00000000-0000-0000-0003-${(_nextId++).toString().padLeft(12, '0')}',
      ),
      demandId: demandId,
      authorId: actorId,
      authorRole: actorIsManager
          ? DemandCommentAuthorRole.caManagement
          : DemandCommentAuthorRole.student,
      content: content,
      createdAt: DateTime.utc(2026, 11, 24, 16, 30),
    );
    _comments.putIfAbsent(demandId, () => []).add(created);
    return created;
  }
}
