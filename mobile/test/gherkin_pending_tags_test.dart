import 'package:cucumber_gherkin/cucumber_gherkin.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../integration_test/support/scenario_filter.dart';

const pendingFeatures = {
  'feature': '''
@pending_mobile
Feature: Selection
  @mobile
  Scenario: Pending
    Given a step
''',
  'rule': '''
Feature: Selection
  @pending_mobile
  Rule: Pending
    @mobile
    Scenario: Pending
      Given a step
''',
  'scenario': '''
Feature: Selection
  @mobile @pending_mobile
  Scenario: Pending
    Given a step
''',
};

Set<String> tagsFromFeature(String source) {
  final envelopes = generateMessages(
    source,
    'selection.feature',
    const GherkinOptions(
      includeSource: false,
      includeGherkinDocument: false,
      includePickles: true,
    ),
  ).toList();

  for (final envelope in envelopes) {
    final error = envelope.parseError;
    if (error != null) {
      throw FormatException(error.message);
    }
  }

  final scenario = envelopes
      .map((envelope) => envelope.pickle)
      .where((pickle) => pickle != null)
      .single!;
  return scenario.tags.map((tag) => tag.name).toSet();
}

void main() {
  for (final entry in pendingFeatures.entries) {
    test('inherits pending_mobile from ${entry.key}', () {
      final tags = tagsFromFeature(entry.value);
      expect(tags, contains('@mobile'));
      expect(isPendingMobileScenario(tags), isTrue);
    });

    testWidgets('does not execute a mobile scenario pending on ${entry.key}', (
      _,
    ) async {
      fail('A callback de um cenário pendente não deve executar.');
    }, skip: isPendingMobileScenario(tagsFromFeature(entry.value)));
  }

  const backendPendingFeature = '''
@pending_backend
Feature: Selection
  @mobile
  Scenario: Available
    Given a step
''';
  testWidgets('executes a mobile scenario pending only on backend', (
    tester,
  ) async {
    await tester.pumpWidget(
      const Text('Executed', textDirection: TextDirection.ltr),
    );
    expect(find.text('Executed'), findsOneWidget);
  }, skip: isPendingMobileScenario(tagsFromFeature(backendPendingFeature)));
}
