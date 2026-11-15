/// Cache versionado do perfil interno, isolado por UID Firebase.
///
/// A restauração online substitui esta fotografia pelo backend. Conteúdo inválido
/// é recusado; senhas e tokens nunca participam do envelope. O codec é local e
/// não depende dos DTOs HTTP, para permitir evoluções independentes.
///
/// ## Motivação e percurso
///
/// O seletor pode retomar contas completas ou incompletas. Esta fotografia mantém
/// o perfil interno disponível entre execuções, enquanto a verificação institucional
/// possui armazenamento e prazo próprios. O controlador salva uma resposta aceita
/// do backend, e [PreferencesAccountProfileStore.loadUser] reconstitui [InternalUser]
/// com as mesmas invariantes usadas pelo gateway. Uma leitura local não renova
/// vínculo ou autentica a conta.
///
/// O envelope v1 inclui versão, proprietário e valores do perfil. Complementos
/// removidos, como curso e e-mail institucional, são ignorados na leitura de dados
/// antigos e não voltam a ser gravados. A sugestão transitória de e-mail do cadastro
/// nunca integra a representação. Falhas de leitura, conteúdo e escrita propagam
/// ao controlador, que decide como apresentar o diagnóstico e preservar a sessão.
library;

import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';
import 'package:uuid/uuid.dart';

import '../../application/models/internal_user.dart';
import '../../application/ports/account_profile_store.dart';

/// Adaptador de preferências para perfis internos, com uma chave por UID Firebase.
///
/// Não possui conexões ou SDKs de autenticação. A propriedade das preferências e
/// a serialização das operações concorrentes pertencem à composição/controlador.
final class PreferencesAccountProfileStore implements AccountProfileStore {
  /// Reutiliza [preferences] sem carregar dados ou assumir seu ciclo de vida.
  const PreferencesAccountProfileStore(this.preferences);

  /// Meio persistente fornecido pela composição; não é um cache HTTP ou de tokens.
  final SharedPreferencesAsync preferences;

  /// Endereça o perfil de [uid] como valor opaco, sem interpretar a identidade.
  /// O chamador deve fornecer o UID confiável da conta dona da operação.
  String _key(String uid) => 'capp.identity.user.$uid';

  /// Lê o envelope de [firebaseUid] e reconstitui uma fotografia validada.
  ///
  /// Retorna nulo quando não há registro. Para conteúdo presente, exige versão 1,
  /// proprietário correspondente, UUID válido, estado conhecido e tipos compatíveis.
  /// Os valores opcionais podem faltar em envelopes anteriores; as exigências de
  /// uma conta ativa e os formatos acadêmicos são conferidos por [InternalUser].
  ///
  /// JSON, envelope ou perfil incompatível lançam [FormatException]. Falhas do
  /// armazenamento propagam com seu diagnóstico original. A leitura não remove
  /// registros rejeitados, regrava os antigos ou altera sua origem; a restauração
  /// online e a política de recuperação pertencem ao controlador.
  @override
  Future<InternalUser?> loadUser(String firebaseUid) async {
    final value = await preferences.getString(_key(firebaseUid));

    if (value == null) {
      return null;
    }

    final data = jsonDecode(value);

    if (data is! Map<String, dynamic> ||
        data['version'] != 1 ||
        data['uid'] != firebaseUid ||
        data['id'] is! String ||
        !InternalUserStatus.values.any((s) => s.name == data['status'])) {
      throw const FormatException('Cache de perfil incompatível');
    }

    // Extensão aditiva do envelope v1: caches anteriores não têm complementos.
    for (final key in ['nickname', 'email', 'entrySemester', 'name']) {
      if (data[key] != null && data[key] is! String) {
        throw const FormatException('Campo de perfil incompatível');
      }
    }

    final semester = data['currentSemester'];

    if (semester != null && semester is! int) {
      throw const FormatException('Período incompatível');
    }

    final term = data['entrySemester'] as String?;

    return InternalUser(
      id: UuidValue.withFormatValidation(data['id'] as String),
      status: InternalUserStatus.values.byName(data['status'] as String),
      nickname: data['nickname'] as String?,
      email: data['email'] as String?,
      currentSemester: semester as int?,
      institutionEntrySemester: term,
      name: data['name'] as String?,
    );
  }

  /// Substitui somente a fotografia de [firebaseUid] pelos valores de [user].
  ///
  /// O modelo já deve ter sido validado e associado a esse UID pela coordenação;
  /// o armazenamento não consulta a autenticação para conferir a associação.
  /// Grava um único envelope JSON v1, incluindo o endereço interno escolhido e
  /// preservando campos nulos. Não grava curso declarado, sugestão de e-mail,
  /// credenciais ou evidência institucional.
  ///
  /// O Future termina após a escrita da preferência. Falhas propagam, permitindo
  /// ao controlador distinguir uma conta aceita pelo backend de falha de cache.
  @override
  Future<void> saveUser(String firebaseUid, InternalUser user) =>
      preferences.setString(
        _key(firebaseUid),
        jsonEncode({
          'version': 1,
          'uid': firebaseUid,
          'id': user.id.toString(),
          'status': user.status.name,
          'nickname': user.nickname,
          'email': user.email,
          'currentSemester': user.currentSemester,
          'entrySemester': user.institutionEntrySemester,
          'name': user.name,
        }),
      );

  /// Remove a preferência de perfil pertencente a [firebaseUid].
  ///
  /// Não remove outras contas, seu índice, credenciais Firebase ou a verificação
  /// institucional; o fluxo de saída coordena esses armazenamentos separadamente.
  /// O Future acompanha a tentativa de remoção e propaga falhas do meio, sem
  /// anunciar que a operação "Esquecer conta" inteira já terminou.
  @override
  Future<void> removeUser(String firebaseUid) =>
      preferences.remove(_key(firebaseUid));
}
