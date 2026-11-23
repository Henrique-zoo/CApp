/// Transporte HTTP de cadastro eleitoral. Converte o JSON de resposta em DTO.
library;

import 'package:dio/dio.dart';
import 'package:uuid/uuid.dart';

/// Estado privado da conta autenticada naquela eleição.
final class VoterRegistrationResponse {
  const VoterRegistrationResponse._(
    this.requiresRegistration,
    this.voterId,
    this.publicKey,
    this.participated,
  );

  final bool requiresRegistration;
  final UuidValue? voterId;
  final String? publicKey;
  final bool participated;

  factory VoterRegistrationResponse.fromJson(Map<String, dynamic>? data) {
    if (data case {
      'requiresRegistration': final bool required,
      'voterId': null,
      'publicKey': null,
      'participated': false,
    }) {
      return VoterRegistrationResponse._(required, null, null, false);
    }
    if (data
        case {
          'requiresRegistration': final bool required,
          'voterId': final String id,
          'publicKey': final String key,
          'participated': final bool participated,
        }
        when key.isNotEmpty) {
      return VoterRegistrationResponse._(
        required,
        UuidValue.withFormatValidation(id),
        key,
        participated,
      );
    }
    throw const FormatException('Estado de cadastro eleitoral inválido');
  }
}

final class RegistrationApi {
  RegistrationApi(this._dio);

  final Dio _dio;

  Future<VoterRegistrationResponse> read(
    UuidValue electionId,
    void Function() guard,
  ) => _request(electionId, guard);

  Future<VoterRegistrationResponse> register(
    UuidValue electionId,
    Map<String, Object> envelope,
    void Function() guard,
  ) => _request(electionId, guard, envelope);

  Future<VoterRegistrationResponse> _request(
    UuidValue electionId,
    void Function() guard, [
    Map<String, Object>? envelope,
  ]) async {
    if (!Uri.parse(_dio.options.baseUrl).isScheme('https')) {
      throw StateError('O cadastro eleitoral exige HTTPS');
    }
    guard();
    final response = await _dio.request<Map<String, dynamic>>(
      '/elections/${electionId.uuid.toLowerCase()}/registration/me',
      data: envelope,
      options: Options(
        method: envelope == null ? 'GET' : 'PUT',
        followRedirects: false,
        extra: {
          'firebaseAuth': true,
          'firebaseAppCheck': true,
          'firebaseAuthGuard': guard,
        },
      ),
    );
    guard();
    return VoterRegistrationResponse.fromJson(response.data);
  }
}
