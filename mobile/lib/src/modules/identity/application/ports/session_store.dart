/// Porta de persistência da última verificação institucional de cada conta.
///
/// O controlador precisa recuperar e salvar modelos da aplicação sem conhecer
/// chaves de preferências, JSON ou DTOs HTTP. Esta biblioteca mantém esse contrato
/// separado do meio de armazenamento e de sua representação persistente.
///
/// ## Leitura e reutilização
///
/// Depois de conhecer uma conta autenticada, o controlador solicita load com
/// seu UID. O resultado distingue registro encontrado, ausente, rejeitado e
/// falha de acesso, preservando os diagnósticos disponíveis.
///
/// Uma verificação expirada continua sendo um dado encontrado. A avaliação de
/// validade pertence ao controlador, e não ao adaptador ou ao codec.
/// Encontrar um registro tampouco autentica a conta.
///
/// ## Escrita e remoção
///
/// save substitui o registro associado ao UID informado; o chamador deve garantir
/// a associação entre conta e verificação. remove atua somente sobre esse registro.
/// Falhas de escrita e remoção propagam, sem simular conclusão bem-sucedida.
///
/// O armazenamento não guarda tokens, consulta a autenticação ou realiza uma
/// transação com o backend. Formato, meio e durabilidade pertencem à implementação;
/// a composição atual utiliza PreferencesSessionStore.
library;

import '../models/institutional_verification.dart';
import '../session_cache_result.dart';

/// Armazena a última verificação institucional associada a cada conta.
///
/// O chamador deve fornecer o UID da autenticação correspondente.
/// O formato persistente e sua durabilidade pertencem à implementação.
///
/// Não armazena credenciais Microsoft ou Firebase.
abstract interface class SessionStore {
  /// Recupera a verificação associada a [firebaseUid].
  ///
  /// Distingue registro encontrado, ausência, conteúdo rejeitado e falha
  /// de acesso ao armazenamento.
  ///
  /// Registros expirados também são retornados. A avaliação da validade
  /// pertence à coordenação da sessão.
  ///
  /// A leitura não remove ou corrige registros. Erros inesperados de
  /// programação continuam propagando.
  Future<SessionCacheLoadResult> load(String firebaseUid);

  /// Substitui os dados armazenados para [firebaseUid].
  ///
  /// [verification] deve pertencer à conta indicada pelo chamador.
  /// O armazenamento não consulta a autenticação para conferir essa associação.
  ///
  /// A conclusão representa o resultado informado pela implementação local,
  /// sem prometer uma transação com o backend.
  ///
  /// Falhas de gravação propagam ao chamador.
  Future<void> save({
    required String firebaseUid,
    required InstitutionalVerification verification,
  });

  /// Remove somente o registro associado a [firebaseUid].
  ///
  /// Ausência de registro é tratada como sucesso.
  /// Falhas de remoção propagam ao chamador.
  ///
  /// Não executa logout nem remove dados do backend.
  Future<void> remove(String firebaseUid);
}
