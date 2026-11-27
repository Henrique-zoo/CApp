import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../application/election_authority.dart';

/// Cerimônia e contribuição da autoridade designada para a eleição.
class ElectionAuthorityPage extends StatefulWidget {
  const ElectionAuthorityPage({
    super.key,
    required this.session,
    this.manifestHash = '',
  });

  final AuthoritySession session;
  final String manifestHash;

  @override
  State<ElectionAuthorityPage> createState() => _ElectionAuthorityPageState();
}

class _ElectionAuthorityPageState extends State<ElectionAuthorityPage> {
  late final _reference = TextEditingController(text: widget.manifestHash);
  AuthorityAssignment? _assignment;
  String? _status;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    _run(widget.session.inspect);
  }

  @override
  void dispose() {
    _reference.dispose();
    super.dispose();
  }

  Future<void> _run(
    Future<AuthorityAssignment> Function() action, [
    String? success,
  ]) async {
    setState(() {
      _busy = true;
      _status = null;
    });
    try {
      final assignment = await action();
      if (mounted) {
        setState(() {
          _assignment = assignment;
          _status = success;
        });
      }
    } on FormatException catch (error) {
      if (mounted) setState(() => _status = error.message);
    } catch (_) {
      if (mounted) {
        setState(
          () => _status = 'A operação não está disponível para esta conta e fase da eleição.',
        );
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _editElection(
    AuthorityAssignment assignment, {
    required bool cancel,
  }) async {
    final reason = TextEditingController();
    var start = assignment.startsAt.toLocal();
    var end = assignment.endsAt.toLocal();
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialog) => StatefulBuilder(
        builder: (dialog, update) {
          String format(DateTime time) {
            final labels = MaterialLocalizations.of(dialog);
            return '${labels.formatShortDate(time)} '
                '${labels.formatTimeOfDay(TimeOfDay.fromDateTime(time))}';
          }

          Future<DateTime?> select(DateTime value) async {
            final day = await showDatePicker(
              context: dialog,
              initialDate: value,
              firstDate: DateTime(value.year - 1),
              lastDate: DateTime(value.year + 5, 12, 31),
            );
            if (day == null || !dialog.mounted) return null;

            final time = await showTimePicker(
              context: dialog,
              initialTime: TimeOfDay.fromDateTime(value),
            );
            if (time == null) return null;
            return DateTime(
              day.year,
              day.month,
              day.day,
              time.hour,
              time.minute,
            );
          }

          return AlertDialog(
            title: Text(cancel ? 'Cancelar eleição?' : 'Alterar período'),
            content: SingleChildScrollView(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  if (cancel)
                    const Text('A eleição deixará de aceitar votos.')
                  else ...[
                    OutlinedButton(
                      onPressed:
                          assignment.manifestHash != null &&
                              !DateTime.now().isBefore(assignment.startsAt)
                          ? null
                          : () async {
                              final selected = await select(start);
                              if (selected != null && dialog.mounted) {
                                update(() => start = selected);
                              }
                            },
                      child: Text('Início • ${format(start)}'),
                    ),
                    OutlinedButton(
                      onPressed: () async {
                        final selected = await select(end);
                        if (selected != null && dialog.mounted) {
                          update(() => end = selected);
                        }
                      },
                      child: Text('Fim • ${format(end)}'),
                    ),
                  ],
                  TextField(
                    controller: reason,
                    decoration: const InputDecoration(labelText: 'Motivo'),
                    maxLength: 2000,
                    onChanged: (_) => update(() {}),
                  ),
                ],
              ),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(dialog, false),
                child: const Text('Voltar'),
              ),
              FilledButton(
                onPressed:
                    reason.text.trim().isEmpty ||
                        (!cancel &&
                            (!end.isAfter(assignment.endsAt) ||
                                start.isBefore(assignment.startsAt) ||
                                !end.isAfter(start)))
                    ? null
                    : () => Navigator.pop(dialog, true),
                child: Text(cancel ? 'Cancelar eleição' : 'Salvar período'),
              ),
            ],
          );
        },
      ),
    );
    final explanation = reason.text.trim();
    reason.dispose();
    if (confirmed != true || !mounted) return;

    if (cancel) {
      await _run(
        () => widget.session.cancel(explanation),
        'Eleição cancelada.',
      );
    } else {
      await _run(
        () => widget.session.reschedule(
          startsAt: start,
          endsAt: end,
          reason: explanation,
        ),
        'Período atualizado.',
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final assignment = _assignment;
    final hash = assignment?.manifestHash;
    return Scaffold(
      appBar: AppBar(title: const Text('Autoridade eleitoral')),
      body: ListView(
        padding: const EdgeInsets.all(20),
        children: [
          if (_status case final status?) Text(status),
          if (_busy) const LinearProgressIndicator(),
          if (assignment == null)
            OutlinedButton(
              onPressed: _busy ? null : () => _run(widget.session.inspect),
              child: const Text('Consultar designação'),
            )
          else ...[
            Text(
              assignment.isCommission
                  ? 'Representante da comissão'
                  : 'Representante de chapa',
            ),
            if (assignment.cancelled)
              const Text('Eleição cancelada.')
            else if (assignment.isCommission &&
                DateTime.now().isBefore(assignment.endsAt)) ...[
              OutlinedButton(
                onPressed: _busy
                    ? null
                    : () => _editElection(assignment, cancel: false),
                child: const Text('Alterar período'),
              ),
              TextButton(
                onPressed: _busy
                    ? null
                    : () => _editElection(assignment, cancel: true),
                child: const Text('Cancelar eleição'),
              ),
            ],
            if (!assignment.cancelled && !assignment.registered)
              FilledButton(
                onPressed: _busy
                    ? null
                    : () => _run(widget.session.register, 'Chave cadastrada.'),
                child: const Text('Cadastrar chave deste aparelho'),
              ),
            if (!assignment.cancelled &&
                assignment.isCommission &&
                assignment.registered &&
                hash == null)
              FilledButton(
                onPressed: _busy
                    ? null
                    : () =>
                          _run(widget.session.publish, 'Manifesto publicado.'),
                child: const Text('Publicar manifesto'),
              ),
            if (!assignment.cancelled && hash != null) ...[
              const SizedBox(height: 12),
              if (assignment.isCommission)
                OutlinedButton(
                  onPressed: () => Clipboard.setData(ClipboardData(text: hash)),
                  child: const Text('Copiar hash do manifesto'),
                ),
              TextField(
                controller: _reference,
                decoration: const InputDecoration(
                  labelText: 'Hash recebido da comissão',
                ),
                textInputAction: TextInputAction.done,
              ),
              OutlinedButton(
                onPressed: _busy
                    ? null
                    : () => _run(() async {
                        await widget.session.observeCheckpoint(
                          _reference.text.trim(),
                        );
                        return widget.session.inspect();
                      }, 'Checkpoint conferido.'),
                child: const Text('Guardar checkpoint'),
              ),
              FilledButton(
                onPressed: _busy
                    ? null
                    : () => _run(() async {
                        await widget.session.contribute(_reference.text.trim());
                        return widget.session.inspect();
                      }, 'Contribuição confirmada.'),
                child: const Text('Enviar contribuição após o encerramento'),
              ),
            ],
          ],
        ],
      ),
    );
  }
}
