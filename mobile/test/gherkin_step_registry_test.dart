/// Confere o vínculo entre as especificações de produto e os steps do mobile.
///
/// A pendência suspende a execução funcional, preservando a obrigação de registrar
/// cada expressão. O teste usa o parser Gherkin e os padrões reais do runner,
/// incluindo contextos e exemplos expandidos, sem executar as ações pendentes.
library;

import 'dart:io';

import 'package:cucumber_gherkin/cucumber_gherkin.dart';
import 'package:flutter_test/flutter_test.dart';

import '../integration_test/steps/steps.dart';

void main() {
  test(
    'every mobile step has one registered definition, including pending',
    () {
      final files =
          Directory('../features')
              .listSync(recursive: true)
              .whereType<File>()
              .where((file) => file.path.endsWith('.feature'))
              .toList()
            ..sort((a, b) => a.path.compareTo(b.path));
      var checked = 0;
      final failures = <String>[];
      for (final file in files) {
        final envelopes = generateMessages(
          file.readAsStringSync(),
          file.path,
          const GherkinOptions(
            includeSource: false,
            includeGherkinDocument: false,
            includePickles: true,
          ),
        );
        for (final envelope in envelopes) {
          final error = envelope.parseError;
          if (error != null) fail('${file.path}: ${error.message}');
          final scenario = envelope.pickle;
          if (scenario == null ||
              !scenario.tags.any((tag) => tag.name == '@mobile')) {
            continue;
          }
          for (final step in scenario.steps) {
            checked += 1;
            final matches = stepDefinitions
                .where((definition) => definition.pattern.hasMatch(step.text))
                .length;
            if (matches != 1) {
              failures.add(
                '${file.path} / ${scenario.name} / ${step.text}: '
                '$matches definições',
              );
            }
          }
        }
      }
      expect(checked, greaterThan(0), reason: 'Nenhum step mobile encontrado');
      expect(failures, isEmpty, reason: failures.join('\n'));
    },
  );
}
