/// Dados de entrada e limites de validação para criação de demandas.
library;

import 'demand_forum.dart';

/// Limites alinhados ao contrato OpenAPI do módulo de demandas.
const int maxDemandTitleLength = 200;
const int maxDemandDescriptionLength = 2000;
const int maxDemandCategoryLength = 100;
const int maxDemandCourseNameLength = 200;
const int maxDemandOpinionLength = 1000;
const int maxDemandCommentLength = 2000;

/// Dados normalizados para registrar uma nova demanda no Centro Acadêmico.
final class CreateDemandInput {
  factory CreateDemandInput({
    required DemandForum forum,
    required String description,
    String? title,
    String? category,
    String? courseName,
  }) {
    final trimmedDescription = description.trim();
    if (trimmedDescription.isEmpty) {
      throw const FormatException('A descrição da demanda é obrigatória.');
    }
    if (trimmedDescription.length > maxDemandDescriptionLength) {
      throw const FormatException(
        'A descrição excede o limite de $maxDemandDescriptionLength caracteres.',
      );
    }

    final trimmedTitle = _normalizeOptional(title);
    if (trimmedTitle != null && trimmedTitle.length > maxDemandTitleLength) {
      throw const FormatException(
        'O título excede o limite de $maxDemandTitleLength caracteres.',
      );
    }

    final trimmedCategory = _normalizeOptional(category);
    if (trimmedCategory != null &&
        trimmedCategory.length > maxDemandCategoryLength) {
      throw const FormatException(
        'A categoria excede o limite de $maxDemandCategoryLength caracteres.',
      );
    }

    final trimmedCourse = _normalizeOptional(courseName);
    if (forum == DemandForum.physicalSpace && trimmedCourse != null) {
      throw const FormatException(
        'Demandas de espaço físico não aceitam curso associado.',
      );
    }
    if (forum == DemandForum.institutional && trimmedCourse == null) {
      throw const FormatException(
        'Demandas institucionais exigem um curso associado.',
      );
    }
    if (trimmedCourse != null &&
        trimmedCourse.length > maxDemandCourseNameLength) {
      throw const FormatException(
        'O nome do curso excede o limite de $maxDemandCourseNameLength caracteres.',
      );
    }

    return CreateDemandInput._(
      forum: forum,
      description: trimmedDescription,
      title: trimmedTitle,
      category: trimmedCategory,
      courseName: trimmedCourse,
    );
  }

  const CreateDemandInput._({
    required this.forum,
    required this.description,
    this.title,
    this.category,
    this.courseName,
  });

  final DemandForum forum;
  final String description;
  final String? title;
  final String? category;
  final String? courseName;

  static String? _normalizeOptional(String? raw) {
    if (raw == null) return null;
    final trimmed = raw.trim();
    return trimmed.isEmpty ? null : trimmed;
  }
}
