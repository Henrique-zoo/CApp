/// Representação do envelope JSON público de erros do backend.
///
/// Os endpoints compartilham uma estrutura de erro, independentemente do módulo
/// que os consome. Esta biblioteca representa o objeto externo que contém o campo
/// error, deixando sua interpretação para o adaptador responsável pelo caso de uso.
///
/// ## Percurso do envelope
///
/// Dio mantém o corpo de uma resposta rejeitada. O gateway lê ApiError a partir
/// desse corpo e consulta ApiErrorDetails para classificar o código e conservar
/// a mensagem explicitamente pública.
///
/// A desserialização exige o formato previsto e propaga conteúdo incompatível.
/// Possuir um DTO de erro não instala um interceptor nem converte automaticamente
/// uma resposta HTTP em uma exceção de aplicação.
///
/// ## Separação entre protocolo e diagnóstico
///
/// O objeto não contém o status HTTP ou a falha de transporte; ambos permanecem
/// na resposta e na exceção Dio. O adaptador precisa preservar esse contexto
/// ao classificar a falha, inclusive se o próprio envelope estiver malformado.
///
/// O DTO não conhece sessão, navegação ou SDKs de autenticação.
library;

// Adaptado do cliente gerado pelo OpenAPI Generator.

// ignore_for_file: unused_element
import 'api_error_details.dart';

import 'package:copy_with_extension/copy_with_extension.dart';
import 'package:json_annotation/json_annotation.dart';

part 'api_error.g.dart';

/// Resposta de erro com os dados públicos agrupados na propriedade `error`.
///
/// Suporta serialização JSON e cópias com alterações por `copyWith`.
/// Campos adicionais são aceitos. A conversão checked exige a estrutura do DTO,
/// sem validar o catálogo de códigos ou sanitizar mensagens e detalhes.
@CopyWith()
@JsonSerializable(
  checked: true,
  createToJson: true,
  disallowUnrecognizedKeys: false,
  explicitToJson: true,
)
class ApiError {
  /// Código, mensagem e detalhes públicos do erro.
  @JsonKey(name: r'error', required: true, includeIfNull: false)
  final ApiErrorDetails error;

  /// Cria o envelope com o erro retornado pelo backend.
  /// Não faz I/O ou validação de negócio sobre [error].
  const ApiError({required this.error});

  /// Compara [error] usando a igualdade implementada por seu tipo.
  @override
  bool operator ==(Object other) =>
      identical(this, other) || other is ApiError && other.error == error;

  /// Hash derivado do mesmo conteúdo usado pela igualdade.
  @override
  int get hashCode => error.hashCode;

  /// Desserializa o envelope, exigindo a propriedade não nula `error`.
  ///
  /// Propriedades desconhecidas são aceitas; campos inválidos causam
  /// uma falha de desserialização.
  /// A conversão gerada encapsula falhas em `CheckedFromJsonException`.
  /// Não interpreta o status, não navega e não encerra autenticação.
  factory ApiError.fromJson(Map<String, dynamic> json) =>
      _$ApiErrorFromJson(json);

  /// Serializa o envelope e o erro interno para o formato do backend.
  Map<String, dynamic> toJson() => _$ApiErrorToJson(this);

  /// Converte o JSON em texto, incluindo mensagem e detalhes sem ocultação.
  @override
  String toString() {
    return toJson().toString();
  }
}
