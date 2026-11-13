/// Regressões da contagem inclusiva dos semestres na sugestão de ingresso.
/// Verifica a virada de ano e o limite entre os semestres civis com datas fixas.
library;

import 'package:capp/src/modules/identity/presentation/institution_entry_suggestion.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  for (final (at, period, expected) in [
    (DateTime(2026, 11, 12), 1, '2026.2'),
    (DateTime(2026, 11, 12), 2, '2026.1'),
    (DateTime(2026, 11, 12), 4, '2025.1'),
    (DateTime(2026, 6, 30), 1, '2026.1'),
    (DateTime(2026, 7, 1), 1, '2026.2'),
    (DateTime(2026, 1, 1), 2, '2025.2'),
    (DateTime(2026, 1, 1), 4, '2024.2'),
  ]) {
    test('$at no período $period sugere $expected', () {
      expect(
        suggestInstitutionEntrySemester(currentSemester: period, at: at),
        expected,
      );
    });
  }

  test('não sugere para períodos inválidos ou fora da representação', () {
    for (final period in [-1, 0, 999999999]) {
      expect(
        suggestInstitutionEntrySemester(
          currentSemester: period,
          at: DateTime(2026),
        ),
        isNull,
      );
    }
  });
}
