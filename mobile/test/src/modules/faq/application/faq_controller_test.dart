/// Testes unitários do [FaqController] cobrindo consulta, filtros, troca de CA e gestão.
library;

import 'dart:async';

import 'package:capp/src/modules/faq/faq.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

final class FakeFaqGateway implements FaqGateway {
  final Map<UuidValue, List<FaqEntry>> entriesByCa = {};
  final List<({UuidValue caId, String? category, String? query})> listCalls =
      [];
  Object? nextError;
  Future<List<FaqEntry>> Function({
    required UuidValue academicCenterId,
    String? category,
    String? query,
  })?
  onListEntries;

  int _sequence = 100;

  @override
  Future<List<FaqEntry>> listEntries({
    required UuidValue academicCenterId,
    String? category,
    String? query,
  }) async {
    listCalls.add((caId: academicCenterId, category: category, query: query));
    if (nextError != null) {
      final error = nextError!;
      nextError = null;
      throw error;
    }
    if (onListEntries != null) {
      return onListEntries!(
        academicCenterId: academicCenterId,
        category: category,
        query: query,
      );
    }
    final all = entriesByCa[academicCenterId] ?? const <FaqEntry>[];
    return all.where((entry) {
      if (category != null && entry.category != category) return false;
      if (query != null && query.trim().isNotEmpty) {
        final needle = query.trim().toLowerCase();
        return entry.question.toLowerCase().contains(needle) ||
            entry.answer.toLowerCase().contains(needle);
      }
      return true;
    }).toList();
  }

  @override
  Future<List<String>> listCategories({
    required UuidValue academicCenterId,
  }) async {
    final all = entriesByCa[academicCenterId] ?? const <FaqEntry>[];
    return {
      for (final entry in all)
        if (entry.category != null && entry.category!.trim().isNotEmpty)
          entry.category!.trim(),
    }.toList()..sort();
  }

  @override
  Future<FaqEntry> getEntry({
    required UuidValue academicCenterId,
    required UuidValue entryId,
  }) async {
    final list = entriesByCa[academicCenterId] ?? const <FaqEntry>[];
    return list.firstWhere((e) => e.id == entryId);
  }

  @override
  Future<FaqEntry> createEntry({
    required UuidValue academicCenterId,
    required FaqEntryDraft draft,
  }) async {
    if (nextError != null) {
      final error = nextError!;
      nextError = null;
      throw error;
    }
    final now = DateTime.utc(2026, 11, 24, 12);
    final id = UuidValue.fromString(
      '00000000-0000-0000-0000-${(_sequence++).toString().padLeft(12, '0')}',
    );
    final created = FaqEntry(
      id: id,
      academicCenterId: academicCenterId,
      question: draft.question,
      answer: draft.answer,
      category: draft.category,
      createdBy: const UuidValue.raw('00000000-0000-0000-0000-000000000099'),
      updatedBy: const UuidValue.raw('00000000-0000-0000-0000-000000000099'),
      createdAt: now,
      updatedAt: now,
    );
    entriesByCa.putIfAbsent(academicCenterId, () => []).insert(0, created);
    return created;
  }

  @override
  Future<FaqEntry> updateEntry({
    required UuidValue academicCenterId,
    required UuidValue entryId,
    required FaqEntryDraft draft,
  }) async {
    if (nextError != null) {
      final error = nextError!;
      nextError = null;
      throw error;
    }
    final list = entriesByCa[academicCenterId] ?? <FaqEntry>[];
    final index = list.indexWhere((e) => e.id == entryId);
    final old = list[index];
    final updated = FaqEntry(
      id: old.id,
      academicCenterId: old.academicCenterId,
      question: draft.question,
      answer: draft.answer,
      category: draft.category ?? old.category,
      createdBy: old.createdBy,
      updatedBy: old.updatedBy,
      createdAt: old.createdAt,
      updatedAt: DateTime.utc(2026, 11, 24, 15),
    );
    list[index] = updated;
    return updated;
  }

  @override
  Future<void> deleteEntry({
    required UuidValue academicCenterId,
    required UuidValue entryId,
  }) async {
    if (nextError != null) {
      final error = nextError!;
      nextError = null;
      throw error;
    }
    entriesByCa[academicCenterId]?.removeWhere((e) => e.id == entryId);
  }
}

FaqEntry _entry({
  required String id,
  required UuidValue caId,
  required String question,
  required String answer,
  String? category,
}) => FaqEntry(
  id: UuidValue.fromString(id),
  academicCenterId: caId,
  question: question,
  answer: answer,
  category: category,
  createdBy: const UuidValue.raw('00000000-0000-0000-0000-000000000020'),
  updatedBy: const UuidValue.raw('00000000-0000-0000-0000-000000000020'),
  createdAt: DateTime.utc(2026, 11, 22, 10),
  updatedAt: DateTime.utc(2026, 11, 22, 11),
);

void main() {
  const caComp = UuidValue.raw('00000000-0000-0000-0000-000000000001');
  const caMath = UuidValue.raw('00000000-0000-0000-0000-000000000002');

  late FakeFaqGateway gateway;
  late FaqController controller;

  setUp(() {
    gateway = FakeFaqGateway();
    gateway.entriesByCa[caComp] = [
      _entry(
        id: '00000000-0000-0000-0000-000000000011',
        caId: caComp,
        question: 'Como solicitar apoio acadêmico?',
        answer: 'Entre em contato com a gestão do CA.',
        category: 'Apoio Acadêmico',
      ),
      _entry(
        id: '00000000-0000-0000-0000-000000000012',
        caId: caComp,
        question: 'Como pedir carteirinha de matrícula?',
        answer: 'Preencha o formulário semestral.',
        category: 'Carteirinhas',
      ),
    ];
    gateway.entriesByCa[caMath] = [
      _entry(
        id: '00000000-0000-0000-0000-000000000021',
        caId: caMath,
        question: 'Qual o horário da biblioteca?',
        answer: 'Das 8h às 20h.',
        category: 'Geral',
      ),
    ];

    controller = FaqController(
      gateway: gateway,
      academicCenterId: caComp,
      academicCenterName: 'CA de Computação',
      canManage: true,
      searchDebounce: Duration.zero,
    );
  });

  tearDown(() => controller.dispose());

  test('carrega apenas perguntas e categorias do CA ativo', () async {
    await controller.load();

    expect(controller.state.hasLoadedOnce, isTrue);
    expect(controller.state.entries, hasLength(2));
    expect(controller.state.categories, ['Apoio Acadêmico', 'Carteirinhas']);
    expect(
      controller.state.entries.every((e) => e.academicCenterId == caComp),
      isTrue,
    );
  });

  test('filtra por categoria e busca textual e limpa filtros', () async {
    await controller.load();

    await controller.selectCategory('Carteirinhas');
    expect(controller.state.selectedCategory, 'Carteirinhas');
    expect(controller.state.entries.map((e) => e.question), [
      'Como pedir carteirinha de matrícula?',
    ]);

    await controller.searchImmediately('termo inexistente');
    expect(controller.state.entries, isEmpty);
    expect(controller.state.hasActiveFilters, isTrue);

    await controller.clearFilters();
    expect(controller.state.selectedCategory, isNull);
    expect(controller.state.searchQuery, isEmpty);
    expect(controller.state.entries, hasLength(2));
  });

  test('alterna expansão inline de uma pergunta', () async {
    await controller.load();
    final id = controller.state.entries.first.id;

    expect(controller.state.expandedEntryIds, isEmpty);
    controller.toggleExpanded(id);
    expect(controller.state.expandedEntryIds, contains(id));
    controller.toggleExpanded(id);
    expect(controller.state.expandedEntryIds, isEmpty);
  });

  test('atualiza a consulta e limpa filtros ao trocar o CA ativo', () async {
    await controller.load();
    await controller.selectCategory('Carteirinhas');

    await controller.setAcademicCenter(
      caMath,
      academicCenterName: 'CA de Matemática',
    );

    expect(controller.state.academicCenterId, caMath);
    expect(controller.state.selectedCategory, isNull);
    expect(controller.state.entries.map((e) => e.question), [
      'Qual o horário da biblioteca?',
    ]);
  });

  test('descarta resposta obsoleta quando busca mais recente termina antes', () async {
    final slowFirst = Completer<List<FaqEntry>>();
    final fastSecond = Completer<List<FaqEntry>>();
    var call = 0;

    gateway.onListEntries = ({
      required academicCenterId,
      category,
      query,
    }) {
      call++;
      return call == 1 ? slowFirst.future : fastSecond.future;
    };

    final firstFuture = controller.searchImmediately('antiga');
    final secondFuture = controller.searchImmediately('nova');

    final expected = _entry(
      id: '00000000-0000-0000-0000-000000000099',
      caId: caComp,
      question: 'Resposta nova',
      answer: 'Conteúdo novo',
    );
    fastSecond.complete([expected]);
    await secondFuture;

    slowFirst.complete([
      _entry(
        id: '00000000-0000-0000-0000-000000000098',
        caId: caComp,
        question: 'Resposta antiga',
        answer: 'Conteúdo antigo',
      ),
    ]);
    await firstFuture;

    expect(controller.state.entries, [expected]);
  });

  test('cadastra, edita e remove entradas quando autorizado', () async {
    await controller.load();

    final created = await controller.createEntry(
      FaqEntryDraft(
        question: 'Onde fica a sala do CA?',
        answer: 'Bloco B, sala 12.',
        category: 'Infraestrutura',
      ),
    );
    expect(created, isTrue);
    expect(controller.state.entries, hasLength(3));
    expect(controller.state.categories, contains('Infraestrutura'));

    final createdEntry = controller.state.entries.first;
    final updated = await controller.updateEntry(
      createdEntry.id,
      FaqEntryDraft(
        question: 'Onde fica a sala do CA?',
        answer: 'Bloco B, sala 14.',
        category: 'Infraestrutura',
      ),
    );
    expect(updated, isTrue);
    expect(controller.state.entries.first.answer, 'Bloco B, sala 14.');

    final deleted = await controller.deleteEntry(createdEntry.id);
    expect(deleted, isTrue);
    expect(controller.state.entries, hasLength(2));
  });

  test('publica mensagem pública em erro de consulta e de mutação', () async {
    gateway.nextError = FaqException(
      kind: FaqFailureKind.communicationFailed,
      cause: Exception('offline'),
      stackTrace: StackTrace.current,
    );

    await controller.load();
    expect(controller.state.errorMessage, contains('Não foi possível conectar'));

    await controller.refresh();
    expect(controller.state.errorMessage, isNull);
    expect(controller.state.entries, hasLength(2));

    gateway.nextError = FaqException(
      kind: FaqFailureKind.forbidden,
      cause: Exception('forbidden'),
      stackTrace: StackTrace.current,
    );
    final ok = await controller.createEntry(
      FaqEntryDraft(question: 'Pergunta?', answer: 'Resposta.'),
    );
    expect(ok, isFalse);
    expect(controller.state.mutationErrorMessage, contains('permissão de gestão'));
  });
}
