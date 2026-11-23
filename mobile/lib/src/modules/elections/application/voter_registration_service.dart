/// Caso de uso que protege o vínculo entre uma sessão e sua chave eleitoral.
library;

import 'package:uuid/uuid.dart';

import 'ports/voter_key_store.dart';
import 'ports/voter_registration_gateway.dart';
import 'voter_registration.dart';

final class VoterRegistrationService {
  VoterRegistrationService(this._gateway, this._keys, this._activeAccount);

  final VoterRegistrationGateway _gateway;
  final VoterKeyStore _keys;
  final UuidValue? Function() _activeAccount;

  /// Consulta a situação da conta, inclusive após o encerramento da eleição.
  Future<VoterRegistration> status(UuidValue electionId) async {
    final guard = _guard(_requireAccount());
    final status = await _gateway.read(electionId, guard);
    guard();
    return status;
  }

  /// Cadastra a chave antes da abertura quando a eleição exigir. Em uma
  /// repetição, reutiliza o mesmo par. Se a chave pertence a outro dispositivo,
  /// informa [VoterRegistrationFailure.keyOnAnotherDevice].
  Future<VoterRegistration> register(UuidValue electionId) async {
    final accountId = _requireAccount();
    final guard = _guard(accountId);
    final status = await _gateway.read(electionId, guard);
    guard();
    if (!status.requiresRegistration) {
      throw VoterRegistrationFailure.registrationNotRequired;
    }

    final publicKey = await _keys.publicKey(
      accountId,
      electionId,
      create: status.publicKey == null,
    );
    guard();

    final registered = status.publicKey;
    if (registered != null) {
      if (registered != publicKey) {
        throw VoterRegistrationFailure.keyOnAnotherDevice;
      }
      return status;
    }
    if (publicKey == null) throw VoterRegistrationFailure.keyUnavailable;

    final proof = await _keys.prove(accountId, electionId, publicKey);
    guard();
    final accepted = await _gateway.register(proof, guard);
    guard();
    if (!accepted.requiresRegistration ||
        accepted.publicKey != publicKey ||
        accepted.voterId != accountId) {
      throw VoterRegistrationFailure.unexpectedResponse;
    }
    return accepted;
  }

  UuidValue _requireAccount() =>
      _activeAccount() ?? (throw VoterRegistrationFailure.accountUnavailable);

  void Function() _guard(UuidValue accountId) => () {
    if (_activeAccount() != accountId) {
      throw VoterRegistrationFailure.accountChanged;
    }
  };
}
