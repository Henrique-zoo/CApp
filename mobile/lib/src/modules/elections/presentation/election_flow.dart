import 'dart:async';

import 'package:flutter/foundation.dart';

import '../application/election_failure.dart';
import '../application/secret_voting.dart';

enum ElectionStep {
  loading,
  overview,
  registering,
  sending,
  pending,
  checking,
  included,
  checkFailed,
  unavailable,
}

/// Estado compartilhado pelo cartão do Início e a tela da mesma eleição.
class ElectionFlow extends ChangeNotifier {
  ElectionFlow(this.inspect, this.register, this.classifyFailure);

  final Future<ElectionInspection> Function() inspect;
  final Future<void> Function() register;
  final ElectionFailure Function(Object) classifyFailure;
  ElectionInspection? election;
  AcceptedVote? receipt;
  SignedVote? _saved;
  ElectionStep step = ElectionStep.loading;
  String? message;
  bool detailOpen = false;
  bool _disposed = false;
  bool _busy = false;
  bool _refreshQueued = false;
  Timer? _timer;

  void _set(ElectionStep next, [String? detail]) {
    step = next;
    message = detail;
    if (!_disposed) notifyListeners();
  }

  Future<void> _run(Future<void> Function() action) async {
    if (_disposed || _busy) return;
    _busy = true;
    try {
      await action();
    } finally {
      _busy = false;
      if (!_disposed && _refreshQueued) {
        _refreshQueued = false;
        unawaited(refresh());
      }
    }
  }

  Future<void> refresh() async {
    if (_busy) {
      _refreshQueued = true;
      return;
    }
    await _run(_inspect);
  }

  Future<void> _inspect() async {
    _set(ElectionStep.loading);
    try {
      final current = await inspect();
      if (_disposed) return;
      election = current;
      _timer?.cancel();
      final remaining = current.timeUntilNextPhase;
      if (remaining != null) {
        _timer = Timer(remaining, refresh);
      }
      final session = current.session;
      if (current.hasSavedReceipt || current.participated) {
        _saved = null;
        if (detailOpen) {
          receipt = await session!.recoverReceipt();
          if (!_disposed) await _check();
        } else {
          _set(ElectionStep.overview);
        }
        return;
      }
      receipt = null;
      _saved = await session?.savedVote();
      _set(
        _saved != null && current.phase == ElectionPhase.voting
            ? ElectionStep.pending
            : ElectionStep.overview,
      );
    } catch (error) {
      _set(ElectionStep.unavailable, _messageFor(classifyFailure(error)));
    }
  }

  Future<void> enroll() => _run(() async {
    final current = election!;
    if (current.phase != ElectionPhase.preparation ||
        current.registered ||
        current.hasSavedReceipt) {
      await _inspect();
      return;
    }
    _set(ElectionStep.registering);
    try {
      await register();
      if (!_disposed) await _inspect();
    } catch (error) {
      _set(ElectionStep.unavailable, _messageFor(classifyFailure(error)));
    }
  });

  /// Assina apenas após a confirmação. Um envio pendente reutiliza o pacote
  /// armazenado pelo serviço antes da primeira tentativa de transmissão.
  Future<void> send([ElectionOption? choice]) => _run(() async {
    final current = election!;
    final session = current.session;
    if (session == null ||
        current.participated ||
        current.hasSavedReceipt ||
        current.phase != ElectionPhase.voting ||
        (_saved == null && (!current.localKeyMatches || choice == null))) {
      await _inspect();
      return;
    }
    _set(ElectionStep.sending);
    try {
      final vote = _saved ?? await session.prepare(choice!.id);
      if (_disposed) return;
      _saved = vote;
      receipt = await session.submit(vote);
      if (!_disposed) await _check();
    } catch (error) {
      final failure = classifyFailure(error);
      if (_saved != null && failure == ElectionFailure.temporarilyUnavailable) {
        _set(ElectionStep.pending);
      } else {
        _set(ElectionStep.unavailable, _messageFor(failure));
      }
    }
  });

  Future<void> checkAgain() => _run(_check);

  Future<void> _check() async {
    _set(ElectionStep.checking);
    try {
      final found = await election!.session!.isIncluded(receipt!);
      _set(
        found ? ElectionStep.included : ElectionStep.checkFailed,
        found ? null : 'O quadro público não confirma a inclusão deste recibo.',
      );
    } catch (error) {
      _set(ElectionStep.checkFailed, _messageFor(classifyFailure(error)));
    }
  }

  String _messageFor(ElectionFailure failure) => switch (failure) {
    ElectionFailure.evidenceInvalid =>
      'As evidências da eleição não passaram na verificação.',
    ElectionFailure.alreadyVoted =>
      'Sua conta já votou. Atualize para consultar o recibo.',
    ElectionFailure.registrationClosed => 'O prazo de habilitação terminou.',
    ElectionFailure.keyConflict =>
      'A habilitação está vinculada a outro aparelho.',
    ElectionFailure.notEligible =>
      'Sua conta não está habilitada para esta eleição.',
    ElectionFailure.votingEnded => 'O período de votação terminou.',
    ElectionFailure.votingNotStarted => 'A votação ainda não começou.',
    ElectionFailure.unauthorized =>
      'Não foi possível confirmar a sessão da sua conta.',
    ElectionFailure.temporarilyUnavailable || ElectionFailure.requestRejected =>
      'Não foi possível consultar o servidor da eleição.',
    ElectionFailure.unknown =>
      'Não foi possível concluir a operação nesta conta e aparelho.',
  };

  @override
  void dispose() {
    _disposed = true;
    _timer?.cancel();
    super.dispose();
  }
}
