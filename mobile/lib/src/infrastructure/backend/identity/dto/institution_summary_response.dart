/// Resumo público da instituição retornado pela verificação institucional.
///
/// Espelha o objeto institution do contrato HTTP: UUID interno do CApp, nome
/// obrigatório e sigla obrigatória que admite nulo. Tenant e grupo Microsoft
/// permanecem no backend. O mapper converte este DTO no modelo da aplicação;
/// o cache mantém uma representação independente.
library;

import 'package:capp/src/infrastructure/backend/converters/uuid_value_converter.dart';
import 'package:copy_with_extension/copy_with_extension.dart';
import 'package:json_annotation/json_annotation.dart';
import 'package:uuid/uuid.dart';

part 'institution_summary_response.g.dart';

@CopyWith()
@JsonSerializable(
  checked: true,
  createToJson: true,
  disallowUnrecognizedKeys: false,
  explicitToJson: true,
)
/// Instituição reconhecida pelo backend, sem comprovar vínculo estudantil.
///
/// A leitura JSON valida o formato do UUID e exige todas as propriedades.
/// Chaves adicionais são aceitas e a sigla nula é preservada na serialização.
class InstitutionSummaryResponse {
  /// Recebe os dados já interpretados, sem validar novamente o UUID.
  InstitutionSummaryResponse({
    required this.id,
    required this.name,
    required this.acronym,
  });

  /// Identificador interno da instituição no backend.
  @UuidValueConverter()
  @JsonKey(name: r'id', required: true, includeIfNull: false)
  final UuidValue id;

  /// Nome da instituição fornecido pelo backend.
  @JsonKey(name: r'name', required: true, includeIfNull: false)
  final String name;

  /// Sigla da instituição fornecida pelo backend, ou `null` se ausente.
  @JsonKey(name: r'acronym', required: true, includeIfNull: true)
  final String? acronym;

  /// Compara identificador, nome e sigla por valor.
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is InstitutionSummaryResponse &&
          other.id == id &&
          other.name == name &&
          other.acronym == acronym;

  /// Hash dos mesmos atributos utilizados pela igualdade.
  @override
  int get hashCode =>
      id.hashCode + name.hashCode + (acronym == null ? 0 : acronym.hashCode);

  /// Lê campos obrigatórios; tipo ou formato inválido produz falha checked.
  factory InstitutionSummaryResponse.fromJson(Map<String, dynamic> json) =>
      _$InstitutionSummaryResponseFromJson(json);

  /// Serializa o UUID como texto e inclui a sigla mesmo quando nula.
  Map<String, dynamic> toJson() => _$InstitutionSummaryResponseToJson(this);

  /// Representação textual do JSON público da instituição.
  @override
  String toString() {
    return toJson().toString();
  }
}
