/// Modelo de comentário ou resposta na linha do tempo de uma demanda.
library;

import 'package:uuid/uuid.dart';

/// Papel do autor de um comentário na demanda.
enum DemandCommentAuthorRole {
  student('student', 'Estudante'),
  caManagement('ca_management', 'Gestão do CA');

  const DemandCommentAuthorRole(this.wireValue, this.label);

  /// Valor canônico trafegado no contrato HTTP.
  final String wireValue;

  /// Rótulo público exibido na thread de interações.
  final String label;

  /// Converte o valor textual da API para [DemandCommentAuthorRole].
  static DemandCommentAuthorRole fromWire(String value) {
    for (final role in DemandCommentAuthorRole.values) {
      if (role.wireValue == value) return role;
    }
    throw FormatException('Papel de autor desconhecido: $value');
  }
}

/// Interação textual publicada na linha do tempo de uma demanda.
final class DemandComment {
  const DemandComment({
    required this.id,
    required this.demandId,
    required this.authorId,
    required this.authorRole,
    required this.content,
    required this.createdAt,
  });

  final UuidValue id;
  final UuidValue demandId;
  final UuidValue authorId;
  final DemandCommentAuthorRole authorRole;
  final String content;
  final DateTime createdAt;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is DemandComment &&
          other.id == id &&
          other.demandId == demandId &&
          other.authorId == authorId &&
          other.authorRole == authorRole &&
          other.content == content &&
          other.createdAt == createdAt;

  @override
  int get hashCode =>
      Object.hash(id, demandId, authorId, authorRole, content, createdAt);
}
