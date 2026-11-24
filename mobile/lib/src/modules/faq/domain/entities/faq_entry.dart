/// Entidades e regras de validação de domínio do módulo de FAQ.
///
/// Espelha os limites estruturais e invariantes do domínio no backend
/// (`MAX_QUESTION_LENGTH = 500`, `MAX_ANSWER_LENGTH = 5000`,
/// `MAX_CATEGORY_LENGTH = 100`), sem depender de transporte HTTP ou Flutter.
library;

import 'package:uuid/uuid.dart';

/// Comprimento máximo em caracteres do texto de uma pergunta frequente.
const int faqMaxQuestionLength = 500;

/// Comprimento máximo em caracteres da resposta de uma pergunta frequente.
const int faqMaxAnswerLength = 5000;

/// Comprimento máximo em caracteres da categoria temática opcional.
const int faqMaxCategoryLength = 100;

/// Valida o texto da pergunta frequente para formulários e rascunhos.
String? validateFaqQuestion(String? value) {
  final trimmed = value?.trim() ?? '';
  if (trimmed.isEmpty) {
    return 'Informe a pergunta frequente.';
  }
  if (trimmed.runes.length > faqMaxQuestionLength) {
    return 'A pergunta deve ter no máximo $faqMaxQuestionLength caracteres.';
  }
  return null;
}

/// Valida a resposta oficial do Centro Acadêmico para formulários e rascunhos.
String? validateFaqAnswer(String? value) {
  final trimmed = value?.trim() ?? '';
  if (trimmed.isEmpty) {
    return 'Informe a resposta da pergunta frequente.';
  }
  if (trimmed.runes.length > faqMaxAnswerLength) {
    return 'A resposta deve ter no máximo $faqMaxAnswerLength caracteres.';
  }
  return null;
}

/// Valida a categoria temática opcional (em branco significa sem categoria).
String? validateFaqCategory(String? value) {
  final trimmed = value?.trim() ?? '';
  if (trimmed.isEmpty) return null;
  if (trimmed.runes.length > faqMaxCategoryLength) {
    return 'A categoria deve ter no máximo $faqMaxCategoryLength caracteres.';
  }
  return null;
}

/// Pergunta e resposta frequente pertencente a um Centro Acadêmico.
final class FaqEntry {
  /// Cria uma entrada de FAQ com timestamps normalizados em UTC.
  FaqEntry({
    required this.id,
    required this.academicCenterId,
    required this.question,
    required this.answer,
    required this.category,
    required this.createdBy,
    required this.updatedBy,
    required DateTime createdAt,
    required DateTime updatedAt,
  }) : createdAt = createdAt.toUtc(),
       updatedAt = updatedAt.toUtc();

  /// Identificador único estável da entrada de FAQ.
  final UuidValue id;

  /// Identificador do Centro Acadêmico responsável pelo conteúdo.
  final UuidValue academicCenterId;

  /// Texto da pergunta frequente.
  final String question;

  /// Resposta oficial do Centro Acadêmico.
  final String answer;

  /// Categoria temática opcional.
  final String? category;

  /// Identificador do usuário que cadastrou a pergunta.
  final UuidValue createdBy;

  /// Identificador do usuário que realizou a última alteração.
  final UuidValue updatedBy;

  /// Instante UTC de criação.
  final DateTime createdAt;

  /// Instante UTC da última atualização.
  final DateTime updatedAt;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FaqEntry &&
          other.id == id &&
          other.academicCenterId == academicCenterId &&
          other.question == question &&
          other.answer == answer &&
          other.category == category &&
          other.createdBy == createdBy &&
          other.updatedBy == updatedBy &&
          other.createdAt == createdAt &&
          other.updatedAt == updatedAt;

  @override
  int get hashCode => Object.hash(
    id,
    academicCenterId,
    question,
    answer,
    category,
    createdBy,
    updatedBy,
    createdAt,
    updatedAt,
  );
}

/// Dados validados e normalizados para criação ou edição de uma entrada de FAQ.
final class FaqEntryDraft {
  /// Constrói o rascunho removendo espaços periféricos e validando limites.
  ///
  /// Lança [FormatException] se a pergunta, resposta ou categoria violarem
  /// as regras estruturais do domínio.
  factory FaqEntryDraft({
    required String question,
    required String answer,
    String? category,
  }) {
    final trimmedQuestion = question.trim();
    final trimmedAnswer = answer.trim();
    final trimmedCategory = category?.trim();
    final normalizedCategory =
        trimmedCategory == null || trimmedCategory.isEmpty
        ? null
        : trimmedCategory;

    final questionError = validateFaqQuestion(trimmedQuestion);
    if (questionError != null) {
      throw FormatException(questionError);
    }

    final answerError = validateFaqAnswer(trimmedAnswer);
    if (answerError != null) {
      throw FormatException(answerError);
    }

    final categoryError = validateFaqCategory(normalizedCategory);
    if (categoryError != null) {
      throw FormatException(categoryError);
    }

    return FaqEntryDraft._(
      question: trimmedQuestion,
      answer: trimmedAnswer,
      category: normalizedCategory,
    );
  }

  const FaqEntryDraft._({
    required this.question,
    required this.answer,
    required this.category,
  });

  /// Pergunta normalizada.
  final String question;

  /// Resposta normalizada.
  final String answer;

  /// Categoria temática normalizada ou `null` quando omitida.
  final String? category;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FaqEntryDraft &&
          other.question == question &&
          other.answer == answer &&
          other.category == category;

  @override
  int get hashCode => Object.hash(question, answer, category);
}
