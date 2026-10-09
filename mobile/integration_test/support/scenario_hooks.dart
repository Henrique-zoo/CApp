import 'package:capp/app/app.dart';
import 'package:flutter_test/flutter_test.dart';

Future<void> beforeScenario(WidgetTester tester, Set<String> tags) async {
  if (tags.contains('@mobile')) {
    expect(find.byType(CApp), findsOneWidget);
  }
}
