/// Coordena o cadastro facultativo da chave antes da votação.
///
/// A sessão identifica a conta, o armazenamento do dispositivo mantém o par
/// por eleição e o backend confirma qual chave pode assinar cédulas.
library;

import 'package:uuid/uuid.dart';

/// Situação da conta autenticada perante a eleição.
final class VoterRegistration {
  const VoterRegistration({
    required this.requiresRegistration,
    required this.participated,
    this.voterId,
    this.publicKey,
  });

  final bool requiresRegistration;
  final bool participated;
  final UuidValue? voterId;
  final String? publicKey;
}

/// Prova assinada pelo dispositivo com a chave a ser cadastrada.
final class VoterRegistrationProof {
  const VoterRegistrationProof({
    required this.electionId,
    required this.accountId,
    required this.publicKey,
    required this.signature,
  });

  final UuidValue electionId;
  final UuidValue accountId;
  final String publicKey;
  final String signature;
}

/// Falhas do fluxo que a apresentação pode tratar de forma específica.
enum VoterRegistrationFailure implements Exception {
  accountUnavailable,
  accountChanged,
  registrationNotRequired,
  registrationClosed,
  notEligible,
  electionNotFound,
  keyConflict,
  invalidProof,
  keyOnAnotherDevice,
  keyUnavailable,
  unexpectedResponse,
  unavailable,
}
