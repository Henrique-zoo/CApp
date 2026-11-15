/// Contratos HTTP para restaurar a sessão e concluir o perfil interno.
///
/// A restauração precisa distinguir uma conta existente sem evidência de uma
/// conta ainda não provisionada. Estes DTOs preservam essa diferença do protocolo
/// sem transferir códigos HTTP ou mapas JSON para a coordenação da sessão.
/// Pertencem à infraestrutura compartilhada e espelham o OpenAPI; não são o formato
/// do cache nem os modelos usados pela apresentação.
///
/// ## Percurso e validação
///
/// O cliente HTTP entrega o objeto JSON às fábricas, que conferem estrutura,
/// UUID e discriminadores. O gateway então converte o usuário em `InternalUser`,
/// onde são validadas as invariantes de perfil ativo, período e ingresso.
/// A evidência utiliza [InstitutionalVerificationResponse], compartilhada com a
/// verificação institucional. Reaproveitar os DTOs evita dois contratos para os
/// mesmos dados; a sugestão transitória de e-mail pertence a outra resposta.
///
/// Uma evidência nula continua sendo resposta válida. Expiração, propriedade do
/// UID e escolha do destino são conferidas pela aplicação, não por esta leitura.
/// Os objetos são imutáveis e não autenticam, consultam provedores ou persistem.
library;

import 'package:uuid/uuid.dart';

import 'institutional_verification_response.dart';

/// Representação HTTP da conta interna, antes da conversão para o modelo validado.
///
/// Campos de cadastro podem ser nulos para uma conta incompleta. Nome declarado
/// é complemento opcional; e-mail é o endereço escolhido para a credencial interna.
final class InternalUserResponse {
  /// Guarda valores já tipados sem executar [InternalUserResponse.fromJson].
  ///
  /// Argumentos obrigatórios nullable representam propriedades presentes no JSON,
  /// ainda sem valor durante o provisionamento. Quem constrói manualmente este
  /// DTO deve respeitar o contrato; isso não valida as invariantes da aplicação.
  const InternalUserResponse({
    required this.id,
    required this.status,
    required this.nickname,
    required this.email,
    required this.currentSemester,
    required this.institutionEntrySemester,
    this.name,
  });

  /// UUID interno da conta, distinto dos identificadores dos provedores.
  final UuidValue id;

  /// Discriminador de protocolo: `incomplete`, `active` ou `inactive`.
  final String status;

  /// Apelido escolhido, ou nulo enquanto ainda não foi concluído o perfil.
  final String? nickname;

  /// Endereço da credencial interna; não é um segundo contato institucional.
  final String? email;

  /// Período atual, cuja faixa válida é conferida no modelo de aplicação.
  final int? currentSemester;

  /// Ingresso informado como texto; o modelo confere o formato AAAA.1/AAAA.2.
  final String? institutionEntrySemester;

  /// Complemento de nome declarado para uma ausência do Graph; pode ser omitido.
  final String? name;

  /// Interpreta a fotografia HTTP e conserva ausências permitidas pelo contrato.
  ///
  /// [json] deve conter UUID, estado e as propriedades de apelido, e-mail, período
  /// e ingresso, mesmo quando seus valores forem nulos. O nome pode estar ausente.
  /// Chaves adicionais são ignoradas; valores são preservados sem normalização.
  ///
  /// Lança [FormatException] para propriedade obrigatória ausente, UUID inválido,
  /// estado desconhecido ou tipo incompatível. Não verifica completude do estado
  /// ativo, faixa do período, formato do ingresso ou sintaxe do e-mail; essas
  /// invariantes pertencem à conversão para o modelo de aplicação.
  ///
  /// ```dart
  /// final dto = InternalUserResponse.fromJson({
  ///   'id': '00000000-0000-0000-0000-000000000010',
  ///   'status': 'incomplete', 'nickname': null, 'email': null,
  ///   'currentSemester': null, 'institutionEntrySemester': null,
  /// });
  /// assert(dto.nickname == null);
  /// ```
  factory InternalUserResponse.fromJson(Map<String, dynamic> json) {
    for (final key in ['name']) {
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
    );
  }
}

/// Conta restaurada com evidência opcional, sem credenciais ou sugestão de cadastro.
///
/// `verification: null` preserva a conta para que a coordenação sincronize o vínculo.
/// Isso não equivale a ausência de usuário Firebase ou a falha de autenticação.
final class AccountSessionResponse {
  /// Agrupa DTOs já lidos; não avalia expiração, propriedade ou prontidão da sessão.
  const AccountSessionResponse({
    required this.user,
    required this.verification,
  });

  /// Conta interna recuperada, inclusive se ainda estiver incompleta.
  final InternalUserResponse user;

  /// Evidência reutilizável ou nulo quando o backend exige nova sincronização.
  final InstitutionalVerificationResponse? verification;

  /// Lê o envelope de restauração e delega os objetos aos respectivos DTOs.
  ///
  /// [json] exige `user` como objeto e a presença de `verification`, que pode ser
  /// objeto ou nulo. Omissão não é tratada como confirmação ausente. A fábrica não
  /// calcula prazos nem atribui direitos a partir do vínculo recebido.
  ///
  /// Lança [FormatException] para envelope incompatível e propaga as falhas das
  /// fábricas aninhadas, inclusive de UUID e serialização checked. A tradução
  /// dessas falhas para o vocabulário de identidade pertence ao gateway.
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
