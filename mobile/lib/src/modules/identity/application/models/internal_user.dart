/// Perfil interno que permite retomar o provisionamento sem depender do Graph.
///
/// E-mail de login e dados acadêmicos escolhidos pelo usuário são independentes
/// do perfil institucional. O backend decide o estado; a apresentação apenas
/// escolhe entre concluir o cadastro e entrar no aplicativo.
/// O complemento de nome é uma declaração para uma ausência do Graph. A
/// apresentação dá prioridade ao dado externo e identifica o complemento;
/// o nome declarado não comprova um vínculo. O e-mail do Graph pode acompanhar
/// o resultado do cadastro como sugestão transitória, sem integrar este perfil.
library;

import 'package:uuid/uuid.dart';

import 'institutional_verification.dart';

/// Estado da conta CApp, distinto da validade institucional e da sessão Firebase.
enum InternalUserStatus { incomplete, active, inactive }

/// Fotografia da conta provisionada. Não contém senha nem tokens.
final class InternalUser {
  /// Rejeita valores inconsistentes antes de expor a fotografia à aplicação.
  /// A regra vale igualmente para respostas HTTP e para reconstituição do cache.
  factory InternalUser({
    required UuidValue id,
    required InternalUserStatus status,
    String? nickname,
    String? email,
    int? currentSemester,
    String? institutionEntrySemester,
    String? name,
  }) {
    if ((currentSemester != null && currentSemester < 1) ||
        (institutionEntrySemester != null &&
            !isValidInstitutionEntrySemester(institutionEntrySemester)) ||
        (status == InternalUserStatus.active &&
            (nickname == null ||
                nickname.trim().isEmpty ||
                email == null ||
                email.isEmpty ||
                currentSemester == null ||
                institutionEntrySemester == null))) {
      throw const FormatException('Perfil interno inconsistente');
    }

    return InternalUser._(
      id: id,
      status: status,
      nickname: nickname,
      email: email,
      currentSemester: currentSemester,
      institutionEntrySemester: institutionEntrySemester,
      name: name,
    );
  }

  const InternalUser._({
    required this.id,
    required this.status,
    this.nickname,
    this.email,
    this.currentSemester,
    this.institutionEntrySemester,
    this.name,
  });

  final UuidValue id;
  final InternalUserStatus status;
  final String? nickname;
  final String? email;

  /// Nome declarado para uma ausência do Graph; não é uma comprovação.
  final String? name;

  /// Período atual do curso, a partir de 1.
  final int? currentSemester;

  /// Semestre de ingresso na instituição, no formato AAAA.1 ou AAAA.2.
  final String? institutionEntrySemester;
}

/// Resposta autenticada que dispensa Microsoft enquanto a evidência estiver válida.
final class AccountSession {
  const AccountSession({
    required this.user,
    required this.verification,
    this.suggestedEmail,
  });

  final InternalUser user;

  /// Nulo quando a evidência precisa de sincronização institucional.
  final InstitutionalVerification? verification;

  /// Sugestão transitória do Graph para concluir o cadastro; nunca vai ao cache.
  /// A restauração de sessão não fornece esse campo e contas ativas o ignoram.
  final String? suggestedEmail;
}

/// Dados necessários para concluir o perfil, sem UID ou estado enviados pelo cliente.
final class ProfileCompletion {
  const ProfileCompletion({
    required this.nickname,
    required this.currentSemester,
    required this.institutionEntrySemester,
    this.name,
  });

  final String nickname;
  final int currentSemester;
  final String institutionEntrySemester;

  /// Nome enviado somente quando ausente no Graph; não altera a evidência.
  final String? name;
}

/// Valida o semestre institucional no formato AAAA.1 ou AAAA.2.
/// É compartilhado pela reconstituição do perfil e pela validação do formulário.
bool isValidInstitutionEntrySemester(String value) =>
    RegExp(r'^\d{4}\.[12]$').hasMatch(value);
