import 'package:flutter/material.dart';
import 'package:uuid/uuid.dart';

import '../application/materials.dart';

class MaterialPublicationForm extends StatefulWidget {
  const MaterialPublicationForm({
    super.key,
    required this.center,
    required this.disciplines,
    required this.selectedDisciplineId,
    required this.gateway,
    required this.device,
    required this.onCancel,
    required this.onPublished,
  });

  final MaterialCenter center;
  final List<MaterialDiscipline> disciplines;
  final String? selectedDisciplineId;
  final MaterialsGateway gateway;
  final MaterialDevice device;
  final VoidCallback onCancel;
  final ValueChanged<PublishedMaterial> onPublished;

  @override
  State<MaterialPublicationForm> createState() =>
      _MaterialPublicationFormState();
}

enum _Kind { link, pdf }

class _MaterialPublicationFormState extends State<MaterialPublicationForm> {
  final _form = GlobalKey<FormState>();
  final _title = TextEditingController();
  final _description = TextEditingController();
  final _url = TextEditingController();
  String? _discipline;
  _Kind _kind = _Kind.link;
  PdfSelection? _file;
  String? _error;
  bool _busy = false;
  ({MaterialDraft draft, String? url, PdfSelection? pdf})? _lastAttempt;
  String? _requestId;

  @override
  void initState() {
    super.initState();
    _discipline = widget.selectedDisciplineId;
  }

  @override
  void dispose() {
    _title.dispose();
    _description.dispose();
    _url.dispose();
    super.dispose();
  }

  Future<void> _choosePdf() async {
    if (_busy) return;
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final file = await widget.device.choosePdf();
      if (mounted && file != null) setState(() => _file = file);
    } on Exception catch (error) {
      if (mounted) {
        setState(
          () => _error = error is MaterialFailure
              ? error.message
              : 'Não foi possível ler o PDF.',
        );
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _publish() async {
    if (_busy) return;
    if (!_form.currentState!.validate()) return;
    final file = _kind == _Kind.pdf ? _file : null;
    if (_kind == _Kind.pdf && file == null) {
      setState(() => _error = 'Selecione um PDF.');
      return;
    }
    final draft = (
      title: _title.text.trim(),
      description: _description.text.trim(),
      disciplineId: _discipline!,
    );
    final attempt = (
      draft: draft,
      url: file == null ? _url.text.trim() : null,
      pdf: file,
    );
    if (attempt != _lastAttempt) {
      _requestId = const Uuid().v4();
      _lastAttempt = attempt;
    }
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final result = await (file == null
          ? widget.gateway.publishLink(
              widget.center.id,
              draft,
              attempt.url!,
              _requestId!,
            )
          : widget.gateway.publishPdf(
              widget.center.id,
              draft,
              file,
              _requestId!,
            ));
      if (mounted) widget.onPublished(result);
    } on Exception catch (error) {
      if (mounted) {
        setState(
          () => _error = error is MaterialFailure
              ? error.message
              : 'Não foi possível publicar o material.',
        );
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) => Form(
    key: _form,
    child: ListView(
      padding: const EdgeInsets.fromLTRB(16, 4, 16, 24),
      children: [
        Align(
          alignment: Alignment.centerLeft,
          child: TextButton.icon(
            onPressed: _busy ? null : widget.onCancel,
            icon: const Icon(Icons.arrow_back),
            label: const Text('Materiais'),
          ),
        ),
        Text(
          'Publicar material',
          style: Theme.of(context).textTheme.headlineSmall,
        ),
        const SizedBox(height: 12),
        TextFormField(
          controller: _title,
          enabled: !_busy,
          maxLength: 200,
          decoration: const InputDecoration(labelText: 'Título'),
          validator: (text) =>
              text == null || text.trim().isEmpty ? 'Informe um título.' : null,
        ),
        TextFormField(
          controller: _description,
          enabled: !_busy,
          maxLength: 5000,
          minLines: 2,
          maxLines: 4,
          decoration: const InputDecoration(labelText: 'Descrição (opcional)'),
        ),
        DropdownButtonFormField<String>(
          initialValue: _discipline,
          isExpanded: true,
          decoration: const InputDecoration(labelText: 'Disciplina'),
          items: [
            for (final item in widget.disciplines)
              DropdownMenuItem(
                value: item.id,
                child: Text(item.name, overflow: TextOverflow.ellipsis),
              ),
          ],
          onChanged: _busy ? null : (value) => _discipline = value,
          validator: (value) =>
              value == null ? 'Selecione uma disciplina.' : null,
        ),
        const SizedBox(height: 12),
        DropdownButtonFormField<_Kind>(
          initialValue: _kind,
          decoration: const InputDecoration(labelText: 'Tipo de conteúdo'),
          items: [
            const DropdownMenuItem(
              value: _Kind.link,
              child: Text('Link externo'),
            ),
            if (widget.center.canUploadPdf)
              const DropdownMenuItem(value: _Kind.pdf, child: Text('PDF')),
          ],
          onChanged: _busy ? null : (value) => setState(() => _kind = value!),
        ),
        const SizedBox(height: 12),
        if (_kind == _Kind.link)
          TextFormField(
            controller: _url,
            enabled: !_busy,
            keyboardType: TextInputType.url,
            decoration: const InputDecoration(labelText: 'Endereço do link'),
            validator: (text) {
              final uri = Uri.tryParse(text?.trim() ?? '');
              return uri == null ||
                      uri.host.isEmpty ||
                      (!uri.isScheme('https') && !uri.isScheme('http'))
                  ? 'Informe um endereço HTTP ou HTTPS válido.'
                  : null;
            },
          )
        else
          OutlinedButton.icon(
            onPressed: _busy ? null : _choosePdf,
            icon: const Icon(Icons.attach_file),
            label: Text(_file?.name ?? 'Selecionar PDF (até 10 MiB)'),
          ),
        const SizedBox(height: 12),
        if (_error case final message?)
          Text(
            message,
            style: TextStyle(color: Theme.of(context).colorScheme.error),
          ),
        FilledButton(
          onPressed: _busy ? null : _publish,
          child: Text(_busy ? 'Aguarde…' : 'Publicar'),
        ),
      ],
    ),
  );
}
