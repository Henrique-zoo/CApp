/// Representação canônica e verificação RSA das evidências eleitorais.
library;

import 'package:uuid/uuid.dart';

import '../device_voter_keys.dart';
import 'election_signing_input.dart';

export 'election_signing_input.dart';

Future<Map<String, dynamic>> verifyElectionEnvelope(
  Object? envelope,
  String key,
  String kind,
  UuidValue election,
) async {
  if (envelope is Map<String, dynamic> && envelope.length == 2) {
    final payload = envelope['payload'];
    final signature = envelope['signature'];
    if (payload is Map<String, dynamic> &&
        signature is String &&
        payload['kind'] == kind &&
        payload['version'] == 1 &&
        payload['electionId'] == election.uuid &&
        await DeviceVoterKeys().verify(
          key,
          electionSigningBytes(payload),
          signature,
        )) {
      return payload;
    }
  }
  throw FormatException('Assinatura ou contexto inválido: $kind');
}


/// Verifica o envelope da cédula pelo formato assinado v1, independente de DTO.
Future<Map<String, dynamic>> verifyBallotEnvelope(
  Object? envelope,
  String key,
  UuidValue election,
  String accountId,
) async {
  if (envelope is Map<String, dynamic> && envelope.length == 2 &&
      envelope.keys.toSet().containsAll({'payload', 'signature'})) {
    final payload = envelope['payload'];
    final signature = envelope['signature'];
    if (payload is Map<String, dynamic> && signature is String &&
        payload['electionId'] == election.uuid &&
        payload['accountId'] == accountId &&
        payload['publicKey'] == key &&
        await DeviceVoterKeys().verify(key, ballotSigningBytes(payload), signature)) {
      return payload;
    }
  }
  throw const FormatException('Cédula ou assinatura eleitoral inválida');
}
