import 'package:flutter/material.dart';

import '../application/materials.dart';
import 'material_publication_form.dart';

/// Consulta e publicação na área autenticada.
class MaterialsPage extends StatefulWidget {
  const MaterialsPage({super.key, required this.gateway, required this.device});

  final MaterialsGateway gateway;
  final MaterialDevice device;

  @override
  State<MaterialsPage> createState() => _MaterialsPageState();
}

class _MaterialsPageState extends State<MaterialsPage> {
  late Future<List<MaterialCenter>> _centers;
  String? _caId;

  @override
  void initState() {
    super.initState();
    _centers = widget.gateway.centers();
  }

  void _refresh() => setState(() => _centers = widget.gateway.centers());

  @override
  Widget build(BuildContext context) => FutureBuilder<List<MaterialCenter>>(
    future: _centers,
    builder: (context, snapshot) {
      if (snapshot.connectionState != ConnectionState.done) {
        return const Center(child: CircularProgressIndicator());
      }
      if (snapshot.hasError) {
        return _Retry(
          message: materialMessage(
            snapshot.error,
            'Não foi possível carregar os Centros Acadêmicos.',
          ),
          onRetry: _refresh,
        );
      }
      final centers = snapshot.requireData;
      if (centers.isEmpty) {
        return _Retry(
          message: 'Nenhum Centro Acadêmico disponível para esta conta.',
          onRetry: _refresh,
        );
      }
      final center = centers.firstWhere(
        (item) => item.id == _caId,
        orElse: () => centers.first,
      );
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 12, 8, 8),
            child: Row(
              children: [
                Expanded(
                  child: InputDecorator(
                    decoration: const InputDecoration(
                      labelText: 'Centro Acadêmico',
                    ),
                    child: DropdownButtonHideUnderline(
                      child: DropdownButton<String>(
                        value: center.id,
                        isExpanded: true,
                        items: [
                          for (final item in centers)
                            DropdownMenuItem(
                              value: item.id,
                              child: Text(
                                item.name,
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                        ],
                        onChanged: (id) => setState(() => _caId = id),
                      ),
                    ),
                  ),
                ),
                IconButton(
                  tooltip: 'Atualizar Centros Acadêmicos',
                  onPressed: _refresh,
                  icon: const Icon(Icons.refresh),
                ),
              ],
            ),
          ),
          Expanded(
            child: _CenterMaterials(
              key: ValueKey(center),
              center: center,
              gateway: widget.gateway,
              device: widget.device,
            ),
          ),
        ],
      );
    },
  );
}

class _CenterMaterials extends StatefulWidget {
  const _CenterMaterials({
    super.key,
    required this.center,
    required this.gateway,
    required this.device,
  });

  final MaterialCenter center;
  final MaterialsGateway gateway;
  final MaterialDevice device;

  @override
  State<_CenterMaterials> createState() => _CenterMaterialsState();
}

class _CenterMaterialsState extends State<_CenterMaterials> {
  late Future<List<MaterialDiscipline>> _disciplines;
  List<PublishedMaterial>? _items;
  int? _nextOffset;
  Object? _listError;
  bool _loadingMore = false;
  String? _disciplineId;
  int _query = 0;
  String? _detailId;
  bool _publishing = false;

  @override
  void initState() {
    super.initState();
    _disciplines = widget.gateway.disciplines(widget.center.id);
    _load();
  }

  Future<void> _load({bool more = false}) async {
    final query = ++_query;
    final offset = more ? _nextOffset! : 0;
    setState(() {
      _listError = null;
      _loadingMore = more;
      if (!more) _items = null;
    });
    try {
      final page = await widget.gateway.list(
        widget.center.id,
        disciplineId: _disciplineId,
        offset: offset,
      );
      if (!mounted || query != _query) return;
      setState(() {
        final retained = more ? _items! : const <PublishedMaterial>[];
        final seen = {for (final item in retained) item.id};
        _items = [
          ...retained,
          ...page.items.where((item) => seen.add(item.id)),
        ];
        _nextOffset = page.nextOffset;
        _loadingMore = false;
      });
    } on Exception catch (error) {
      if (!mounted || query != _query) return;
      setState(() {
        _listError = error;
        _loadingMore = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) => FutureBuilder<List<MaterialDiscipline>>(
    future: _disciplines,
    builder: (context, snapshot) {
      if (snapshot.connectionState != ConnectionState.done) {
        return const Center(child: CircularProgressIndicator());
      }
      if (snapshot.hasError) {
        return _Retry(
          message: materialMessage(
            snapshot.error,
            'Não foi possível carregar as disciplinas.',
          ),
          onRetry: () => setState(
            () => _disciplines = widget.gateway.disciplines(widget.center.id),
          ),
        );
      }
      final disciplines = snapshot.requireData;
      if (_publishing) {
        return MaterialPublicationForm(
          center: widget.center,
          disciplines: disciplines,
          selectedDisciplineId: _disciplineId,
          gateway: widget.gateway,
          device: widget.device,
          onCancel: () => setState(() => _publishing = false),
          onPublished: (material) {
            setState(() {
              _publishing = false;
              _disciplineId = material.disciplineId;
              _detailId = material.id;
            });
            _load();
          },
        );
      }
      if (_detailId case final id?) {
        return _MaterialDetails(
          key: ValueKey(id),
          caId: widget.center.id,
          id: id,
          disciplines: disciplines,
          gateway: widget.gateway,
          device: widget.device,
          onBack: () => setState(() => _detailId = null),
        );
      }
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                if (disciplines.isEmpty)
                  const Text(
                    'Nenhuma disciplina disponível para organizar os materiais.',
                  )
                else
                  DropdownButtonFormField<String>(
                    key: ValueKey(_disciplineId),
                    initialValue: _disciplineId ?? '',
                    isExpanded: true,
                    decoration: const InputDecoration(labelText: 'Disciplina'),
                    items: [
                      const DropdownMenuItem(
                        value: '',
                        child: Text('Todas as disciplinas'),
                      ),
                      for (final item in disciplines)
                        DropdownMenuItem(
                          value: item.id,
                          child: Text(
                            item.name,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                    ],
                    onChanged: (value) {
                      _disciplineId = value == '' ? null : value;
                      _load();
                    },
                  ),
                if (widget.center.canPublish) ...[
                  const SizedBox(height: 8),
                  FilledButton.icon(
                    onPressed: disciplines.isEmpty
                        ? null
                        : () => setState(() => _publishing = true),
                    icon: const Icon(Icons.add),
                    label: const Text('Publicar material'),
                  ),
                  if (disciplines.isEmpty)
                    const Text(
                      'Cadastre uma disciplina para publicar neste Centro Acadêmico.',
                    ),
                ],
              ],
            ),
          ),
          Expanded(child: _buildList(disciplines)),
        ],
      );
    },
  );

  Widget _buildList(List<MaterialDiscipline> disciplines) {
    final items = _items;
    if (items == null) {
      if (_listError case final error?) {
        return _Retry(
          message: materialMessage(
            error,
            'Não foi possível carregar os materiais.',
          ),
          onRetry: () => _load(),
        );
      }
      return const Center(child: CircularProgressIndicator());
    }
    return RefreshIndicator(
      onRefresh: _load,
      child: ListView(
        physics: const AlwaysScrollableScrollPhysics(),
        padding: const EdgeInsets.only(bottom: 16),
        children: [
          if (items.isEmpty)
            const ListTile(
              title: Text('Nenhum material publicado nesta seleção.'),
            ),
          for (final item in items)
            ListTile(
              leading: Icon(
                item.content is MaterialLink
                    ? Icons.link
                    : Icons.picture_as_pdf_outlined,
              ),
              title: Text(item.title),
              subtitle: Text(_disciplineName(disciplines, item.disciplineId)),
              trailing: const Icon(Icons.chevron_right),
              onTap: () => setState(() => _detailId = item.id),
            ),
          if (_loadingMore)
            const Center(
              child: Padding(
                padding: EdgeInsets.all(12),
                child: CircularProgressIndicator(),
              ),
            ),
          if (_listError case final error?)
            ListTile(
              title: Text(
                materialMessage(
                  error,
                  'Não foi possível carregar mais materiais.',
                ),
              ),
            ),
          if (_nextOffset != null && !_loadingMore)
            Center(
              child: TextButton(
                onPressed: () => _load(more: true),
                child: Text(
                  _listError == null ? 'Carregar mais' : 'Tentar novamente',
                ),
              ),
            ),
        ],
      ),
    );
  }
}

class _MaterialDetails extends StatefulWidget {
  const _MaterialDetails({
    super.key,
    required this.caId,
    required this.id,
    required this.gateway,
    required this.device,
    required this.disciplines,
    required this.onBack,
  });

  final String caId;
  final String id;
  final MaterialsGateway gateway;
  final MaterialDevice device;
  final List<MaterialDiscipline> disciplines;
  final VoidCallback onBack;

  @override
  State<_MaterialDetails> createState() => _MaterialDetailsState();
}

class _MaterialDetailsState extends State<_MaterialDetails> {
  late Future<PublishedMaterial> _details;
  bool _busy = false;
  String? _message;

  @override
  void initState() {
    super.initState();
    _details = widget.gateway.details(widget.caId, widget.id);
  }

  Future<void> _access(MaterialContent content) async {
    if (_busy) return;
    setState(() {
      _busy = true;
      _message = null;
    });
    try {
      switch (content) {
        case MaterialLink(:final url):
          if (!await widget.device.openLink(url)) {
            throw const MaterialFailure('Não foi possível abrir o link.');
          }
        case MaterialPdf(:final name, :final sizeBytes):
          final data = await widget.gateway.download(widget.caId, widget.id);
          if (!mounted) return;
          if (data.length != sizeBytes) {
            throw const MaterialFailure('O PDF recebido está incompleto.');
          }
          final saved = await widget.device.savePdf(name, data);
          if (mounted && saved) setState(() => _message = 'PDF salvo.');
      }
    } on Exception catch (error) {
      if (mounted) {
        setState(
          () => _message = materialMessage(
            error,
            'Não foi possível acessar o conteúdo.',
          ),
        );
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      TextButton.icon(
        onPressed: widget.onBack,
        icon: const Icon(Icons.arrow_back),
        label: const Text('Materiais'),
      ),
      Expanded(
        child: FutureBuilder<PublishedMaterial>(
          future: _details,
          builder: (context, snapshot) {
            if (snapshot.connectionState != ConnectionState.done) {
              return const Center(child: CircularProgressIndicator());
            }
            if (snapshot.hasError) {
              return _Retry(
                message: materialMessage(
                  snapshot.error,
                  'Não foi possível carregar o material.',
                ),
                onRetry: () => setState(
                  () =>
                      _details = widget.gateway.details(widget.caId, widget.id),
                ),
              );
            }
            final item = snapshot.requireData;
            final content = item.content;
            return ListView(
              padding: const EdgeInsets.all(16),
              children: [
                Text(
                  item.title,
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
                Text(_disciplineName(widget.disciplines, item.disciplineId)),
                if (item.description.isNotEmpty)
                  Padding(
                    padding: const EdgeInsets.symmetric(vertical: 12),
                    child: Text(item.description),
                  ),
                if (content case MaterialPdf(:final name)) Text(name),
                if (content case MaterialLink(:final url))
                  SelectableText(url.toString()),
                const SizedBox(height: 12),
                FilledButton.icon(
                  onPressed: _busy ? null : () => _access(content),
                  icon: Icon(
                    content is MaterialLink
                        ? Icons.open_in_new
                        : Icons.download_outlined,
                  ),
                  label: Text(
                    _busy
                        ? 'Aguarde…'
                        : content is MaterialLink
                        ? 'Abrir link'
                        : 'Salvar PDF',
                  ),
                ),
                if (_message case final message?) Text(message),
              ],
            );
          },
        ),
      ),
    ],
  );
}

String _disciplineName(List<MaterialDiscipline> disciplines, String id) =>
    disciplines.where((item) => item.id == id).firstOrNull?.name ??
    'Disciplina indisponível';

String materialMessage(Object? error, String fallback) =>
    error is MaterialFailure ? error.message : fallback;

class _Retry extends StatelessWidget {
  const _Retry({required this.message, required this.onRetry});

  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) => Center(
    child: Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(message, textAlign: TextAlign.center),
          TextButton(onPressed: onRetry, child: const Text('Tentar novamente')),
        ],
      ),
    ),
  );
}
