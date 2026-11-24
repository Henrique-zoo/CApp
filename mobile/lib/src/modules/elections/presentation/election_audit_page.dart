import 'package:flutter/material.dart';

import '../application/election_audit.dart';

/// Consulta e recálculo local das evidências do quadro publicadas após fechar.
class ElectionAuditPage extends StatefulWidget {
  const ElectionAuditPage({super.key, required this.audit});

  final RunElectionAudit audit;

  @override
  State<ElectionAuditPage> createState() => _ElectionAuditPageState();
}

class _ElectionAuditPageState extends State<ElectionAuditPage> {
  ElectionAudit? _report;
  String? _failure;
  bool _busy = false;
  final _bundle = TextEditingController();
  final _anchor = TextEditingController();

  @override
  void dispose() {
    _bundle.dispose();
    _anchor.dispose();
    super.dispose();
  }

  Future<void> _run(bool download, {String? bundle}) async {
    setState(() {
      _failure = null;
      _report = null;
      _busy = true;
    });
    try {
      final anchor = _anchor.text.trim();
      final result = await widget.audit(
        download,
        bundleJson: bundle,
        anchorJson: anchor.isEmpty ? null : anchor,
      );
      if (mounted) setState(() => _report = result);
    } on FormatException catch (error) {
      if (mounted) {
        setState(() => _failure = 'Evidência inválida. ${error.message}');
      }
    } catch (_) {
      if (mounted) {
        setState(
          () => _failure = download
              ? 'A apuração não está disponível para consulta.'
              : 'Não foi possível verificar uma cópia neste aparelho.',
        );
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: const Text('Auditoria eleitoral')),
    body: ListView(
      padding: const EdgeInsets.all(20),
      children: [
        FilledButton(
          onPressed: _busy ? null : () => _run(true),
          child: const Text('Baixar e verificar'),
        ),
        OutlinedButton(
          onPressed: _busy ? null : () => _run(false),
          child: const Text('Verificar cópia local'),
        ),
        ExpansionTile(
          title: const Text('Inserir evidências JSON'),
          children: [
            TextField(
              controller: _anchor,
              maxLines: 2,
              decoration: const InputDecoration(
                labelText: 'Recibo ou encerramento externo',
              ),
            ),
            TextField(
              controller: _bundle,
              maxLines: 4,
              decoration: const InputDecoration(
                labelText: 'Pacote com quadro e apuração',
              ),
            ),
            TextButton(
              onPressed: _busy
                  ? null
                  : () => _run(false, bundle: _bundle.text.trim()),
              child: const Text('Verificar pacote inserido'),
            ),
          ],
        ),
        if (_busy) const Center(child: CircularProgressIndicator()),
        if (_failure case final failure?) Text(failure),
        if (_report case final report?) ...[
          const SizedBox(height: 12),
          Text(switch (report.tally) {
            AuditedTally.valid => 'Contagens conferidas no aparelho',
            AuditedTally.incomplete =>
              'Apuração incompleta. Faltam contribuições.',
            AuditedTally.annulled => 'Apuração anulada por inconsistência',
            AuditedTally.cancelled => 'Eleição cancelada pela comissão',
          }),
          if (report.cancellationReason case final reason?) Text(reason),
          if (report.tally == AuditedTally.valid && report.totals != null)
            for (var i = 0; i < report.optionNames.length; i++)
              ListTile(
                title: Text(report.optionNames[i]),
                trailing: Text('${report.totals![i]}'),
              ),
          if (report.anchoredThrough > 0)
            Text('Recibo confirmado no registro ${report.anchoredThrough}.'),
          Text(
            report.finalClosureAnchored
                ? 'Encerramento conferido com a evidência externa.'
                : 'Completude inconclusiva sem um encerramento obtido por outro canal.',
          ),
          const Text(
            'O recálculo não prova a validade individual das escolhas cifradas.',
          ),
        ],
      ],
    ),
  );
}
