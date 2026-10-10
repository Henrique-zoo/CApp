import 'package:flutter_test/flutter_test.dart';

typedef StepAction = Future<void> Function(
  WidgetTester tester,
  RegExpMatch match,
);

class StepDefinition {
  final RegExp pattern;
  final StepAction action;

  const StepDefinition(this.pattern, this.action);
}

StepDefinition pendingStep(String pattern) {
  return StepDefinition(RegExp(pattern), (tester, match) async {
    throw UnimplementedError('Step não implementado: ${match.group(0)}');
  });
}

Future<void> executeStep({
  required WidgetTester tester,
  required String text,
  required Iterable<StepDefinition> stepDefinitions,
}) async {
  final matches = <(StepDefinition, RegExpMatch)>[];

  for (final definition in stepDefinitions) {
    final match = definition.pattern.firstMatch(text);

    if (match != null) {
      matches.add((definition, match));
    }
  }

  if (matches.isEmpty) {
    throw StateError('Step definition não encontrada: "$text"');
  }

  if (matches.length > 1) {
    throw StateError(
      'Step definition ambígua: "$text"'
      '(${matches.length} correspondências)',
    );
  }

  final (definition, match) = matches.single;

  await definition.action(tester, match);
}
