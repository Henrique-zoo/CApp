/// Preserva a regra de nomes únicos antes de decodificar uma evidência.
library;

import 'dart:convert';

/// Rejeita nomes de propriedades duplicados, inclusive com escapes equivalentes.
String checkElectionJson(String text) {
  final names = <Set<String>>[];
  for (var i = 0; i < text.length; i++) {
    switch (text.codeUnitAt(i)) {
      case 0x7b: // {
        names.add(<String>{});
      case 0x7d: // }
        if (names.isEmpty) throw const FormatException('Objeto JSON inválido');
        names.removeLast();
      case 0x22: // "
        final start = i;
        i++;
        while (i < text.length && text.codeUnitAt(i) != 0x22) {
          if (text.codeUnitAt(i) == 0x5c) i++; // Pula o caractere escapado.
          i++;
        }
        if (i >= text.length) {
          throw const FormatException('String JSON incompleta');
        }
        var next = i + 1;
        while (next < text.length && text.codeUnitAt(next) <= 0x20) {
          next++;
        }
        if (next < text.length && text.codeUnitAt(next) == 0x3a) {
          final name = jsonDecode(text.substring(start, i + 1)) as String;
          if (names.isEmpty || !names.last.add(name)) {
            throw const FormatException(
              'Nome de campo inválido ou repetido na evidência',
            );
          }
        }
    }
  }
  return text;
}
