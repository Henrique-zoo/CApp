/// Preferências controladas para exercitar o adaptador persistente real.
///
/// Não utiliza plugins. Falhas operacionais e erros de programação podem
/// ser injetados separadamente, sem converter seus diagnósticos.
library;

import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Implementa somente as operações usadas pelo armazenamento institucional.
///
/// Os contêineres mutáveis são intencionais para controlar as respostas.
/// Chamadas a outras operações falham por noSuchMethod.
final class FakePreferences implements SharedPreferencesAsync {
  /// Conteúdo associado às chaves, inclusive tipos incompatíveis para testes.
  final values = <String, Object?>{};

  /// Falha lançada na próxima leitura enquanto permanecer configurada.
  final readFailure = ValueNotifier<Object?>(null);

  /// Falha lançada antes de modificar uma chave na gravação.
  final writeFailure = ValueNotifier<Object?>(null);

  /// Falha lançada antes de remover uma chave.
  final removeFailure = ValueNotifier<Object?>(null);

  /// Entrega valores filtrados ou propaga a falha de leitura injetada.
  @override
  Future<Map<String, Object?>> getAll({Set<String>? allowList}) async {
    final failure = readFailure.value;
    if (failure != null) Error.throwWithStackTrace(failure, StackTrace.current);
    return Map.fromEntries(
      values.entries.where(
        (entry) => allowList == null || allowList.contains(entry.key),
      ),
    );
  }

  /// Substitui o texto somente quando a etapa de gravação não falha.
  @override
  Future<void> setString(String key, String value) async {
    final failure = writeFailure.value;
    if (failure != null) Error.throwWithStackTrace(failure, StackTrace.current);
    values[key] = value;
  }

  /// Remove a chave somente quando a etapa de remoção não falha.
  @override
  Future<void> remove(String key) async {
    final failure = removeFailure.value;
    if (failure != null) Error.throwWithStackTrace(failure, StackTrace.current);
    values.remove(key);
  }

  /// Libera os notificadores do teste sem alterar os registros observados.
  void dispose() {
    readFailure.dispose();
    writeFailure.dispose();
    removeFailure.dispose();
  }

  /// Operações não previstas pelo adaptador falham explicitamente.
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}
