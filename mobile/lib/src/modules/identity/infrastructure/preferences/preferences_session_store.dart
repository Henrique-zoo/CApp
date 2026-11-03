/// Persistência local do perfil institucional por meio de preferências.
///
/// Esta biblioteca implementa SessionStore sem levar SharedPreferencesAsync à
/// aplicação. O objetivo é reutilizar uma verificação entre execuções, depois que
/// o SDK confirmar a conta autenticada, mantendo o formato persistente independente
/// do protocolo HTTP.
///
/// ## Caminho da leitura
///
/// O adaptador consulta apenas a chave `capp.identity.session.<UID>`.
/// Usa getAll com allowList para distinguir chave ausente de um valor de tipo
/// incompatível antes de entregar uma string ao SessionCacheCodec.
///
/// Ausência produz SessionCacheMissing, tipo incompatível produz rejeição e
/// exceções de acesso produzem SessionCacheReadFailed com causa e pilha.
/// A conversão pelo codec ocorre fora dessa captura, evitando classificar erros
/// de programação como falhas do armazenamento.
///
/// ## Gravação e isolamento
///
/// save codifica versão, UID e verificação no mesmo envelope e substitui o registro
/// da conta. remove atua somente sobre sua chave. Falhas dessas operações propagam;
/// uma gravação local não constitui transação com a verificação no backend.
///
/// Chamadas concorrentes não são serializadas pelo adaptador. No fluxo de sessão,
/// o controlador organiza o acesso ao armazenamento.
///
/// ## Divisão de responsabilidades
///
/// O codec interpreta o conteúdo, e o controlador avalia proprietário vigente,
/// prazo e possibilidade de entrada. Esta biblioteca não consulta Firebase,
/// renova evidência, remove dados rejeitados automaticamente ou armazena tokens.
/// Uma cópia persistente não autentica a conta que a lê.
library;

import 'package:shared_preferences/shared_preferences.dart';

import 'package:capp/src/modules/identity/application/models/institutional_verification.dart';
import 'package:capp/src/modules/identity/application/ports/session_store.dart';
import 'package:capp/src/modules/identity/application/session_cache_result.dart';

import 'session_cache_codec.dart';

/// Implementa o armazenamento institucional em preferências persistentes.
///
/// Uma gravação substitui o envelope completo de uma conta.
/// Chamadas concorrentes não são serializadas por este adaptador.
///
/// O cache pode sobreviver ao encerramento do processo, mas não constitui
/// autenticação nem uma transação com o backend.
final class PreferencesSessionStore implements SessionStore {
  /// Recebe o acesso às preferências e o codec do formato persistente.
  ///
  /// A construção não lê, grava ou remove registros.
  /// A inicialização dos plugins pertence ao chamador.
  const PreferencesSessionStore(
    this._preferences, {
    this._codec = const SessionCacheCodec(),
  });

  final SharedPreferencesAsync _preferences;
  final SessionCacheCodec _codec;

  /// Prefixo preservado para compatibilidade com os registros existentes.
  static const String _keyPrefix = 'capp.identity.session.';

  /// Recupera o registro associado a [firebaseUid].
  ///
  /// Ausência, tipo incompatível, conteúdo rejeitado e falha de acesso
  /// permanecem resultados distintos.
  ///
  /// Usa getAll para inspecionar o tipo armazenado antes de interpretá-lo
  /// como string. Assim, um valor de outro tipo pode ser classificado
  /// explicitamente como conteúdo rejeitado.
  ///
  /// Exceções de acesso preservam causa e pilha em SessionCacheReadFailed.
  /// A interpretação pelo codec ocorre fora dessa captura, para não
  /// classificar erros de programação como falhas de acesso.
  @override
  Future<SessionCacheLoadResult> load(String firebaseUid) async {
    final key = _keyFor(firebaseUid);
    final Map<String, Object?> values;

    try {
      values = await _preferences.getAll(allowList: {key});
    } on Exception catch (error, stackTrace) {
      return SessionCacheReadFailed(cause: error, stackTrace: stackTrace);
    }

    if (!values.containsKey(key)) {
      return const SessionCacheMissing();
    }

    final encoded = values[key];

    if (encoded is! String) {
      return const SessionCacheRejected(
        SessionCacheRejectionReason.unexpectedValueType,
      );
    }

    return _codec.decode(encoded, expectedFirebaseUid: firebaseUid);
  }

  /// Substitui o registro de [firebaseUid] pela [verification] recebida.
  ///
  /// O chamador deve garantir que a verificação pertence à conta indicada.
  /// UID, versão e conteúdo são codificados no mesmo envelope.
  ///
  /// Falhas de serialização ou gravação propagam ao chamador.
  /// Não avalia expiração nem altera a autenticação.
  @override
  Future<void> save({
    required String firebaseUid,
    required InstitutionalVerification verification,
  }) async {
    final encoded = _codec.encode(
      firebaseUid: firebaseUid,
      verification: verification,
    );

    await _preferences.setString(_keyFor(firebaseUid), encoded);
  }

  /// Remove somente o registro associado a [firebaseUid].
  ///
  /// A remoção de chave ausente segue o comportamento idempotente do plugin.
  /// Falhas propagam. Não executa logout ou remove evidências do backend.
  @override
  Future<void> remove(String firebaseUid) {
    return _preferences.remove(_keyFor(firebaseUid));
  }

  /// Constrói a chave da conta sem normalizar ou substituir seu identificador.
  String _keyFor(String firebaseUid) {
    return '$_keyPrefix$firebaseUid';
  }
}
