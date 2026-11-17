/// Conteúdo público e programático de uma resposta de erro do backend.
///
/// Esta biblioteca representa code, message e details do envelope HTTP.
/// Separar esse conteúdo do erro de aplicação permite que cada módulo interprete
/// o mesmo protocolo de acordo com as operações que oferece.
///
/// ## Uso na fronteira
///
/// ApiError envolve este DTO. O gateway usa code para reconhecer a falha,
/// conserva a mensagem como pública e mantém o diagnóstico de transporte
/// separadamente. Diferentes códigos podem compartilhar o mesmo status HTTP;
/// a classificação não deve depender apenas desse status.
///
/// details admite qualquer valor JSON ou nulo e não corresponde a uma pilha
/// técnica do servidor. As propriedades exigidas continuam obrigatórias mesmo
/// quando seu conteúdo admite nulo.
///
/// ## Limites do tipo
///
/// O DTO não mantém um catálogo fechado de códigos e não decide qual mensagem
/// uma tela deve renderizar. Construtores manuais não sanitizam conteúdo.
/// A distinção entre mensagem pública e causa técnica é responsabilidade das
/// fronteiras que constroem as falhas da aplicação.
///
/// Não há requisição, recuperação automática ou alteração da sessão nesta
/// biblioteca.
library;

// Adaptado do cliente gerado pelo OpenAPI Generator.

// ignore_for_file: unused_element
import 'package:copy_with_extension/copy_with_extension.dart';
import 'package:json_annotation/json_annotation.dart';

part 'api_error_details.g.dart';

/// Conteúdo do campo `error` nas respostas de erro do backend.
///
/// Suporta serialização JSON e cópias com alterações por `copyWith`.
/// O tipo não impõe um enum fechado de códigos nem sanitiza dados fornecidos
/// diretamente ao construtor. Detalhes são valores JSON opcionais, não um tipo
/// de diagnóstico interno garantido pela conversão.
@CopyWith()
@JsonSerializable(
  checked: true,
  createToJson: true,
  disallowUnrecognizedKeys: false,
  explicitToJson: true,
)
class ApiErrorDetails {
  /// Cria o erro público; [details] é obrigatório, mas pode ser `null`.
  /// A construção não interpreta o código nem faz requisições adicionais.
  ApiErrorDetails({
    required this.code,
    required this.message,
    required this.details,
  });

  /// Identificador estável para tratamento programático.
  ///
  /// Os códigos aplicáveis estão descritos em cada resposta HTTP do contrato.
  @JsonKey(name: r'code', required: true, includeIfNull: false)
  final String code;

  /// Mensagem pública legível, sem diagnósticos internos ou credenciais.
  @JsonKey(name: r'message', required: true, includeIfNull: false)
  final String message;

  /// Contexto público adicional, ou `null` quando não há detalhes.
  ///
  /// O formato depende de [code]. O consumidor não deve assumir que todo detalhe
  /// seja nulo ou mapa. Ausência de vínculo estudantil é um resultado de sucesso
  /// da verificação e não deve ser inferida a partir deste envelope de erro.
  @JsonKey(name: r'details', required: true, includeIfNull: true)
  final Object? details;

  /// Compara código, mensagem e detalhes pela igualdade de cada objeto.
  /// Mapas e listas em [details] não recebem comparação profunda adicional.
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is ApiErrorDetails &&
          other.code == code &&
          other.message == message &&
          other.details == details;

  /// Hash derivado de código, mensagem e detalhes, com zero para detalhe nulo.
  @override
  int get hashCode =>
      code.hashCode +
      message.hashCode +
      (details == null ? 0 : details.hashCode);

  /// Desserializa `code`, `message` e `details`, exigindo as três propriedades.
  ///
  /// Apenas `details` admite `null`. Propriedades desconhecidas são aceitas;
  /// campos inválidos causam uma falha de desserialização.
  /// Falhas de leitura checked são propagadas como `CheckedFromJsonException`.
  /// A propriedade `details` pode conter um valor JSON de qualquer tipo.
  factory ApiErrorDetails.fromJson(Map<String, dynamic> json) =>
      _$ApiErrorDetailsFromJson(json);

  /// Serializa o erro, incluindo `details` mesmo quando seu valor é `null`.
  Map<String, dynamic> toJson() => _$ApiErrorDetailsToJson(this);

  /// Serializa todos os campos em texto, sem ocultar mensagem ou detalhes.
  @override
  String toString() {
    return toJson().toString();
  }
}
