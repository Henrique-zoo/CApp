/// Cliente HTTP dos endpoints de perguntas frequentes (FAQ) do backend CApp.
///
/// Implementa as rotas definidas no OpenAPI sob `/api/v1/academic-centers/{id}/faq`.
/// Todas as operações solicitam autenticação Firebase e comprovação App Check
/// pelos interceptors compartilhados no [Dio].
library;

import 'package:dio/dio.dart';
import 'package:uuid/uuid.dart';

import 'dto/faq_entry_dto.dart';

/// Operações HTTP de consulta, busca e gestão do FAQ por Centro Acadêmico.
final class FaqApi {
  /// Cria o cliente com a instância compartilhada de [Dio].
  const FaqApi(this._dio);

  final Dio _dio;

  static const _protectedExtra = <String, Object?>{
    'firebaseAuth': true,
    'firebaseAppCheck': true,
  };

  /// Lista e pesquisa entradas de FAQ do Centro Acadêmico informado.
  ///
  /// Suporta filtro opcional por [category] e busca textual por [query]
  /// (enviada no parâmetro `q` aceito com o alias `search` pelo backend).
  Future<List<FaqEntryResponse>> listEntries({
    required UuidValue academicCenterId,
    String? category,
    String? query,
    CancelToken? cancelToken,
  }) async {
    final queryParameters = <String, dynamic>{};
    final trimmedCategory = category?.trim();
    if (trimmedCategory != null && trimmedCategory.isNotEmpty) {
      queryParameters['category'] = trimmedCategory;
    }
    final trimmedQuery = query?.trim();
    if (trimmedQuery != null && trimmedQuery.isNotEmpty) {
      queryParameters['q'] = trimmedQuery;
    }

    final response = await _dio.get<Object?>(
      '/api/v1/academic-centers/${academicCenterId.uuid}/faq',
      queryParameters: queryParameters.isEmpty ? null : queryParameters,
      options: Options(
        responseType: ResponseType.json,
        receiveDataWhenStatusError: true,
        validateStatus: (status) => status == 200,
        extra: _protectedExtra,
      ),
      cancelToken: cancelToken,
    );

    final data = response.data;
    if (data is! List) {
      throw const FormatException('Listagem de FAQ sem vetor JSON');
    }

    final entries = <FaqEntryResponse>[];
    for (final item in data) {
      if (item is! Map<String, dynamic>) {
        throw const FormatException('Item da listagem de FAQ inválido');
      }
      entries.add(FaqEntryResponse.fromJson(item));
    }
    return entries;
  }

  /// Lista as categorias temáticas distintas cadastradas no FAQ do CA.
  Future<List<String>> listCategories({
    required UuidValue academicCenterId,
    CancelToken? cancelToken,
  }) async {
    final response = await _dio.get<Object?>(
      '/api/v1/academic-centers/${academicCenterId.uuid}/faq/categories',
      options: Options(
        responseType: ResponseType.json,
        receiveDataWhenStatusError: true,
        validateStatus: (status) => status == 200,
        extra: _protectedExtra,
      ),
      cancelToken: cancelToken,
    );

    final data = response.data;
    if (data is! List) {
      throw const FormatException('Categorias de FAQ sem vetor JSON');
    }

    final categories = <String>[];
    for (final item in data) {
      if (item is! String) {
        throw const FormatException('Categoria de FAQ inválida');
      }
      categories.add(item);
    }
    return categories;
  }

  /// Consulta os detalhes de uma entrada individual do FAQ.
  Future<FaqEntryResponse> getEntry({
    required UuidValue academicCenterId,
    required UuidValue faqEntryId,
    CancelToken? cancelToken,
  }) async {
    final response = await _dio.get<Object?>(
      '/api/v1/academic-centers/${academicCenterId.uuid}/faq/${faqEntryId.uuid}',
      options: Options(
        responseType: ResponseType.json,
        receiveDataWhenStatusError: true,
        validateStatus: (status) => status == 200,
        extra: _protectedExtra,
      ),
      cancelToken: cancelToken,
    );

    final data = response.data;
    if (data is! Map<String, dynamic>) {
      throw const FormatException('Detalhe de FAQ sem objeto JSON');
    }

    return FaqEntryResponse.fromJson(data);
  }

  /// Cadastra uma nova pergunta e resposta no FAQ do CA (HTTP 201).
  Future<FaqEntryResponse> createEntry({
    required UuidValue academicCenterId,
    required CreateFaqEntryRequest request,
    CancelToken? cancelToken,
  }) async {
    final response = await _dio.post<Object?>(
      '/api/v1/academic-centers/${academicCenterId.uuid}/faq',
      data: request.toJson(),
      options: Options(
        contentType: Headers.jsonContentType,
        responseType: ResponseType.json,
        receiveDataWhenStatusError: true,
        validateStatus: (status) => status == 201,
        extra: _protectedExtra,
      ),
      cancelToken: cancelToken,
    );

    final data = response.data;
    if (data is! Map<String, dynamic>) {
      throw const FormatException('Resposta de criação de FAQ sem objeto JSON');
    }

    return FaqEntryResponse.fromJson(data);
  }

  /// Atualiza parcialmente uma pergunta e resposta do FAQ (HTTP 200).
  Future<FaqEntryResponse> updateEntry({
    required UuidValue academicCenterId,
    required UuidValue faqEntryId,
    required UpdateFaqEntryRequest request,
    CancelToken? cancelToken,
  }) async {
    final response = await _dio.patch<Object?>(
      '/api/v1/academic-centers/${academicCenterId.uuid}/faq/${faqEntryId.uuid}',
      data: request.toJson(),
      options: Options(
        contentType: Headers.jsonContentType,
        responseType: ResponseType.json,
        receiveDataWhenStatusError: true,
        validateStatus: (status) => status == 200,
        extra: _protectedExtra,
      ),
      cancelToken: cancelToken,
    );

    final data = response.data;
    if (data is! Map<String, dynamic>) {
      throw const FormatException(
        'Resposta de atualização de FAQ sem objeto JSON',
      );
    }

    return FaqEntryResponse.fromJson(data);
  }

  /// Remove uma entrada do FAQ do CA (HTTP 204).
  Future<void> deleteEntry({
    required UuidValue academicCenterId,
    required UuidValue faqEntryId,
    CancelToken? cancelToken,
  }) async {
    await _dio.delete<Object?>(
      '/api/v1/academic-centers/${academicCenterId.uuid}/faq/${faqEntryId.uuid}',
      options: Options(
        receiveDataWhenStatusError: true,
        validateStatus: (status) => status == 204,
        extra: _protectedExtra,
      ),
      cancelToken: cancelToken,
    );
  }
}
