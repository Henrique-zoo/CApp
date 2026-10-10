import 'dart:convert';

import 'package:capp/app/app.dart';
import 'package:capp/bootstrap.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'generated/gherkin_cases.dart';
import 'steps/steps.dart';
import 'support/scenario_filter.dart';
import 'support/scenario_hooks.dart';
import 'support/step_registry.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(() async {
    await initializeServices();
  });

  final scenarios = (jsonDecode(gherkinCasesJson) as List)
      .cast<Map<String, dynamic>>();

  for (final (index, scenario) in scenarios.indexed) {
    final tags = (scenario['tags'] as List<dynamic>)
        .map((tag) => (tag as Map<String, dynamic>)['name'] as String)
        .toSet();

    if (!tags.contains('@mobile')) continue;

    final name = scenario['name'] as String;
    final uri = scenario['uri'] as String;
    final steps = scenario['steps'] as List;

    testWidgets('$uri: $name [${index + 1}]', (tester) async {
      await tester.pumpWidget(const ProviderScope(child: CApp()));

      await beforeScenario(tester, tags);

      for (final rawStep in steps) {
        final step = rawStep as Map<String, dynamic>;

        final text = step['text'] as String;

        if (step['argument'] != null) {
          throw UnsupportedError(
            'Data Tables e Doc Strings ainda não são '
            'suportados: "$text"',
          );
        }

        await executeStep(
          tester: tester,
          text: text,
          stepDefinitions: stepDefinitions,
        );
      }
    }, skip: isPendingMobileScenario(tags));
  }
}
