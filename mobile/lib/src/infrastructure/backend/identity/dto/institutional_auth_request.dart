/// Representação JSON da credencial Microsoft enviada à verificação.
///
/// Esta biblioteca define o corpo do endpoint institucional, preservando a
/// separação entre a credencial delegada Microsoft e a autenticação Firebase
/// da própria requisição HTTP.
///
/// ## Percurso da credencial
///
/// O caso de uso obtém o access token Microsoft e o gateway constrói
/// InstitutionalAuthRequest. IdentityApi serializa microsoftAccessToken no JSON,
/// enquanto o interceptor do transporte obtém o ID token Firebase separadamente
/// e o inclui em Authorization.
///
/// O DTO representa o formato, sem comprovar autenticidade, assinatura ou prazo
/// da string. As guardas de uso e a validação do servidor continuam necessárias.
///
/// ## Serialização e alcance
///
/// As anotações exigem a propriedade na leitura e delegam conversores e
/// copyWith à parte gerada. Regenerar esses métodos não altera o significado
/// de segurança da credencial.
///
/// toJson contém o token e é destinado ao envio, não à persistência do perfil.
/// A biblioteca não executa login, faz I/O ou salva o corpo em preferências.
library;

// Adaptado do cliente gerado pelo OpenAPI Generator.

// ignore_for_file: unused_element
import 'package:copy_with_extension/copy_with_extension.dart';
import 'package:json_annotation/json_annotation.dart';

part 'institutional_auth_request.g.dart';

/// Credencial Microsoft enviada para a verificação institucional no backend.
///
/// O Firebase ID Token acompanha a requisição no cabeçalho de autorização.
/// Suporta serialização JSON e cópias com alterações por `copyWith`.
@CopyWith()
@JsonSerializable(
  checked: true,
  createToJson: true,
  disallowUnrecognizedKeys: false,
  explicitToJson: true,
)
class InstitutionalAuthRequest {
  /// Cria a requisição com o token delegado obtido no login Microsoft.
  ///
  /// A validade e o conteúdo do token são verificados pelo backend.
  /// O construtor aceita qualquer string, inclusive vazia; a rejeição de negócio
  /// descrita pelo contrato ocorre no servidor.
  InstitutionalAuthRequest({required this.microsoftAccessToken});

  /// Access token delegado para as consultas ao Microsoft Graph.
  ///
  /// Permite consultar organização, perfil e participação em grupos.
  /// É distinto do Firebase ID Token. O backend rejeita strings vazias e
  /// não aplica `trim` antes de consultar o provedor.
  @JsonKey(name: r'microsoftAccessToken', required: true, includeIfNull: false)
  final String microsoftAccessToken;

  /// Compara a string da credencial, sem validar assinatura ou expiração.
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is InstitutionalAuthRequest &&
          other.microsoftAccessToken == microsoftAccessToken;

  /// Hash da string, destinado à igualdade local e não a uma impressão segura.
  @override
  int get hashCode => microsoftAccessToken.hashCode;

  /// Desserializa o campo obrigatório e não nulo `microsoftAccessToken`.
  ///
  /// Propriedades desconhecidas são aceitas. Esta conversão não valida
  /// a autenticidade nem a expiração da credencial.
  /// Falhas de tipo ou campo obrigatório são encapsuladas pela conversão checked.
  factory InstitutionalAuthRequest.fromJson(Map<String, dynamic> json) =>
      _$InstitutionalAuthRequestFromJson(json);

  /// Serializa o corpo HTTP, incluindo o token sem ocultação.
  ///
  /// O mapa contém uma credencial e não deve ser registrado em logs.
  Map<String, dynamic> toJson() => _$InstitutionalAuthRequestToJson(this);

  /// Retorna uma representação textual com o token oculto.
  @override
  String toString() {
    return 'InstitutionalAuthRequest(microsoftAccessToken: [REDACTED])';
  }
}
