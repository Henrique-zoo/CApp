part of 'institutional_affiliation.dart';

/// Vínculo cuja participação no grupo de estudantes não foi confirmada.
///
/// Não informa a profissão nem identifica outra categoria institucional.
/// É um resultado conclusivo da consulta recebida, não indisponibilidade do
/// provedor. A apresentação pode informar a ausência de reconhecimento estudantil
/// sem encerrar a autenticação e sem derivar permissões gerais deste tipo.
final class NonStudentAffiliation extends InstitutionalAffiliation {
  /// Cria o resultado sem confirmação de vínculo estudantil.
  const NonStudentAffiliation();

  /// Desserializa um vínculo `nonStudent`, que não admite `courseName`.
  ///
  /// Lança [FormatException] se o tipo estiver incorreto ou se houver a
  /// propriedade `courseName`, inclusive quando seu valor for `null`.
  /// Outras chaves desconhecidas são aceitas; não há chamada ao diretório.
  factory NonStudentAffiliation.fromJson(Map<String, dynamic> json) {
    if (json['type'] != 'nonStudent' || json.containsKey('courseName')) {
      throw const FormatException('Vínculo não estudantil inválido');
    }
    return const NonStudentAffiliation();
  }

  /// Serializa somente o discriminador `type: nonStudent`.
  @override
  Map<String, dynamic> toJson() => {'type': 'nonStudent'};
}
