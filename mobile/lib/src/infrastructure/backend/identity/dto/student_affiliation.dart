part of 'institutional_affiliation.dart';

/// Vínculo estudantil confirmado pelo backend.
///
/// Esta variante descreve o protocolo HTTP e é convertida para o modelo da
/// aplicação pelo mapper do gateway. O cache possui representação própria.
/// Construir o DTO não comprova validade atual ou autorização. O curso continua
/// sendo texto externo, sem associação ao catálogo acadêmico interno.
final class StudentAffiliation extends InstitutionalAffiliation {
  /// Cria o vínculo confirmado com curso não vazio fornecido pelo Graph.
  StudentAffiliation({required this.courseName}) {
    if (courseName.trim().isEmpty) {
      throw const FormatException(
        'Vínculo estudantil exige curso institucional',
      );
    }
  }

  /// Curso não vazio informado em `jobTitle`.
  final String courseName;

  /// Desserializa um vínculo `student` com a propriedade `courseName` presente.
  ///
  /// Lança [FormatException] se o tipo estiver incorreto, se o campo estiver
  /// ausente, nulo, vazio ou composto apenas por espaços.
  /// Outras chaves são aceitas; nenhum texto é aparado ou normalizado.
  factory StudentAffiliation.fromJson(Map<String, dynamic> json) {
    if (json['type'] != 'student' || !json.containsKey('courseName')) {
      throw const FormatException('Vínculo estudantil inválido');
    }
    final raw = json['courseName'];
    if (raw is! String) {
      throw const FormatException('courseName deve ser uma string não vazia');
    }
    return StudentAffiliation(courseName: raw);
  }

  /// Serializa `type: student` e seu curso institucional obrigatório.
  @override
  Map<String, dynamic> toJson() => {
    'type': 'student',
    'courseName': courseName,
  };
}
