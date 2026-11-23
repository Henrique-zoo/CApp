import 'package:dio/dio.dart';

import '../application/election_failure.dart';

/// Interpreta falhas de transporte e códigos públicos recebidos da API.
ElectionFailure classifyElectionFailure(Object error) {
  if (error is FormatException ||
      (error is DioException && error.error is FormatException)) {
    return ElectionFailure.evidenceInvalid;
  }
  if (error is! DioException) return ElectionFailure.unknown;
  final status = error.response?.statusCode;
  final body = error.response?.data;
  final code = body is Map && body['error'] is Map
      ? body['error']['code']
      : null;
  return switch (code) {
    'already_voted' => ElectionFailure.alreadyVoted,
    'registration_closed' => ElectionFailure.registrationClosed,
    'registration_key_conflict' => ElectionFailure.keyConflict,
    'voter_not_eligible' => ElectionFailure.notEligible,
    'voting_ended' => ElectionFailure.votingEnded,
    'voting_not_started' => ElectionFailure.votingNotStarted,
    _ => switch (status) {
      401 => ElectionFailure.unauthorized,
      null || 408 || 429 || >= 500 => ElectionFailure.temporarilyUnavailable,
      _ => ElectionFailure.requestRejected,
    },
  };
}
