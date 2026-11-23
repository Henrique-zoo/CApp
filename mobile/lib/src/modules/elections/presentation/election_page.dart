import 'package:flutter/material.dart';

import '../application/secret_voting.dart';
import 'election_flow.dart';

const electionTitle = 'Eleição do Centro Acadêmico';

class ElectionPage extends StatefulWidget {
  const ElectionPage({super.key, required this.flow});

  final ElectionFlow flow;

  @override
  State<ElectionPage> createState() => _ElectionPageState();
}

class _ElectionPageState extends State<ElectionPage> {
  ElectionOption? _selected;

  ElectionFlow get flow => widget.flow;

  String _label(ElectionOption option) =>
      option.isNull ? 'Voto nulo' : option.name;

  Future<void> _review(ElectionOption choice) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Confirmar seu voto?'),
        content: Text(
          'Você escolheu ${_label(choice)}. '
          'Depois do envio, seu voto não poderá ser alterado.',
        ),
        actions: [
          TextButton(
            autofocus: true,
            onPressed: () => Navigator.pop(dialogContext, false),
            child: const Text('Voltar e alterar'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(dialogContext, true),
            child: const Text('Confirmar e enviar'),
          ),
        ],
      ),
    );
    if (!mounted || confirmed != true) return;
    setState(() => _selected = null);
    await flow.send(choice);
  }

  String _openingTime(BuildContext context, DateTime date) {
    final local = date.toLocal();
    final strings = MaterialLocalizations.of(context);
    final time = strings.formatTimeOfDay(
      TimeOfDay.fromDateTime(local),
      alwaysUse24HourFormat: true,
    );
    // O CApp também funciona com a localidade inglesa do sistema.
    final day = local.day.toString().padLeft(2, '0');
    final month = local.month.toString().padLeft(2, '0');
    return '$day/$month/${local.year} às $time';
  }

  Widget _screen({
    required String title,
    String? body,
    Widget? content,
    String? action,
    VoidCallback? onPressed,
  }) => Column(
    children: [
      Expanded(
        child: ListView(
          padding: const EdgeInsets.all(20),
          children: [
            Semantics(
              header: true,
              liveRegion: true,
              child: Text(
                title,
                style: Theme.of(context).textTheme.headlineSmall,
              ),
            ),
            if (body != null) ...[const SizedBox(height: 12), Text(body)],
            if (content != null) ...[const SizedBox(height: 18), content],
          ],
        ),
      ),
      if (action != null)
        Padding(
          padding: const EdgeInsets.fromLTRB(20, 8, 20, 16),
          child: SizedBox(
            width: double.infinity,
            child: FilledButton(onPressed: onPressed, child: Text(action)),
          ),
        ),
    ],
  );

  Widget _back(String title, String body) => _screen(
    title: title,
    body: body,
    action: 'Voltar ao Início',
    onPressed: () => Navigator.pop(context),
  );

  Widget _receipt() {
    final checking = flow.step == ElectionStep.checking;
    final verified = flow.step == ElectionStep.included;
    return _screen(
      title: 'Voto registrado',
      body: electionTitle,
      content: Card(
        margin: EdgeInsets.zero,
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text('Recibo validado'),
              const SizedBox(height: 6),
              Text(
                'Registro nº ${flow.receipt!.index}',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 8),
              Semantics(
                liveRegion: true,
                child: Text(
                  checking
                      ? 'Conferindo a inclusão no quadro público…'
                      : verified
                      ? 'Inclusão verificada no quadro público.'
                      : flow.message!,
                ),
              ),
              if (!checking && !verified)
                TextButton(
                  onPressed: flow.checkAgain,
                  child: const Text('Conferir novamente'),
                ),
            ],
          ),
        ),
      ),
      action: 'Voltar ao Início',
      onPressed: () => Navigator.pop(context),
    );
  }

  Widget _ballot(ElectionInspection election) {
    final selected = election.options
        .where((option) => option.id == _selected?.id)
        .firstOrNull;
    return _screen(
      title: electionTitle,
      body: 'Escolha uma opção',
      content: RadioGroup<ElectionOption>(
        groupValue: selected,
        onChanged: (value) => setState(() => _selected = value),
        child: Column(
          children: [
            for (final option in election.options)
              Card.outlined(
                child: RadioListTile<ElectionOption>(
                  title: Text(_label(option)),
                  value: option,
                ),
              ),
          ],
        ),
      ),
      action: 'Votar',
      onPressed: selected == null ? null : () => _review(selected),
    );
  }

  Widget _overview(ElectionInspection election) {
    if (election.hasSavedReceipt || election.participated) {
      return _screen(title: 'Consultando recibo…');
    }
    if (election.phase == ElectionPhase.closed) {
      return _back(
        'Votação encerrada',
        'O prazo para enviar uma cédula terminou.',
      );
    }
    if (election.registered && !election.localKeyMatches) {
      return _back(
        'Use o aparelho habilitado',
        'A chave desta eleição está no celular usado na sua habilitação.',
      );
    }
    if (election.phase == ElectionPhase.preparation) {
      final opens = _openingTime(context, election.startsAt);
      if (election.registered) {
        return _back(
          'Você está habilitado para votar neste aparelho.',
          'A votação abre em $opens. Volte por aqui para votar.',
        );
      }
      return _screen(
        title: electionTitle,
        body: 'A votação abre em $opens.',
        content: const Text(
          'Use o celular com que você pretende votar. '
          'Ao se habilitar, sua chave de votação é criada neste aparelho. '
          'Só a chave pública é enviada para a eleição.',
        ),
        action: 'Habilitar para votar',
        onPressed: flow.enroll,
      );
    }
    if (!election.registered) {
      return _back(
        'O prazo de habilitação terminou.',
        'Sua conta não se habilitou nesta eleição antes da abertura.',
      );
    }
    return _ballot(election);
  }

  Widget _body() => switch (flow.step) {
    ElectionStep.loading => _screen(title: 'Consultando eleição…'),
    ElectionStep.registering => _screen(title: 'Habilitando este aparelho…'),
    ElectionStep.overview => _overview(flow.election!),
    ElectionStep.sending => _screen(
      title: 'Enviando voto…',
      body: 'Aguarde a confirmação do registro.',
    ),
    ElectionStep.pending => _screen(
      title: 'Envio sem confirmação',
      body: 'Retome o envio para conferir o registro da mesma cédula.',
      action: 'Retomar envio',
      onPressed: () => flow.send(),
    ),
    ElectionStep.unavailable => _screen(
      title: 'Não foi possível confirmar',
      body: flow.message,
      action: 'Atualizar',
      onPressed: flow.refresh,
    ),
    ElectionStep.checking ||
    ElectionStep.included ||
    ElectionStep.checkFailed => _receipt(),
  };

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: const Text('Eleição')),
    body: SafeArea(
      child: ListenableBuilder(
        listenable: flow,
        builder: (context, _) => _body(),
      ),
    ),
  );
}
