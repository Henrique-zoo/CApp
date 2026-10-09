import 'dart:convert';
import 'dart:io';

import 'package:cucumber_gherkin/cucumber_gherkin.dart';

Future<void> main() async {
  final directory = Directory('../features');

  if (!await directory.exists()) {
    throw StateError('Diretório features/ não encontrado');
  }

  final files = await directory
      .list(recursive: true)
      .where((entity) => entity is File && entity.path.endsWith('.feature'))
      .cast<File>()
      .toList();

  files.sort((a, b) => a.path.compareTo(b.path));

  final scenarios = <Map<String, Object?>>[];

  for (final file in files) {
    final source = await file.readAsString();

    final envelopes = generateMessages(
      source,
      file.path.replaceAll('\\', '/'),
      const GherkinOptions(
        includeSource: false,
        includeGherkinDocument: false,
        includePickles: true,
      ),
    );

    for (final envelope in envelopes) {
      final error = envelope.parseError;

      if (error != null) {
        throw FormatException('${file.path}: ${error.message}');
      }

      final pickle = envelope.pickle;
      if (pickle != null) {
        scenarios.add(pickle.toJson());
      }
    }
  }

  if (scenarios.isEmpty) {
    throw StateError('Nenhum cenário Gherkin encontrado.');
  }

  final output = File('integration_test/generated/gherkin_cases.dart');

  await output.parent.create(recursive: true);

  final encoded = jsonEncode(jsonEncode(scenarios)).replaceAll(r'$', r'\$');

  await output.writeAsString('''
    // GENERATED CODE - DO NOT MODIFY BY HAND
    const String gherkinCasesJson = $encoded;
  ''');

  stdout.writeln('${scenarios.length} cenários Gherkin gerados.');
}
