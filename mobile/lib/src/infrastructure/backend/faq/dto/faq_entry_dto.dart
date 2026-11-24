/// Contratos HTTP de transferência de dados (DTO) para o módulo de FAQ.
///
/// Espelha os schemas `FaqEntryResponse`, `CreateFaqEntryRequest` e
/// `UpdateFaqEntryRequest` definidos no OpenAPI do backend CApp.
/// Pertence à infraestrutura compartilhada e não interpreta permissões ou
/// regras de exibição.
library;

import 'package:uuid/uuid.dart';

/// Payload de requisição para cadastrar uma nova pergunta frequente.
final class CreateFaqEntryRequest {
  /// Cria o payload com pergunta, resposta e categoria opcional.
  const CreateFaqEntryRequest({
    required this.question,
    required this.answer,
    this.category,
  });

  /// Texto da pergunta frequente.
  final String question;

  /// Resposta oficial do Centro Acadêmico.
  final String answer;

  /// Categoria temática opcional.
  final String? category;

  /// Serializa os campos aceitos pelo backend em JSON camelCase.
  Map<String, dynamic> toJson() => <String, dynamic>{
    'question': question,
    'answer': answer,
    if (category != null) 'category': category,
  };
}

/// Payload de requisição para atualizar parcialmente uma pergunta frequente.
final class UpdateFaqEntryRequest {
  /// Cria o payload com os campos alterados da entrada de FAQ.
  const UpdateFaqEntryRequest({this.question, this.answer, this.category});

  /// Novo texto opcional da pergunta frequente.
  final String? question;

  /// Nova resposta opcional do Centro Acadêmico.
  final String? answer;

  /// Nova categoria temática opcional.
  final String? category;

  /// Serializa apenas os campos informados para envio no `PATCH`.
  Map<String, dynamic> toJson() => <String, dynamic>{
    if (question != null) 'question': question,
    if (answer != null) 'answer': answer,
    if (category != null) 'category': category,
  };
}

/// Representação HTTP de uma pergunta e resposta do FAQ retornada pela API.
final class FaqEntryResponse {
  /// Constrói o DTO já validado pelos tipos da camada de transporte.
  const FaqEntryResponse({
    required this.id,
    required this.academicCenterId,
    required this.question,
    required this.answer,
    required this.category,
    required this.createdBy,
    required this.updatedBy,
    required this.createdAt,
    required this.updatedAt,
  });

  /// Identificador único (UUID) da entrada de FAQ.
  final UuidValue id;

  /// Identificador único (UUID) do Centro Acadêmico responsável.
  final UuidValue academicCenterId;

  /// Texto da pergunta frequente.
  final String question;

  /// Resposta explicativa fornecida pelo Centro Acadêmico.
  final String answer;

  /// Categoria temática opcional.
  final String? category;

  /// Identificador do usuário que cadastrou a entrada.
  final UuidValue createdBy;

  /// Identificador do usuário que realizou a última alteração.
  final UuidValue updatedBy;

  /// Instante UTC de criação do registro.
  final DateTime createdAt;

  /// Instante UTC da última modificação do registro.
  final DateTime updatedAt;

  /// Desserializa o objeto JSON retornado pelos endpoints de FAQ.
  ///
  /// Lança [FormatException] quando campos obrigatórios estiverem ausentes ou
  /// possuírem tipos, UUIDs ou datas inválidos.
  factory FaqEntryResponse.fromJson(Map<String, dynamic> json) {
    final id = json['id'];
    final academicCenterId = json['academicCenterId'];
    final question = json['question'];
    final answer = json['answer'];
    final category = json['category'];
    final createdBy = json['createdBy'];
    final updatedBy = json['updatedBy'];
    final createdAt = json['createdAt'];
    final updatedAt = json['updatedAt'];

    if (id is! String ||
        academicCenterId is! String ||
        question is! String ||
        answer is! String ||
        (category != null && category is! String) ||
        createdBy is! String ||
        updatedBy is! String ||
        createdAt is! String ||
        updatedAt is! String) {
      throw const FormatException('Entrada de FAQ incompatível com o contrato');
    }

    return FaqEntryResponse(
      id: UuidValue.withFormatValidation(id),
      academicCenterId: UuidValue.withFormatValidation(academicCenterId),
      question: question,
      answer: answer,
      category: category as String?,
      createdBy: UuidValue.withFormatValidation(createdBy),
      updatedBy: UuidValue.withFormatValidation(updatedBy),
      createdAt: DateTime.parse(createdAt).toUtc(),
      updatedAt: DateTime.parse(updatedAt).toUtc(),
    );
  }
}
