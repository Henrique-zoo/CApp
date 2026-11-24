/// Estado imutável da tela de consulta e gestão de FAQ.
library;

import 'package:uuid/uuid.dart';

import '../domain/entities/faq_entry.dart';

const Object _sentinel = Object();

/// Fotografia do estado de consulta, busca e edição do FAQ do CA ativo.
final class FaqState {
  /// Cria o estado inicial ou atualizado da apresentação de FAQ.
  const FaqState({
    this.academicCenterId,
    this.academicCenterName,
    this.canManage = false,
    this.entries = const <FaqEntry>[],
    this.categories = const <String>[],
    this.selectedCategory,
    this.searchQuery = '',
    this.expandedEntryIds = const <UuidValue>{},
    this.isLoading = false,
    this.isSubmitting = false,
    this.hasLoadedOnce = false,
    this.errorMessage,
    this.mutationErrorMessage,
  });

  /// Centro Acadêmico ativo cuja base de FAQ está sendo consultada.
  final UuidValue? academicCenterId;

  /// Nome exibível do Centro Acadêmico ativo, quando conhecido.
  final String? academicCenterName;

  /// Indica se o usuário possui permissão de gestão para cadastrar/editar/remover.
  final bool canManage;

  /// Entradas de FAQ retornadas para o CA e filtros ativos.
  final List<FaqEntry> entries;

  /// Categorias temáticas disponíveis para filtro no CA ativo.
  final List<String> categories;

  /// Categoria selecionada nos chips (`null` representa "Todas").
  final String? selectedCategory;

  /// Termo atual de busca textual.
  final String searchQuery;

  /// Identificadores das perguntas atualmente expandidas na lista.
  final Set<UuidValue> expandedEntryIds;

  /// Indica carregamento de listagem, busca ou atualização.
  final bool isLoading;

  /// Indica envio de criação, edição ou remoção pela gestão.
  final bool isSubmitting;

  /// Indica se ao menos uma consulta para o CA atual já foi concluída.
  final bool hasLoadedOnce;

  /// Mensagem pública de erro da consulta/carregamento, se houver.
  final String? errorMessage;

  /// Mensagem pública de erro em operações de gestão (criar/editar/remover).
  final String? mutationErrorMessage;

  /// Indica se existe algum filtro de categoria ou termo textual ativo.
  bool get hasActiveFilters =>
      selectedCategory != null || searchQuery.trim().isNotEmpty;

  /// Indica se há Centro Acadêmico configurado para consulta.
  bool get hasAcademicCenter => academicCenterId != null;

  /// Produz uma nova fotografia preservando campos não alterados.
  FaqState copyWith({
    Object? academicCenterId = _sentinel,
    Object? academicCenterName = _sentinel,
    bool? canManage,
    List<FaqEntry>? entries,
    List<String>? categories,
    Object? selectedCategory = _sentinel,
    String? searchQuery,
    Set<UuidValue>? expandedEntryIds,
    bool? isLoading,
    bool? isSubmitting,
    bool? hasLoadedOnce,
    Object? errorMessage = _sentinel,
    Object? mutationErrorMessage = _sentinel,
  }) {
    return FaqState(
      academicCenterId: identical(academicCenterId, _sentinel)
          ? this.academicCenterId
          : academicCenterId as UuidValue?,
      academicCenterName: identical(academicCenterName, _sentinel)
          ? this.academicCenterName
          : academicCenterName as String?,
      canManage: canManage ?? this.canManage,
      entries: entries ?? this.entries,
      categories: categories ?? this.categories,
      selectedCategory: identical(selectedCategory, _sentinel)
          ? this.selectedCategory
          : selectedCategory as String?,
      searchQuery: searchQuery ?? this.searchQuery,
      expandedEntryIds: expandedEntryIds ?? this.expandedEntryIds,
      isLoading: isLoading ?? this.isLoading,
      isSubmitting: isSubmitting ?? this.isSubmitting,
      hasLoadedOnce: hasLoadedOnce ?? this.hasLoadedOnce,
      errorMessage: identical(errorMessage, _sentinel)
          ? this.errorMessage
          : errorMessage as String?,
      mutationErrorMessage: identical(mutationErrorMessage, _sentinel)
          ? this.mutationErrorMessage
          : mutationErrorMessage as String?,
    );
  }
}
