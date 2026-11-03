part of 'institutional_affiliation.dart';

/// Vínculo estudantil confirmado pelo backend.
///
/// Esta variante descreve o protocolo HTTP e é convertida para o modelo da
/// aplicação pelo mapper do gateway. O cache possui representação própria.
/// Construir o DTO não comprova validade atual ou autorização. O curso continua
/// sendo texto externo, sem associação ao catálogo acadêmico interno.
final class StudentAffiliation extends InstitutionalAffiliation {
  /// Cria o vínculo confirmado, mesmo quando o nome do curso não foi informado.
  const StudentAffiliation({required this.courseName});

  /// Curso informado em `jobTitle`, ou `null` se não fornecido.
  final String? courseName;

  /// Desserializa um vínculo `student` com a propriedade `courseName` presente.
  ///
  /// Lança [FormatException] se o tipo estiver incorreto, se o campo estiver
  /// ausente ou se seu valor não for uma string ou `null`.
  /// Outras chaves são aceitas; nenhum texto é aparado ou normalizado.
  factory StudentAffiliation.fromJson(Map<String, dynamic> json) {
    if (json['type'] != 'student' || !json.containsKey('courseName')) {
      throw const FormatException('Vínculo estudantil inválido');
    }
    final raw = json['courseName'];
    if (raw != null && raw is! String) {
      throw const FormatException('courseName deve ser string ou null');
    }
    return StudentAffiliation(courseName: raw as String?);
  }

  /// Serializa `type: student` e inclui `courseName`, mesmo quando nulo.
  @override
  Map<String, dynamic> toJson() => {
    'type': 'student',
    'courseName': courseName,
  };
}
