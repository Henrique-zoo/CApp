/// Resumo da instituição conhecida pela verificação institucional.
///
/// O identificador pertence ao cadastro interno do CApp; não é o tenant Microsoft
/// nem o identificador da pessoa no diretório. O gateway e o codec constroem este
/// modelo a partir de seus próprios contratos, sem propagar DTOs ou JSON.
/// A instituição não comprova vínculo estudantil nem concede permissões.
library;

import 'package:uuid/uuid.dart';

/// Nome, sigla opcional e identificador interno de uma instituição.
///
/// É um valor imutável com igualdade estrutural, sem configuração do diretório.
final class InstitutionSummary {
  /// Preserva os dados recebidos; [acronym] é obrigatório, mas admite nulo.
  ///
  /// A construção não valida novamente [id] nem normaliza nome ou sigla.
  const InstitutionSummary({
    required this.id,
    required this.name,
    required this.acronym,
  });

  /// Identificador interno do cadastro de instituições do CApp.
  final UuidValue id;

  /// Nome da instituição registrado no backend.
  final String name;

  /// Sigla cadastrada, ou nulo quando a instituição não a informa.
  final String? acronym;

  /// Compara identificador, nome e sigla por valor.
  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        other is InstitutionSummary &&
            other.id == id &&
            other.name == name &&
            other.acronym == acronym;
  }

  /// Hash dos mesmos atributos usados pela igualdade.
  @override
  int get hashCode => Object.hash(id, name, acronym);
}
