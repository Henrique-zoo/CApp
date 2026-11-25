/// Utilitários de formatação de datas e identificação de autores para demandas.
library;

import 'package:uuid/uuid.dart';

/// Formata [date] no horário local como `dd/MM/yyyy` ou `dd/MM/yyyy às HH:mm`.
String formatDemandDate(DateTime date, {bool includeTime = false}) {
  final local = date.toLocal();
  final day = local.day.toString().padLeft(2, '0');
  final month = local.month.toString().padLeft(2, '0');
  final base = '$day/$month/${local.year}';
  if (!includeTime) return base;
  final hour = local.hour.toString().padLeft(2, '0');
  final minute = local.minute.toString().padLeft(2, '0');
  return '$base às $hour:$minute';
}

/// Rótulo público do autor sem expor identificadores técnicos longos.
String formatDemandAuthor(
  UuidValue authorId,
  UuidValue? currentUserId, {
  String fallbackRole = 'Estudante',
}) {
  if (currentUserId != null && authorId == currentUserId) {
    return 'Você';
  }
  final compact = authorId.uuid.replaceAll('-', '');
  final shortCode = compact.length >= 6 ? compact.substring(0, 6) : compact;
  return '$fallbackRole #$shortCode';
}
