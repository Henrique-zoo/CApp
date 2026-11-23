import 'package:uuid/uuid.dart';

enum ElectionPhase { preparation, voting, closed }

final class ElectionOption {
  const ElectionOption(this.id, this.name, {required this.isNull});
  final UuidValue id;
  final String name;
  final bool isNull;
}

/// Estado da eleição para a conta ativa, com o calendário da comissão.
final class ElectionInspection {
  ElectionInspection({
    required this.startsAt,
    required this.endsAt,
    required List<ElectionOption> options,
    required this.requiresRegistration,
    required this.registered,
    required this.participated,
    required this.localKeyMatches,
    required this.hasSavedReceipt,
    required this.session,
    required this._closed,
    required this._clock,
  }) : options = List.unmodifiable(options);

  final DateTime startsAt;
  final DateTime endsAt;
  final List<ElectionOption> options;
  final bool requiresRegistration;
  final bool registered;
  final bool participated;
  final bool localKeyMatches;

  /// Existe um recibo neste aparelho, independente do estado remoto.
  final bool hasSavedReceipt;
  final BallotSession? session;
  final bool _closed;
  final DateTime Function() _clock;

  /// Indica que o app pode preparar uma nova cédula para esta conta.
  bool get canPrepareVote =>
      !participated &&
      (!requiresRegistration || (registered && localKeyMatches));

  ElectionPhase get phase {
    final now = _clock();
    if (_closed || !now.isBefore(endsAt)) return ElectionPhase.closed;
    return now.isBefore(startsAt)
        ? ElectionPhase.preparation
        : ElectionPhase.voting;
  }

  Duration? get timeUntilNextPhase {
    final now = _clock();
    if (_closed || !now.isBefore(endsAt)) return null;
    return (now.isBefore(startsAt) ? startsAt : endsAt).difference(now);
  }
}

/// Envelope cifrado e assinado, preservado integralmente para reenvio.
final class SignedVote {
  const SignedVote(this.json);
  final String json;
}

/// Comprovante que a sessão valida na confirmação e na consulta de inclusão.
final class AcceptedVote {
  const AcceptedVote({
    required this.receiptJson,
    required this.index,
    required this.entryHash,
    required this.ballotHash,
    required this.recovered,
  });
  final String receiptJson;
  final int index;
  final String entryHash;
  final String ballotHash;
  final bool recovered;
}

/// Operações do eleitor no contexto de uma única conta, eleição e manifesto.
abstract interface class BallotSession {
  List<UuidValue> get optionIds;
  Future<SignedVote?> savedVote();
  Future<SignedVote> prepare(UuidValue choice);
  Future<AcceptedVote> submit(SignedVote vote);
  Future<AcceptedVote> recoverReceipt();
  Future<bool> isIncluded(AcceptedVote vote);
}
