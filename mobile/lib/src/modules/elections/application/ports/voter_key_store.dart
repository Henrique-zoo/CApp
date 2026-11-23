/// Acesso ao par de assinatura e à prova de posse mantidos pelo dispositivo.
library;

import 'package:uuid/uuid.dart';

import '../voter_registration.dart';

abstract interface class VoterKeyStore {
  /// Recupera a chave da conta para a eleição. Com [create] habilitado, cria
  /// o par quando necessário e permite reutilizá-lo em nova tentativa.
  Future<String?> publicKey(
    UuidValue accountId,
    UuidValue electionId, {
    required bool create,
  });

  /// Assina a prova que vincula a [publicKey] à conta e à eleição.
  Future<VoterRegistrationProof> prove(
    UuidValue accountId,
    UuidValue electionId,
    String publicKey,
  );
}
