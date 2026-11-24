/// Contrato de acesso às operações remotas do módulo de FAQ.
///
/// Pertence à camada de aplicação para isolar o controlador de detalhes de
/// transporte HTTP, DTOs e bibliotecas de rede.
library;

import 'package:uuid/uuid.dart';

import '../../domain/entities/faq_entry.dart';

/// Porta de consulta, busca e gestão de perguntas frequentes por CA.
abstract interface class FaqGateway {
  /// Lista as perguntas frequentes do CA filtrado opcionalmente por categoria e termo.
  Future<List<FaqEntry>> listEntries({
    required UuidValue academicCenterId,
    String? category,
    String? query,
  });

  /// Lista as categorias temáticas distintas disponíveis no FAQ do CA.
  Future<List<String>> listCategories({required UuidValue academicCenterId});

  /// Consulta os detalhes de uma entrada individual do FAQ.
  Future<FaqEntry> getEntry({
    required UuidValue academicCenterId,
    required UuidValue entryId,
  });

  /// Cadastra uma nova pergunta e resposta no FAQ do CA.
  Future<FaqEntry> createEntry({
    required UuidValue academicCenterId,
    required FaqEntryDraft draft,
  });

  /// Atualiza uma pergunta frequente existente no CA.
  Future<FaqEntry> updateEntry({
    required UuidValue academicCenterId,
    required UuidValue entryId,
    required FaqEntryDraft draft,
  });

  /// Remove uma pergunta frequente existente no CA.
  Future<void> deleteEntry({
    required UuidValue academicCenterId,
    required UuidValue entryId,
  });
}
