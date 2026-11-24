/// Coordenação dos casos de uso de consulta, busca, filtragem e gestão de FAQ.
///
/// Permanece independente de widgets Flutter, coordenando o ciclo de vida das
/// requisições por revisão para descartar respostas obsoletas em trocas de CA,
/// filtros ou busca digitada rapidamente.
library;

import 'dart:async';

import 'package:uuid/uuid.dart';

import '../domain/entities/faq_entry.dart';
import '../domain/errors/faq_exception.dart';
import 'faq_state.dart';
import 'ports/faq_gateway.dart';

/// Controlador reativo de estado da consulta e gestão do FAQ por CA ativo.
final class FaqController {
  /// Cria o controlador vinculado ao [gateway] e ao contexto inicial opcional.
  ///
  /// Construir não inicia chamadas externas; chame [load] ao montar a tela.
  FaqController({
    required this.gateway,
    UuidValue? academicCenterId,
    String? academicCenterName,
    bool canManage = false,
    this.searchDebounce = const Duration(milliseconds: 300),
  }) : _state = FaqState(
         academicCenterId: academicCenterId,
         academicCenterName: academicCenterName,
         canManage: canManage,
       );

  /// Porta de comunicação com o backend de FAQ.
  final FaqGateway gateway;

  /// Atraso aplicado à busca textual digitada antes de consultar o backend.
  final Duration searchDebounce;

  final StreamController<FaqState> _changes =
      StreamController<FaqState>.broadcast();

  FaqState _state;
  Timer? _debounceTimer;
  int _revision = 0;
  bool _disposed = false;

  /// Fotografia atual do estado.
  FaqState get state => _state;

  /// Transições posteriores de estado para observação pela interface.
  Stream<FaqState> get states => _changes.stream;

  void _emit(FaqState next) {
    if (_disposed) return;
    _state = next;
    _changes.add(next);
  }

  /// Carrega categorias e entradas iniciais para o Centro Acadêmico ativo.
  ///
  /// Se nenhum CA estiver configurado, permanece em estado ocioso sem chamadas.
  Future<void> load() async {
    if (_disposed) return;
    final caId = _state.academicCenterId;
    if (caId == null) {
      _emit(
        _state.copyWith(
          entries: const <FaqEntry>[],
          categories: const <String>[],
          isLoading: false,
          hasLoadedOnce: false,
          errorMessage: null,
        ),
      );
      return;
    }

    final revision = ++_revision;
    _emit(_state.copyWith(isLoading: true, errorMessage: null));

    try {
      final categoriesFuture = _loadCategoriesSafely(caId);
      final entriesFuture = gateway.listEntries(
        academicCenterId: caId,
        category: _state.selectedCategory,
        query: _normalizedQuery(_state.searchQuery),
      );

      final entries = await entriesFuture;
      final loadedCategories = await categoriesFuture;

      if (_disposed || revision != _revision) return;

      final categories = loadedCategories ?? _deriveCategories(entries);
      _emit(
        _state.copyWith(
          entries: List<FaqEntry>.unmodifiable(entries),
          categories: List<String>.unmodifiable(categories),
          isLoading: false,
          hasLoadedOnce: true,
          errorMessage: null,
        ),
      );
    } on FaqException catch (error) {
      if (_disposed || revision != _revision) return;
      _emit(
        _state.copyWith(
          isLoading: false,
          errorMessage: faqFailureMessage(error),
        ),
      );
    } on Exception {
      if (_disposed || revision != _revision) return;
      _emit(
        _state.copyWith(
          isLoading: false,
          errorMessage:
              'Ocorreu um erro inesperado ao consultar o FAQ. Tente novamente.',
        ),
      );
    }
  }

  /// Recarrega categorias e entradas preservando filtros atuais.
  Future<void> refresh() => load();

  /// Atualiza o termo de busca e agenda a consulta com debounce.
  void setSearchQuery(String query) {
    if (_disposed) return;
    _debounceTimer?.cancel();
    _emit(_state.copyWith(searchQuery: query, errorMessage: null));

    if (_state.academicCenterId == null) return;

    if (searchDebounce == Duration.zero) {
      unawaited(_fetchEntries());
    } else {
      _debounceTimer = Timer(searchDebounce, () {
        unawaited(_fetchEntries());
      });
    }
  }

  /// Executa imediatamente a busca com o termo informado.
  Future<void> searchImmediately(String query) {
    if (_disposed) return Future<void>.value();
    _debounceTimer?.cancel();
    _emit(_state.copyWith(searchQuery: query, errorMessage: null));
    return _fetchEntries();
  }

  /// Limpa o termo de busca e recarrega a lista com a categoria ativa.
  Future<void> clearSearch() => searchImmediately('');

  /// Seleciona uma categoria (`null` seleciona "Todas") e consulta o backend.
  Future<void> selectCategory(String? category) {
    if (_disposed) return Future<void>.value();
    _debounceTimer?.cancel();
    if (_state.selectedCategory == category && _state.hasLoadedOnce) {
      return Future<void>.value();
    }
    _emit(_state.copyWith(selectedCategory: category, errorMessage: null));
    return _fetchEntries();
  }

  /// Remove busca textual e categoria selecionada, recarregando todo o FAQ do CA.
  Future<void> clearFilters() {
    if (_disposed) return Future<void>.value();
    _debounceTimer?.cancel();
    _emit(
      _state.copyWith(
        selectedCategory: null,
        searchQuery: '',
        errorMessage: null,
      ),
    );
    return _fetchEntries();
  }

  /// Alterna a expansão inline da resposta para [entryId].
  void toggleExpanded(UuidValue entryId) {
    if (_disposed) return;
    final next = Set<UuidValue>.from(_state.expandedEntryIds);
    if (!next.add(entryId)) {
      next.remove(entryId);
    }
    _emit(_state.copyWith(expandedEntryIds: Set<UuidValue>.unmodifiable(next)));
  }

  /// Atualiza o CA ativo, descartando resultados/filtros do CA anterior.
  Future<void> setAcademicCenter(
    UuidValue? academicCenterId, {
    String? academicCenterName,
    bool? canManage,
  }) {
    if (_disposed) return Future<void>.value();
    _debounceTimer?.cancel();
    ++_revision;

    _emit(
      FaqState(
        academicCenterId: academicCenterId,
        academicCenterName: academicCenterName,
        canManage: canManage ?? _state.canManage,
      ),
    );

    if (academicCenterId == null) return Future<void>.value();
    return load();
  }

  /// Cadastra uma nova pergunta frequente e atualiza a listagem e categorias.
  Future<bool> createEntry(FaqEntryDraft draft) async {
    final caId = _state.academicCenterId;
    if (_disposed || caId == null || _state.isSubmitting) return false;

    _emit(_state.copyWith(isSubmitting: true, mutationErrorMessage: null));

    try {
      await gateway.createEntry(academicCenterId: caId, draft: draft);
      if (_disposed) return false;
      _emit(_state.copyWith(isSubmitting: false));
      await load();
      return true;
    } on FaqException catch (error) {
      if (_disposed) return false;
      _emit(
        _state.copyWith(
          isSubmitting: false,
          mutationErrorMessage: faqFailureMessage(error),
        ),
      );
      return false;
    } on Exception {
      if (_disposed) return false;
      _emit(
        _state.copyWith(
          isSubmitting: false,
          mutationErrorMessage:
              'Não foi possível salvar a pergunta frequente. Tente novamente.',
        ),
      );
      return false;
    }
  }

  /// Atualiza uma pergunta frequente existente e atualiza a listagem e categorias.
  Future<bool> updateEntry(UuidValue entryId, FaqEntryDraft draft) async {
    final caId = _state.academicCenterId;
    if (_disposed || caId == null || _state.isSubmitting) return false;

    _emit(_state.copyWith(isSubmitting: true, mutationErrorMessage: null));

    try {
      await gateway.updateEntry(
        academicCenterId: caId,
        entryId: entryId,
        draft: draft,
      );
      if (_disposed) return false;
      _emit(_state.copyWith(isSubmitting: false));
      await load();
      return true;
    } on FaqException catch (error) {
      if (_disposed) return false;
      _emit(
        _state.copyWith(
          isSubmitting: false,
          mutationErrorMessage: faqFailureMessage(error),
        ),
      );
      return false;
    } on Exception {
      if (_disposed) return false;
      _emit(
        _state.copyWith(
          isSubmitting: false,
          mutationErrorMessage:
              'Não foi possível atualizar a pergunta frequente. Tente novamente.',
        ),
      );
      return false;
    }
  }

  /// Remove uma pergunta frequente existente e atualiza a listagem e categorias.
  Future<bool> deleteEntry(UuidValue entryId) async {
    final caId = _state.academicCenterId;
    if (_disposed || caId == null || _state.isSubmitting) return false;

    _emit(_state.copyWith(isSubmitting: true, mutationErrorMessage: null));

    try {
      await gateway.deleteEntry(academicCenterId: caId, entryId: entryId);
      if (_disposed) return false;
      final remainingExpanded = Set<UuidValue>.from(_state.expandedEntryIds)
        ..remove(entryId);
      _emit(
        _state.copyWith(
          isSubmitting: false,
          expandedEntryIds: Set<UuidValue>.unmodifiable(remainingExpanded),
        ),
      );
      await load();
      return true;
    } on FaqException catch (error) {
      if (_disposed) return false;
      _emit(
        _state.copyWith(
          isSubmitting: false,
          mutationErrorMessage: faqFailureMessage(error),
        ),
      );
      return false;
    } on Exception {
      if (_disposed) return false;
      _emit(
        _state.copyWith(
          isSubmitting: false,
          mutationErrorMessage:
              'Não foi possível remover a pergunta frequente. Tente novamente.',
        ),
      );
      return false;
    }
  }

  /// Limpa a mensagem transitória de erro de cadastro/edição/remoção.
  void clearMutationError() {
    if (_disposed || _state.mutationErrorMessage == null) return;
    _emit(_state.copyWith(mutationErrorMessage: null));
  }

  Future<void> _fetchEntries() async {
    if (_disposed) return;
    final caId = _state.academicCenterId;
    if (caId == null) return;

    final revision = ++_revision;
    _emit(_state.copyWith(isLoading: true, errorMessage: null));

    try {
      final entries = await gateway.listEntries(
        academicCenterId: caId,
        category: _state.selectedCategory,
        query: _normalizedQuery(_state.searchQuery),
      );

      if (_disposed || revision != _revision) return;

      _emit(
        _state.copyWith(
          entries: List<FaqEntry>.unmodifiable(entries),
          isLoading: false,
          hasLoadedOnce: true,
          errorMessage: null,
        ),
      );
    } on FaqException catch (error) {
      if (_disposed || revision != _revision) return;
      _emit(
        _state.copyWith(
          isLoading: false,
          errorMessage: faqFailureMessage(error),
        ),
      );
    } on Exception {
      if (_disposed || revision != _revision) return;
      _emit(
        _state.copyWith(
          isLoading: false,
          errorMessage:
              'Ocorreu um erro inesperado ao consultar o FAQ. Tente novamente.',
        ),
      );
    }
  }

  Future<List<String>?> _loadCategoriesSafely(UuidValue caId) async {
    try {
      return await gateway.listCategories(academicCenterId: caId);
    } on Exception {
      return null;
    }
  }

  List<String> _deriveCategories(List<FaqEntry> entries) {
    final unique = <String>{
      for (final entry in entries)
        if (entry.category != null && entry.category!.trim().isNotEmpty)
          entry.category!.trim(),
    }.toList()..sort((a, b) => a.toLowerCase().compareTo(b.toLowerCase()));
    return unique;
  }

  String? _normalizedQuery(String raw) {
    final trimmed = raw.trim();
    return trimmed.isEmpty ? null : trimmed;
  }

  /// Cancela temporizadores e fecha o stream de notificações.
  Future<void> dispose() async {
    if (_disposed) return;
    _disposed = true;
    _debounceTimer?.cancel();
    await _changes.close();
  }
}
