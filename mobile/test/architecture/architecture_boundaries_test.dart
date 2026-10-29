/// Regressões das fronteiras de importação do módulo e da composição.
///
/// Impede reintroduzir SDKs e transporte na aplicação e na apresentação.
/// A composição visual usa fachadas; a raiz de dependências concentra as
/// implementações concretas. A infraestrutura compartilhada não depende de app.
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

  test('infraestrutura compartilhada não conhece app ou módulos', () {
    final root = Directory('lib/src/infrastructure').absolute.uri.toString();
    for (final source in _sources('lib/src/infrastructure')) {
      for (final uri in _directives(source)) {
        if (uri.startsWith('dart:') ||
            (uri.startsWith('package:') && !uri.startsWith('package:capp/'))) {
          continue;
        }
        expect(
          _target(source, uri).startsWith(root),
          isTrue,
          reason: '${source.path} depende de $uri',
        );
      }
    }
  });

  test('app usa fachadas e concentra integrações na raiz de composição', () {
    final root = Directory('lib/src/app').absolute.uri.toString();
    final composition = File('lib/src/app/app_dependencies.dart').absolute.uri
        .toString();
    final presentation = File('lib/src/presentation/presentation.dart')
        .absolute
        .uri
        .toString();
    final modules = Directory('lib/src/modules').absolute.uri.toString();

    for (final source in _sources('lib/src/app')) {
      if (source.absolute.uri.toString() == composition) continue;
      for (final uri in _directives(source)) {
        if (uri.startsWith('dart:') ||
            uri.startsWith('package:flutter/') ||
            uri.startsWith('package:go_router/') ||
            uri.startsWith('package:flutter_riverpod/')) {
          continue;
        }
        final target = _target(source, uri);
        if (target.startsWith(root) || target == presentation) continue;

        final relative = target.startsWith(modules)
            ? target.substring(modules.length).split('/')
            : <String>[];
        final publicModuleApi =
            relative.length == 2 &&
            (relative[1] == '${relative[0]}.dart' ||
                relative[1] == '${relative[0]}_widgets.dart');
        expect(
          publicModuleApi,
          isTrue,
          reason: '${source.path} depende diretamente de $uri',
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
