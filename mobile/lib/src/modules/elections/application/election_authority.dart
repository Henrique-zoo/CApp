/// Operações de uma conta designada para a cerimônia do manifesto.
library;

import 'package:uuid/uuid.dart';

/// Atribuição registrada pelo CA para a eleição.
final class AuthorityAssignment {
  const AuthorityAssignment({
    required this.id,
    required this.isCommission,
    required this.slate,
    required this.registered,
    required this.startsAt,
    required this.endsAt,
    required this.cancelled,
    required this.manifestHash,
  });

  final UuidValue id;
  final bool isCommission;
  final UuidValue? slate;
  final bool registered;
  final DateTime startsAt;
  final DateTime endsAt;
  final bool cancelled;
  final String? manifestHash;
}

/// Prepara a chave local, publica o manifesto e participa da apuração.
abstract interface class AuthoritySession {
  Future<AuthorityAssignment> inspect();
  Future<AuthorityAssignment> register();
  Future<AuthorityAssignment> publish();
  Future<AuthorityAssignment> reschedule({
    required DateTime startsAt,
    required DateTime endsAt,
    required String reason,
  });
  Future<AuthorityAssignment> cancel(String reason);
  Future<void> observeCheckpoint(String trustedManifestHash);
  Future<void> contribute(String trustedManifestHash);
}
