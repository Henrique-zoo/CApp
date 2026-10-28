/// Regressões das fronteiras de importação do módulo e da composição.
///
/// Impede reintroduzir SDKs e transporte na aplicação e na apresentação.
library;

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

/// Extrai URIs de diretivas, inclusive quando divididas em várias linhas.
Iterable<String> _directives(File source) sync* {
  final pattern = RegExp(
    r'''^\s*(?:import|export)\s+['"]([^'"]+)['"]''',
    multiLine: true,
  );
  for (final match in pattern.allMatches(source.readAsStringSync())) {
    yield match.group(1)!;
  }
}

/// Resolve bibliotecas próprias independentemente dos separadores do SO.
String _target(File source, String uri) {
  if (uri.startsWith('package:capp/')) {
    return File('lib/${uri.substring('package:capp/'.length)}').absolute.uri
        .toString();
  }
  return source.absolute.uri.resolve(uri).toString();
}

/// Lista os arquivos autorais de uma área.
Iterable<File> _sources(String root) => Directory(root)
    .listSync(recursive: true)
    .whereType<File>()
    .where(
      (file) => file.path.endsWith('.dart') && !file.path.endsWith('.g.dart'),
    );

/// Registra os limites fiscalizáveis pelas diretivas das bibliotecas.
void main() {
  test('aplicação depende somente de Dart e da própria aplicação', () {
    final root = Directory('lib/src/modules/identity/application').absolute.uri
        .toString();
    for (final source in _sources('lib/src/modules/identity/application')) {
      for (final uri in _directives(source)) {
        if (uri.startsWith('dart:')) continue;
        expect(
          _target(source, uri).startsWith(root),
          isTrue,
          reason: '${source.path} depende de $uri',
        );
      }
    }
  });

  test('apresentação não conhece transporte, Firebase ou preferências', () {
    final identity = Directory('lib/src/modules/identity').absolute.uri
        .toString();
    final sharedPresentation = File('lib/src/presentation/presentation.dart')
        .absolute
        .uri
        .toString();
    for (final source in _sources('lib/src/modules/identity/presentation')) {
      for (final uri in _directives(source)) {
        if (uri.startsWith('dart:') ||
            uri.startsWith('package:flutter/') ||
            uri.startsWith('package:flutter_svg/')) {
          continue;
        }
        final target = _target(source, uri);
        if (target == sharedPresentation) continue;
        expect(
          target.startsWith(identity),
          isTrue,
          reason: '${source.path} depende de $uri',
        );
        expect(
          target.contains('/infrastructure/'),
          isFalse,
          reason: '${source.path} acessa uma implementação',
        );
      }
    }
  });

  test(
    'apresentação compartilhada depende somente de Flutter e de si própria',
    () {
      final root = Directory('lib/src/presentation').absolute.uri.toString();
      for (final source in _sources('lib/src/presentation')) {
        for (final uri in _directives(source)) {
          if (uri.startsWith('dart:') || uri.startsWith('package:flutter/')) {
            continue;
          }
          expect(
            _target(source, uri).startsWith(root),
            isTrue,
            reason: '${source.path} depende de $uri',
          );
        }
      }
    },
  );

  test('transporte compartilhado não conhece módulos de negócio', () {
    for (final source in _sources('lib/src/infrastructure/backend')) {
      for (final uri in _directives(source)) {
        expect(
          _target(source, uri).contains('/modules/'),
          isFalse,
          reason: '${source.path} depende de $uri',
        );
      }
    }
  });

  test('as APIs públicas exportam apenas suas camadas correspondentes', () {
    for (final api in [
      ('lib/src/modules/identity/identity.dart', 'application/'),
      ('lib/src/modules/identity/identity_widgets.dart', 'presentation/'),
    ]) {
      for (final uri in _directives(File(api.$1))) {
        expect(uri.startsWith(api.$2), isTrue, reason: '${api.$1}: $uri');
      }
    }
  });
}
