/// Contratos HTTP para restaurar a sessão e concluir o perfil interno.
///
/// O parse verifica o formato JSON, UUIDs e estados definidos pelo protocolo.
/// As invariantes do perfil são conferidas ao converter a resposta para o modelo.
/// A evidência usa o mesmo DTO do endpoint de verificação institucional.
library;

import 'package:uuid/uuid.dart';

import 'institutional_verification_response.dart';

final class InternalUserResponse {
  const InternalUserResponse({
    required this.id,
    required this.status,
    required this.nickname,
    required this.email,
    required this.currentSemester,
    required this.institutionEntrySemester,
    this.name,
    this.institutionalEmail,
    this.courseName,
  });

  final UuidValue id;
  final String status;
  final String? nickname;
  final String? email;
  final int? currentSemester;
  final String? institutionEntrySemester;
  final String? name;
  final String? institutionalEmail;
  final String? courseName;

  factory InternalUserResponse.fromJson(Map<String, dynamic> json) {
    for (final key in ['name', 'institutionalEmail', 'courseName']) {
      if (json[key] != null && json[key] is! String) {
        throw FormatException('Complemento de perfil incompatível: $key');
      }
    }

    for (final key in ['nickname', 'email', 'institutionEntrySemester']) {
      if (!json.containsKey(key) ||
          (json[key] != null && json[key] is! String)) {
        throw FormatException('Campo de perfil incompatível: $key');
      }
    }

    final id = json['id'];
    final status = json['status'];
    final semester = json['currentSemester'];

    if (id is! String ||
        !['incomplete', 'active', 'inactive'].contains(status) ||
        !json.containsKey('currentSemester') ||
        (semester != null && semester is! int)) {
      throw const FormatException('Perfil interno incompatível');
    }

    return InternalUserResponse(
      id: UuidValue.withFormatValidation(id),
      status: status as String,
      nickname: json['nickname'] as String?,
      email: json['email'] as String?,
      currentSemester: semester as int?,
      institutionEntrySemester: json['institutionEntrySemester'] as String?,
      name: json['name'] as String?,
      institutionalEmail: json['institutionalEmail'] as String?,
      courseName: json['courseName'] as String?,
    );
  }
}

final class AccountSessionResponse {
  const AccountSessionResponse({
    required this.user,
    required this.verification,
  });

  final InternalUserResponse user;
  final InstitutionalVerificationResponse? verification;

  factory AccountSessionResponse.fromJson(Map<String, dynamic> json) {
    final user = json['user'];
    final proof = json['verification'];

    if (user is! Map<String, dynamic> ||
        !json.containsKey('verification') ||
        (proof != null && proof is! Map<String, dynamic>)) {
      throw const FormatException('Sessão incompatível');
    }

    return AccountSessionResponse(
      user: InternalUserResponse.fromJson(user),
      verification: proof == null
          ? null
          : InstitutionalVerificationResponse.fromJson(
              proof as Map<String, dynamic>,
            ),
    );
  }
}
