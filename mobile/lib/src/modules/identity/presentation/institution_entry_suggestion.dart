/// Sugestão editável de ingresso para reduzir o preenchimento do cadastro.
///
/// Conta o período atual como parte do curso: no primeiro período, a sugestão é
/// o próprio semestre corrente; no segundo, é o imediatamente anterior. Usa
/// semestres civis (janeiro–junho e julho–dezembro), sem consultar calendários
/// acadêmicos. Por isso não valida nem substitui um ingresso informado: pausas,
/// transferências e calendários institucionais podem exigir outro valor.
library;

/// Retorna AAAA.1/AAAA.2 ou nulo para um período/data sem sugestão representável.
///
/// Por exemplo, em novembro de 2026, o 4º período sugere 2025.1. Subtrair quatro
/// semestres excluiria indevidamente o período corrente; devem ser subtraídos
/// apenas os três períodos anteriores.
String? suggestInstitutionEntrySemester({
  required int currentSemester,
  required DateTime at,
}) {
  if (currentSemester < 1 || at.year < 1 || at.year > 9999) return null;

  final currentTerm = at.year * 2 + (at.month >= 7 ? 1 : 0);
  final entryTerm = currentTerm - (currentSemester - 1);

  if (entryTerm < 2) return null;

  return '${(entryTerm ~/ 2).toString().padLeft(4, '0')}.${entryTerm % 2 + 1}';
}
