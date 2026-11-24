/// Converte o contrato HTTP de cadastro em resultados da aplicação eleitoral.
library;

import 'package:dio/dio.dart';
import 'package:uuid/uuid.dart';

import '../../../../infrastructure/backend/elections/registration_api.dart';
import '../../application/ports/voter_registration_gateway.dart';
import '../../application/voter_registration.dart';
import '../voter_registration_payload.dart';

final class BackendVoterRegistrationGateway
    implements VoterRegistrationGateway {
  BackendVoterRegistrationGateway(this._api);

  final RegistrationApi _api;

  @override
  Future<VoterRegistration> read(
    UuidValue electionId,
    void Function() ensureCurrentAccount,
  ) => _request(() => _api.read(electionId, ensureCurrentAccount));

  @override
  Future<VoterRegistration> register(
    VoterRegistrationProof proof,
    void Function() ensureCurrentAccount,
  ) => _request(
    () => _api.register(proof.electionId, {
      'payload': voterRegistrationRequestPayload(proof.publicKey),
      'signature': proof.signature,
    }, ensureCurrentAccount),
  );

  /// Traduz apenas os códigos do cadastro e as falhas do transporte.
  Future<VoterRegistration> _request(
    Future<VoterRegistrationResponse> Function() request,
  ) async {
    try {
      return _toModel(await request());
    } on DioException catch (error) {
      if (error.error case final VoterRegistrationFailure failure) {
        throw failure;
      }
      final data = error.response?.data;
      final code = switch (data) {
        {'error': {'code': final String code}} => code,
        _ => null,
      };
      throw switch (code) {
        'registration_not_required' =>
          VoterRegistrationFailure.registrationNotRequired,
        'registration_closed' => VoterRegistrationFailure.registrationClosed,
        'registration_key_conflict' => VoterRegistrationFailure.keyConflict,
        'voter_not_eligible' => VoterRegistrationFailure.notEligible,
        'election_not_found' => VoterRegistrationFailure.electionNotFound,
        'invalid_registration_proof' => VoterRegistrationFailure.invalidProof,
        _ => VoterRegistrationFailure.unavailable,
      };
    } on FormatException {
      throw VoterRegistrationFailure.unexpectedResponse;
    }
  }

  VoterRegistration _toModel(VoterRegistrationResponse response) =>
      VoterRegistration(
        requiresRegistration: response.requiresRegistration,
        participated: response.participated,
        voterId: response.voterId,
        publicKey: response.publicKey,
      );
}
