/// Envelope HTTP do perfil, da instituição e da data de expiração da evidência.
///
/// Este DTO representa a evidência presente nas respostas de verificação e de
/// restauração da sessão. O protocolo agrupa profile, institution e expiresAt
/// dentro de verification, ao lado do perfil interno em user.
///
/// ## Participação no fluxo
///
/// IdentityApi desserializa a resposta, e o gateway converte este DTO em
/// InstitutionalVerification. O caso de uso associa o modelo ao UID observado;
/// o controlador confere contexto e prazo antes de permitir entrada.
///
/// A existência do DTO não significa que a evidência ainda esteja válida.
/// A leitura converte a data recebida, mas não consulta relógio nem aplica
/// a política de prontidão da sessão.
///
/// ## Separação de representações
///
/// O objeto pertence ao contrato de rede e não é o formato persistente da sessão.
/// A gravação local codifica modelos de aplicação em um envelope versionado,
/// sem depender destes conversores HTTP.
///
/// Credenciais Microsoft e Firebase não aparecem na resposta. O prazo
/// institucional é independente da expiração do token de autenticação e não
/// substitui a autorização de cada requisição pelo servidor.
///
/// ## Exemplo
///
/// A leitura é local e não autentica ou consulta serviços:
///
/// ```dart
/// final response = InstitutionalVerificationResponse.fromJson({
///   'profile': {
///     'microsoftObjectId': '00000000-0000-0000-0000-000000000001',
///     'name': null,
///     'email': null,
///     'affiliation': {'type': 'nonStudent'},
///   },
///   'institution': {
///     'id': '00000000-0000-0000-0000-000000000001',
///     'name': 'Instituição',
///     'acronym': null,
///   },
///   'expiresAt': '2026-10-25T12:00:00Z',
/// });
/// assert(response.expiresAt.isUtc);
/// ```
library;

// Adaptado do cliente gerado pelo OpenAPI Generator.

// ignore_for_file: unused_element
import 'package:capp/src/infrastructure/backend/identity/dto/institution_summary_response.dart';

import 'institutional_identity_response.dart';

import 'package:copy_with_extension/copy_with_extension.dart';
import 'package:json_annotation/json_annotation.dart';

part 'institutional_verification_response.g.dart';

/// Evidência institucional composta por perfil, instituição e data de expiração.
///
/// Suporta serialização JSON e cópias com alterações por `copyWith`.
/// A criação do objeto não registra temporizador, não agenda renovação e não
/// confirma que o servidor ainda aceitará a evidência em uma requisição futura.
@CopyWith()
@JsonSerializable(
  checked: true,
  createToJson: true,
  disallowUnrecognizedKeys: false,
  explicitToJson: true,
)
class InstitutionalVerificationResponse {
  /// Cria o resultado com o perfil, a instituição e o instante de expiração.
  /// Não exige instante futuro nem converte seu fuso automaticamente.
  InstitutionalVerificationResponse({
    required this.profile,
    required this.institution,
    required this.expiresAt,
  });

  /// Perfil institucional confirmado pelo backend.
  @JsonKey(name: r'profile', required: true, includeIfNull: false)
  final InstitutionalIdentityResponse profile;

  /// Resumo público da instituição, sem tenant ou configuração de grupos.
  @JsonKey(name: r'institution', required: true, includeIfNull: false)
  final InstitutionSummaryResponse institution;

  /// Expiração da evidência institucional, recebida em UTC no formato RFC 3339.
  ///
  /// Corresponde a 120 dias após o início das consultas institucionais.
  /// É independente da expiração do token Firebase e não substitui a
  /// validação no servidor. A resposta não inclui um campo de TTL separado.
  @JsonKey(name: r'expiresAt', required: true, includeIfNull: false)
  final DateTime expiresAt;

  /// Compara perfil, instituição e instante pelos respectivos valores.
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is InstitutionalVerificationResponse &&
          other.profile == profile &&
          other.institution == institution &&
          other.expiresAt == expiresAt;

  /// Hash combinado do perfil, da instituição e do instante.
  @override
  int get hashCode =>
      profile.hashCode + institution.hashCode + expiresAt.hashCode;

  /// Desserializa o perfil e converte `expiresAt` de uma string para [DateTime].
  ///
  /// Perfil, instituição e expiração são obrigatórios e não nulos. Propriedades
  /// desconhecidas são aceitas; dados inválidos causam falha de desserialização.
  /// A data segue `DateTime.parse`; o cliente não verifica que o texto recebido
  /// estava em UTC ou representa a política de 120 dias do servidor. Falhas são
  /// encapsuladas pela conversão checked, inclusive datas e UUIDs incompatíveis.
  factory InstitutionalVerificationResponse.fromJson(
    Map<String, dynamic> json,
  ) => _$InstitutionalVerificationResponseFromJson(json);

  /// Serializa perfil e instituição com UUIDs textuais e expiração em ISO 8601.
  Map<String, dynamic> toJson() =>
      _$InstitutionalVerificationResponseToJson(this);

  /// Converte o JSON em texto, incluindo os dados pessoais do perfil.
  @override
  String toString() {
    return toJson().toString();
  }
}
