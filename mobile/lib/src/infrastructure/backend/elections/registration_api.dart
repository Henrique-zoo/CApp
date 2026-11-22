import 'package:dio/dio.dart';
import 'package:uuid/uuid.dart';

/// Estado privado da conta autenticada naquela eleição.
final class VoterRegistration {
  const VoterRegistration._(this.voterId, this.publicKey, this.participated);

  final UuidValue? voterId;
  final String? publicKey;
  final bool participated;

  factory VoterRegistration.fromJson(Map<String, dynamic>? data) {
    if (data case {'voterId': null, 'publicKey': null, 'participated': false}) {
      return const VoterRegistration._(null, null, false);
    }
    if (data
        case {
          'voterId': final String id,
          'publicKey': final String key,
          'participated': final bool participated,
        }
        when key.isNotEmpty) {
      return VoterRegistration._(
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

  Future<VoterRegistration> read(UuidValue electionId, void Function() guard) =>
      _request(electionId, guard);

  Future<VoterRegistration> register(
    UuidValue electionId,
    Map<String, Object> envelope,
    void Function() guard,
  ) => _request(electionId, guard, envelope);

  Future<VoterRegistration> _request(
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
    return VoterRegistration.fromJson(response.data);
  }
}
