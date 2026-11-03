/// Contrato de representação persistente da verificação institucional.
///
/// O armazenamento local precisa continuar legível quando o protocolo HTTP mudar.
/// Esta biblioteca codifica modelos de aplicação diretamente, sem reutilizar DTOs
/// do backend, e separa interpretação de conteúdo do acesso às preferências.
///
/// ## Envelope e compatibilidade
///
/// A versão 1 contém schemaVersion, firebaseUid e verification. A gravação
/// preserva campos opcionais nulos e representa expiresAt em UTC.
/// Não inclui credenciais Firebase ou Microsoft.
///
/// decode lê o JSON, confere a estrutura e versão do envelope e compara seu UID
/// ao proprietário esperado. Em seguida reconstrói perfil, vínculo e validade.
/// Campos adicionais são ignorados; campos obrigatórios continuam obrigatórios
/// mesmo quando seu valor admite nulo.
///
/// ## Resultados de interpretação
///
/// Dados compatíveis produzem SessionCacheFound. JSON malformado, versão não
/// suportada, proprietário divergente e verificação incompatível produzem rejeições
/// distintas, com causa e pilha quando existe uma falha de formato.
///
/// O vínculo desconhecido não vira não estudante. A variante não estudantil
/// também rejeita courseName, inclusive nulo, conforme o formato da versão atual.
///
/// ## Fronteiras e efeitos
///
/// O codec é síncrono, sem acesso a preferências, autenticação ou rede.
/// Não consegue determinar ausência de chave ou falha de leitura do meio.
/// Também não avalia expiração: uma evidência expirada pode ser reconstruída
/// corretamente e caberá ao controlador decidir sua reutilização.
///
/// Não remove conteúdo rejeitado nem executa migração automática.
/// Erros de programação continuam propagando fora das falhas de formato previstas.
library;

import 'dart:convert';

import 'package:capp/src/modules/identity/application/models/institutional_affiliation.dart';
import 'package:capp/src/modules/identity/application/models/institutional_profile.dart';
import 'package:capp/src/modules/identity/application/models/institutional_verification.dart';
import 'package:capp/src/modules/identity/application/session_cache_result.dart';

/// Codifica e interpreta registros institucionais de sessão.
///
/// A interpretação distingue JSON malformado, envelope incompatível,
/// proprietário divergente e dados de verificação inválidos.
///
/// Não avalia expiração, remove registros ou executa migrações automáticas.
/// Campos adicionais são ignorados durante a leitura.
final class SessionCacheCodec {
  /// Cria um codec sem recursos externos ou estado mutável.
  const SessionCacheCodec();

  /// Versão do formato persistente produzido e aceito pelo codec.
  static const int _schemaVersion = 1;

  /// Serializa [verification] em um envelope associado a [firebaseUid].
  ///
  /// Preserva campos opcionais nulos e grava a expiração em UTC.
  /// Não inclui credenciais ou objetos próprios do transporte HTTP.
  ///
  /// Não confere a identidade autenticada nem exige uma expiração futura.
  /// Erros inesperados de serialização propagam.
  String encode({
    required String firebaseUid,
    required InstitutionalVerification verification,
  }) {
    return jsonEncode({
      'schemaVersion': _schemaVersion,
      'firebaseUid': firebaseUid,
      'verification': _encodeVerification(verification),
    });
  }

  /// Interpreta [encoded] e confere seu proprietário.
  ///
  /// [expectedFirebaseUid] deve identificar a conta cujo registro foi solicitado.
  ///
  /// Retorna [SessionCacheFound] para dados compatíveis ou
  /// [SessionCacheRejected] para conteúdo que não pode ser reutilizado.
  /// Ausência de registro e falhas de acesso são determinadas pelo armazenamento.
  ///
  /// Falhas de formato preservam causa e pilha. Rejeições detectadas por
  /// comparação possuem somente o motivo correspondente.
  ///
  /// Não remove o conteúdo rejeitado. Erros de programação não são capturados
  /// como dados inválidos.
  SessionCacheLoadResult decode(
    String encoded, {
    required String expectedFirebaseUid,
  }) {
    final Object? decoded;

    try {
      decoded = jsonDecode(encoded);
    } on FormatException catch (error, stackTrace) {
      return SessionCacheRejected(
        SessionCacheRejectionReason.malformedJson,
        cause: error,
        stackTrace: stackTrace,
      );
    }

    if (decoded is! Map<String, dynamic>) {
      return const SessionCacheRejected(
        SessionCacheRejectionReason.invalidEnvelope,
      );
    }

    final version = decoded['schemaVersion'];
    final owner = decoded['firebaseUid'];

    if (version is! int || owner is! String) {
      return const SessionCacheRejected(
        SessionCacheRejectionReason.invalidEnvelope,
      );
    }

    if (version != _schemaVersion) {
      return const SessionCacheRejected(
        SessionCacheRejectionReason.unsupportedVersion,
      );
    }

    if (owner != expectedFirebaseUid) {
      return const SessionCacheRejected(
        SessionCacheRejectionReason.ownerMismatch,
      );
    }

    final verificationJson = decoded['verification'];

    if (verificationJson is! Map<String, dynamic>) {
      return const SessionCacheRejected(
        SessionCacheRejectionReason.invalidVerification,
      );
    }

    try {
      return SessionCacheFound(_readVerification(verificationJson));
    } on FormatException catch (error, stackTrace) {
      return SessionCacheRejected(
        SessionCacheRejectionReason.invalidVerification,
        cause: error,
        stackTrace: stackTrace,
      );
    }
  }

  /// Constrói a representação persistente da verificação.
  ///
  /// Os nomes dos campos são definidos pela versão do cache.
  /// Sua semelhança com o protocolo HTTP atual não cria uma dependência
  /// entre os dois contratos.
  Map<String, Object?> _encodeVerification(
    InstitutionalVerification verification,
  ) {
    final profile = verification.profile;

    final Map<String, Object?> affiliation = switch (profile.affiliation) {
      StudentAffiliation(:final courseName) => {
        'type': 'student',
        'courseName': courseName,
      },
      NonStudentAffiliation() => {'type': 'nonStudent'},
    };

    return {
      'profile': {
        'microsoftObjectId': profile.microsoftObjectId,
        'name': profile.name,
        'email': profile.email,
        'affiliation': affiliation,
      },
      'expiresAt': verification.expiresAt.toUtc().toIso8601String(),
    };
  }

  /// Reconstrói uma verificação a partir do formato persistente.
  ///
  /// Exige os campos obrigatórios da versão 1. Nome, e-mail e curso estudantil
  /// admitem nulo, mas precisam estar presentes nos respectivos objetos.
  ///
  /// A data segue a interpretação de DateTime.parse, preservando a política
  /// de leitura do formato anterior. O modelo normaliza o instante para UTC.
  ///
  /// Lança FormatException para conteúdo incompatível.
  InstitutionalVerification _readVerification(Map<String, dynamic> json) {
    final profile = _readObject(json['profile'], 'verification.profile');

    final affiliation = _readObject(
      profile['affiliation'],
      'verification.profile.affiliation',
    );

    final expiresAt = _readString(json['expiresAt'], 'verification.expiresAt');

    return InstitutionalVerification(
      profile: InstitutionalProfile(
        microsoftObjectId: _readString(
          profile['microsoftObjectId'],
          'profile.microsoftObjectId',
        ),
        name: _readNullableString(profile, 'name'),
        email: _readNullableString(profile, 'email'),
        affiliation: _readAffiliation(affiliation),
      ),
      expiresAt: DateTime.parse(expiresAt),
    );
  }

  /// Reconstrói a variante de vínculo definida no formato persistente.
  ///
  /// Estudantes exigem `courseName`, admitindo nulo.
  /// A variante não estudantil não admite esse campo, inclusive com valor nulo.
  ///
  /// Lança FormatException para discriminador ou conteúdo incompatível.
  InstitutionalAffiliation _readAffiliation(Map<String, dynamic> json) {
    final type = _readString(json['type'], 'affiliation.type');

    switch (type) {
      case 'student':
        return StudentAffiliation(
          courseName: _readNullableString(json, 'courseName'),
        );

      case 'nonStudent':
        if (json.containsKey('courseName')) {
          throw const FormatException(
            'Vínculo não estudantil não admite courseName no cache',
          );
        }

        return const NonStudentAffiliation();

      default:
        throw const FormatException(
          'Tipo de vínculo institucional desconhecido no cache',
        );
    }
  }

  /// Exige um objeto JSON em [value].
  ///
  /// [field] identifica o campo no diagnóstico sem incluir seu conteúdo.
  Map<String, dynamic> _readObject(Object? value, String field) {
    if (value is! Map<String, dynamic>) {
      throw FormatException('O campo "$field" deve ser um objeto JSON');
    }

    return value;
  }

  /// Exige uma string não nula em [value].
  ///
  /// Preserva o texto, inclusive vazio, conforme o formato anterior.
  /// [field] identifica o campo no diagnóstico.
  String _readString(Object? value, String field) {
    if (value is! String) {
      throw FormatException('O campo "$field" deve ser uma string');
    }

    return value;
  }

  /// Lê uma propriedade obrigatória que admite string ou nulo.
  ///
  /// Diferencia campo ausente de campo explicitamente nulo.
  /// Outros tipos produzem FormatException.
  String? _readNullableString(Map<String, dynamic> json, String field) {
    if (!json.containsKey(field)) {
      throw FormatException('O campo "$field" está ausente no cache');
    }

    final value = json[field];

    if (value == null) return null;
    if (value is String) return value;

    throw FormatException('O campo "$field" deve ser string ou nulo');
  }
}
