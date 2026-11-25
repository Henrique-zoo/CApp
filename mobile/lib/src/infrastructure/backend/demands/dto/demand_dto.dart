/// Contratos de transferência HTTP (DTOs) para o módulo de demandas.
///
/// Implementa serialização e desserialização estritas conforme o contrato
/// OpenAPI do backend, validando UUIDs, tipos e datas na fronteira de rede.
library;

import 'package:uuid/uuid.dart';

UuidValue _parseUuid(Object? value, String field) {
  if (value is! String) {
    throw FormatException('Campo UUID inválido ou ausente: $field');
  }
  return UuidValue.withFormatValidation(value);
}

String _parseRequiredString(Object? value, String field) {
  if (value is! String) {
    throw FormatException('Campo textual inválido ou ausente: $field');
  }
  return value;
}

String? _parseOptionalString(Object? value, String field) {
  if (value == null) return null;
  if (value is! String) {
    throw FormatException('Campo textual opcional inválido: $field');
  }
  return value;
}

DateTime _parseDateTime(Object? value, String field) {
  if (value is! String) {
    throw FormatException('Data inválida ou ausente: $field');
  }
  return DateTime.parse(value).toUtc();
}

/// Payload enviado para criação de demanda em um Centro Acadêmico.
final class CreateDemandRequestDto {
  const CreateDemandRequestDto({
    required this.forum,
    required this.description,
    this.title,
    this.category,
    this.courseName,
  });

  final String forum;
  final String description;
  final String? title;
  final String? category;
  final String? courseName;

  Map<String, Object?> toJson() => {
    'forum': forum,
    'description': description,
    'title': ?title,
    'category': ?category,
    'courseName': ?courseName,
  };
}

/// Payload enviado para atualização de status de uma demanda pela gestão.
final class UpdateDemandStatusRequestDto {
  const UpdateDemandStatusRequestDto({
    required this.status,
    required this.opinion,
  });

  final String status;
  final String opinion;

  Map<String, Object?> toJson() => {'status': status, 'opinion': opinion};
}

/// Payload enviado para publicação de comentário em uma demanda.
final class CreateDemandCommentRequestDto {
  const CreateDemandCommentRequestDto({required this.content});

  final String content;

  Map<String, Object?> toJson() => {'content': content};
}

/// Registro do histórico de mudanças de status retornado pelo backend.
final class DemandStatusHistoryResponseDto {
  const DemandStatusHistoryResponseDto({
    required this.id,
    required this.demandId,
    required this.fromStatus,
    required this.toStatus,
    required this.changedById,
    required this.comment,
    required this.opinion,
    required this.createdAt,
  });

  factory DemandStatusHistoryResponseDto.fromJson(Map<String, dynamic> json) {
    return DemandStatusHistoryResponseDto(
      id: _parseUuid(json['id'], 'id'),
      demandId: _parseUuid(json['demandId'], 'demandId'),
      fromStatus: _parseOptionalString(json['fromStatus'], 'fromStatus'),
      toStatus: _parseRequiredString(json['toStatus'], 'toStatus'),
      changedById: _parseUuid(json['changedById'], 'changedById'),
      comment: _parseOptionalString(json['comment'], 'comment'),
      opinion: _parseOptionalString(json['opinion'], 'opinion'),
      createdAt: _parseDateTime(json['createdAt'], 'createdAt'),
    );
  }

  final UuidValue id;
  final UuidValue demandId;
  final String? fromStatus;
  final String toStatus;
  final UuidValue changedById;
  final String? comment;
  final String? opinion;
  final DateTime createdAt;
}

/// Representação de uma demanda retornada pelos endpoints HTTP.
final class DemandResponseDto {
  const DemandResponseDto({
    required this.id,
    required this.academicCenterId,
    required this.authorId,
    required this.forum,
    required this.title,
    required this.description,
    required this.category,
    required this.courseName,
    required this.status,
    required this.createdAt,
    required this.updatedAt,
    this.statusHistory,
    this.commentCount,
  });

  factory DemandResponseDto.fromJson(Map<String, dynamic> json) {
    final rawHistory = json['statusHistory'];
    List<DemandStatusHistoryResponseDto>? history;
    if (rawHistory != null) {
      if (rawHistory is! List) {
        throw const FormatException('Campo statusHistory inválido');
      }
      history = rawHistory.map((item) {
        if (item is! Map<String, dynamic>) {
          throw const FormatException('Entrada de histórico inválida');
        }
        return DemandStatusHistoryResponseDto.fromJson(item);
      }).toList();
    }

    final rawCommentCount = json['commentCount'];
    if (rawCommentCount is! int?) {
      throw const FormatException('Campo commentCount inválido');
    }

    return DemandResponseDto(
      id: _parseUuid(json['id'], 'id'),
      academicCenterId: _parseUuid(
        json['academicCenterId'],
        'academicCenterId',
      ),
      authorId: _parseUuid(json['authorId'], 'authorId'),
      forum: _parseRequiredString(json['forum'], 'forum'),
      title: _parseOptionalString(json['title'], 'title'),
      description: _parseRequiredString(json['description'], 'description'),
      category: _parseOptionalString(json['category'], 'category'),
      courseName: _parseOptionalString(json['courseName'], 'courseName'),
      status: _parseRequiredString(json['status'], 'status'),
      createdAt: _parseDateTime(json['createdAt'], 'createdAt'),
      updatedAt: _parseDateTime(json['updatedAt'], 'updatedAt'),
      statusHistory: history,
      commentCount: rawCommentCount,
    );
  }

  final UuidValue id;
  final UuidValue academicCenterId;
  final UuidValue authorId;
  final String forum;
  final String? title;
  final String description;
  final String? category;
  final String? courseName;
  final String status;
  final DateTime createdAt;
  final DateTime updatedAt;
  final List<DemandStatusHistoryResponseDto>? statusHistory;
  final int? commentCount;
}

/// Representação de um comentário ou resposta na linha do tempo da demanda.
final class DemandCommentResponseDto {
  const DemandCommentResponseDto({
    required this.id,
    required this.demandId,
    required this.authorId,
    required this.authorRole,
    required this.content,
    required this.createdAt,
  });

  factory DemandCommentResponseDto.fromJson(Map<String, dynamic> json) {
    return DemandCommentResponseDto(
      id: _parseUuid(json['id'], 'id'),
      demandId: _parseUuid(json['demandId'], 'demandId'),
      authorId: _parseUuid(json['authorId'], 'authorId'),
      authorRole: _parseRequiredString(json['authorRole'], 'authorRole'),
      content: _parseRequiredString(json['content'], 'content'),
      createdAt: _parseDateTime(json['createdAt'], 'createdAt'),
    );
  }

  final UuidValue id;
  final UuidValue demandId;
  final UuidValue authorId;
  final String authorRole;
  final String content;
  final DateTime createdAt;
}
