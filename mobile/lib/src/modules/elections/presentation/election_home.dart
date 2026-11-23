import 'dart:async';

import 'package:flutter/material.dart';

import '../application/election_failure.dart';
import '../application/secret_voting.dart';
import 'election_flow.dart';
import 'election_page.dart';

/// Entrada da eleição configurada no Início da conta já autenticada.
class ElectionHome extends StatefulWidget {
  const ElectionHome({
    super.key,
    required this.inspect,
    required this.register,
    required this.classifyFailure,
  });

  final Future<ElectionInspection> Function() inspect;
  final Future<void> Function() register;
  final ElectionFailure Function(Object) classifyFailure;

  @override
  State<ElectionHome> createState() => _ElectionHomeState();
}

class _ElectionHomeState extends State<ElectionHome>
    with WidgetsBindingObserver {
  late final ElectionFlow _flow;

  @override
  void initState() {
    super.initState();
    _flow = ElectionFlow(
      () => widget.inspect(),
      () => widget.register(),
      (error) => widget.classifyFailure(error),
    );
    WidgetsBinding.instance.addObserver(this);
    unawaited(_flow.refresh());
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) unawaited(_flow.refresh());
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _flow.dispose();
    super.dispose();
  }

  Future<void> _open() async {
    if (_flow.detailOpen) return;
    _flow.detailOpen = true;
    unawaited(_flow.refresh());
    await Navigator.of(
      context,
    ).push<void>(MaterialPageRoute(builder: (_) => ElectionPage(flow: _flow)));
    if (!mounted) return;
    _flow.detailOpen = false;
    unawaited(_flow.refresh());
  }

  String _summary() {
    final election = _flow.election;
    if (_flow.step == ElectionStep.unavailable) return 'Consulta indisponível';
    if (election == null) return 'Consultando eleição…';
    if (_flow.receipt != null) return 'Voto registrado';
    if (election.hasSavedReceipt) return 'Recibo disponível neste aparelho';
    if (election.participated) return 'Participação registrada';
    if (_flow.step == ElectionStep.pending) return 'Envio sem confirmação';
    return switch (election.phase) {
      ElectionPhase.preparation =>
        !election.requiresRegistration
            ? 'Aguardando abertura'
            : election.registered
            ? 'Habilitação confirmada'
            : 'Habilitação aberta',
      ElectionPhase.voting => 'Votação aberta',
      ElectionPhase.closed => 'Votação encerrada',
    };
  }

  String _action() {
    final election = _flow.election;
    if (_flow.step == ElectionStep.unavailable || election == null) {
      return 'Abrir eleição';
    }
    if (_flow.receipt != null ||
        election.hasSavedReceipt ||
        election.participated) {
      return 'Ver recibo';
    }
    if (_flow.step == ElectionStep.pending) return 'Retomar envio';
    if (election.phase == ElectionPhase.voting) {
      return election.canPrepareVote ? 'Votar' : 'Ver situação';
    }
    return election.phase == ElectionPhase.closed
        ? 'Ver situação'
        : 'Abrir eleição';
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: _flow,
    builder: (context, _) {
      final theme = Theme.of(context);
      return ListView(
        padding: const EdgeInsets.all(20),
        children: [
          Text('Eleições', style: theme.textTheme.headlineSmall),
          const SizedBox(height: 12),
          Card(
            margin: EdgeInsets.zero,
            child: Padding(
              padding: const EdgeInsets.all(18),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Semantics(
                    liveRegion: true,
                    child: Text(
                      _summary(),
                      style: TextStyle(color: theme.colorScheme.primary),
                    ),
                  ),
                  const SizedBox(height: 10),
                  Text(electionTitle, style: theme.textTheme.titleLarge),
                  const SizedBox(height: 16),
                  FilledButton(onPressed: _open, child: Text(_action())),
                ],
              ),
            ),
          ),
        ],
      );
    },
  );
}
