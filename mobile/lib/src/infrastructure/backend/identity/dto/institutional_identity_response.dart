/// Representação HTTP do perfil e do vínculo confirmados pelo backend.
///
/// Esta biblioteca espelha o contrato JSON do perfil institucional. Ela permanece
/// na infraestrutura para que nomes de campos e regras de desserialização não
/// se tornem dependências da coordenação ou das telas.
///
/// ## Percurso da resposta
///
/// InstitutionalVerificationResponse agrega este DTO ao expiresAt.
/// Depois da desserialização, o mapper do gateway constrói o perfil próprio da
/// aplicação e converte a união de vínculo para suas variantes internas.
///
/// microsoftObjectId identifica o objeto Microsoft, não o UID Firebase.
/// name e email são propriedades obrigatórias que admitem nulo; ausência
/// de informação externa não é preenchida por inferência no cliente.
///
/// ## Responsabilidade do contrato
///
/// O objeto representa os dados retornados, sem provisionar usuário interno,
/// associar a centros acadêmicos ou definir permissões.
/// A criação manual não valida o UUID nem comprova que o backend ainda aceitará
/// a evidência em outra requisição.
///
/// O formato persistente local pertence ao codec do módulo e não depende
/// da serialização deste DTO.
library;

// Adaptado do cliente gerado pelo OpenAPI Generator.

// ignore_for_file: unused_element
import 'institutional_affiliation.dart';

import 'package:copy_with_extension/copy_with_extension.dart';
import 'package:json_annotation/json_annotation.dart';

part 'institutional_identity_response.g.dart';

/// Perfil institucional consultado no diretório Microsoft pelo backend.
///
/// Suporta serialização JSON e cópias com alterações por `copyWith`.
/// A conversão aceita chaves adicionais e preserva campos opcionais nulos.
/// Não faz consultas ao diretório, comparação com UID Firebase ou validação
/// local do UUID informado em `microsoftObjectId`.
@CopyWith()
@JsonSerializable(
  checked: true,
  createToJson: true,
  disallowUnrecognizedKeys: false,
  explicitToJson: true,
)
class InstitutionalIdentityResponse {
  /// Cria o perfil; [name] e [email] são obrigatórios, mas admitem `null`.
  /// Não normaliza os textos nem infere vínculo pelo domínio do e-mail ou curso.
  InstitutionalIdentityResponse({
    required this.microsoftObjectId,
    required this.name,
    required this.email,
    required this.affiliation,
  });

  /// Identificador de objeto Microsoft validado como UUID pelo backend.
  ///
  /// É distinto do UID Firebase e de um eventual ID interno do CApp.
  @JsonKey(name: r'microsoftObjectId', required: true, includeIfNull: false)
  final String microsoftObjectId;

  /// `displayName` fornecido pela instituição, ou `null` se ausente.
  @JsonKey(name: r'name', required: true, includeIfNull: true)
  final String? name;

  /// `mail` fornecido pelo diretório Microsoft, ou `null` se ausente.
  ///
  /// O backend não substitui um e-mail ausente por `userPrincipalName`.
  @JsonKey(name: r'email', required: true, includeIfNull: true)
  final String? email;

  /// Resultado da consulta ao vínculo estudantil no diretório institucional.
  /// Deve ser apresentado como fato conhecido, sem ser convertido automaticamente
  /// em um catálogo de capacidades do aplicativo.
  @JsonKey(name: r'affiliation', required: true, includeIfNull: false)
  final InstitutionalAffiliation affiliation;

  /// Compara os atributos e delega a igualdade de [affiliation] ao seu objeto.
  /// As variantes atuais do vínculo não implementam igualdade estrutural própria.
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is InstitutionalIdentityResponse &&
          other.microsoftObjectId == microsoftObjectId &&
          other.name == name &&
          other.email == email &&
          other.affiliation == affiliation;

  /// Hash dos mesmos atributos usados pela igualdade, com zero para campos nulos.
  @override
  int get hashCode =>
      microsoftObjectId.hashCode +
      (name == null ? 0 : name.hashCode) +
      (email == null ? 0 : email.hashCode) +
      affiliation.hashCode;

  /// Desserializa o perfil, exigindo todas as propriedades do contrato.
  ///
  /// `name` e `email` admitem `null`. O vínculo é convertido de acordo com seu
  /// discriminador `type`; propriedades desconhecidas do perfil são aceitas.
  /// Campos, formatos ou variantes inválidos propagam pela conversão checked.
  factory InstitutionalIdentityResponse.fromJson(Map<String, dynamic> json) =>
      _$InstitutionalIdentityResponseFromJson(json);

  /// Serializa o perfil e o vínculo, preservando nome e e-mail nulos.
  Map<String, dynamic> toJson() => _$InstitutionalIdentityResponseToJson(this);

  /// Retorna o JSON como texto, incluindo nome e e-mail sem ocultação.
  @override
  String toString() {
    return toJson().toString();
  }
}
