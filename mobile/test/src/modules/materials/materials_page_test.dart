import 'dart:typed_data';

import 'package:capp/src/modules/materials/materials.dart';
import 'package:capp/src/modules/materials/materials_widgets.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

const _computing = (
  id: 'computation',
  name: 'CA de Computação',
  canPublish: true,
  canUploadPdf: true,
);
const _math = (
  id: 'math',
  name: 'CA de Matemática',
  canPublish: false,
  canUploadPdf: false,
);
const _calculus = (id: 'calculus', name: 'Cálculo I');
const _physics = (id: 'physics', name: 'Física');
const _algebra = (id: 'algebra', name: 'Álgebra');

typedef _Entry = ({String caId, PublishedMaterial value});

class _Fixture implements MaterialsGateway, MaterialDevice {
  final pdf = Uint8List.fromList([1, 2, 3, 4, 5]);
  final List<String> attempts = [];
  final List<_Entry> entries = [];
  final List<Uri> opened = [];
  final List<PdfSelection> saved = [];
  bool failPdf = false;

  void addLink(
    MaterialCenter center,
    MaterialDiscipline discipline,
    String title,
  ) {
    _add(center.id, (
      title: title,
      description: 'Descrição de teste',
      disciplineId: discipline.id,
    ), MaterialLink(Uri.parse('https://exemplo.edu.br')));
  }

  @override
  Future<List<MaterialCenter>> centers() async => [_computing, _math];

  @override
  Future<List<MaterialDiscipline>> disciplines(String caId) async =>
      caId == _computing.id ? [_calculus, _physics] : [_algebra];

  @override
  Future<PublishedMaterialPage> list(
    String caId, {
    String? disciplineId,
    int offset = 0,
  }) async => (
    items: [
      for (final entry in entries)
        if (entry.caId == caId &&
            (disciplineId == null || entry.value.disciplineId == disciplineId))
          entry.value,
    ],
    nextOffset: null,
  );

  @override
  Future<PublishedMaterial> details(String caId, String id) async => entries
      .singleWhere((item) => item.caId == caId && item.value.id == id)
      .value;

  @override
  Future<Uint8List> download(String caId, String id) async {
    expect((await details(caId, id)).content, isA<MaterialPdf>());
    return pdf;
  }

  @override
  Future<PublishedMaterial> publishLink(
    String caId,
    MaterialDraft draft,
    String url,
    String requestId,
  ) async => _add(caId, draft, MaterialLink(Uri.parse(url)));

  @override
  Future<PublishedMaterial> publishPdf(
    String caId,
    MaterialDraft draft,
    PdfSelection file,
    String requestId,
  ) async {
    attempts.add(requestId);
    if (failPdf) {
      failPdf = false;
      throw const MaterialFailure(
        'Armazenamento de materiais temporariamente indisponível.',
      );
    }
    return _add(caId, draft, MaterialPdf(file.name, file.bytes.length));
  }

  PublishedMaterial _add(
    String caId,
    MaterialDraft draft,
    MaterialContent content,
  ) {
    final item = (
      id: entries.length.toString(),
      disciplineId: draft.disciplineId,
      title: draft.title,
      description: draft.description,
      content: content,
    );
    entries.add((caId: caId, value: item));
    return item;
  }

  @override
  Future<PdfSelection?> choosePdf() async => (name: 'resumo.pdf', bytes: pdf);

  @override
  Future<bool> openLink(Uri uri) async {
    opened.add(uri);
    return true;
  }

  @override
  Future<bool> savePdf(String name, Uint8List bytes) async {
    saved.add((name: name, bytes: bytes));
    return true;
  }
}

Future<void> _open(WidgetTester tester, _Fixture fixture) async {
  tester.view.devicePixelRatio = 1;
  tester.view.physicalSize = const Size(800, 1000);
  addTearDown(tester.view.resetDevicePixelRatio);
  addTearDown(tester.view.resetPhysicalSize);
  await tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: MaterialsPage(gateway: fixture, device: fixture),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

Future<void> _tap(WidgetTester tester, String text) async {
  await tester.tap(find.text(text).last);
  await tester.pumpAndSettle();
}

Future<void> _form(WidgetTester tester, String title) async {
  await _tap(tester, 'Publicar material');
  await tester.enterText(find.widgetWithText(TextFormField, 'Título'), title);
  await tester.enterText(
    find.widgetWithText(TextFormField, 'Descrição (opcional)'),
    'Para a prova',
  );
  await _tap(tester, 'Disciplina');
  await _tap(tester, _calculus.name);
}

void main() {
  testWidgets(
    'filtra por disciplina e isola a seleção e as capacidades de cada CA',
    (tester) async {
      final data = _Fixture()
        ..addLink(_computing, _calculus, 'Apostila I')
        ..addLink(_computing, _physics, 'Lista de Física')
        ..addLink(_math, _algebra, 'Apostila de Álgebra');

      await _open(tester, data);
      expect(find.text('Apostila I'), findsOneWidget);
      expect(find.text('Lista de Física'), findsOneWidget);
      expect(find.text('Apostila de Álgebra'), findsNothing);

      await _tap(tester, 'Todas as disciplinas');
      await _tap(tester, _calculus.name);
      expect(find.text('Apostila I'), findsOneWidget);
      expect(find.text('Lista de Física'), findsNothing);

      await _tap(tester, _computing.name);
      await _tap(tester, _math.name);
      expect(find.text('Apostila I'), findsNothing);
      expect(find.text('Apostila de Álgebra'), findsOneWidget);
      expect(find.text('Publicar material'), findsNothing);
    },
  );

  testWidgets('publica um link, exibe os detalhes e abre o endereço', (
    tester,
  ) async {
    final data = _Fixture();
    const url = 'https://exemplo.edu.br/referencias';
    await _open(tester, data);
    await _form(tester, 'Referências');
    await tester.enterText(
      find.widgetWithText(TextFormField, 'Endereço do link'),
      'sem-url',
    );
    await _tap(tester, 'Publicar');
    expect(
      find.text('Informe um endereço HTTP ou HTTPS válido.'),
      findsOneWidget,
    );
    expect(data.entries, isEmpty);

    await tester.enterText(
      find.widgetWithText(TextFormField, 'Endereço do link'),
      url,
    );
    await _tap(tester, 'Publicar');
    expect(find.text('Referências'), findsOneWidget);
    expect(find.text('Para a prova'), findsOneWidget);
    await _tap(tester, 'Abrir link');
    expect(data.opened, [Uri.parse(url)]);
  });

  testWidgets(
    'repete o envio após falha com a mesma chave e permite salvar o PDF',
    (tester) async {
      final data = _Fixture()..failPdf = true;
      await _open(tester, data);
      await _form(tester, 'Resumo');
      await _tap(tester, 'Link externo');
      await _tap(tester, 'PDF');
      await _tap(tester, 'Selecionar PDF (até 10 MiB)');
      expect(find.text('resumo.pdf'), findsOneWidget);

      await _tap(tester, 'Publicar');
      expect(
        find.text('Armazenamento de materiais temporariamente indisponível.'),
        findsOneWidget,
      );
      expect(data.entries, isEmpty);
      await _tap(tester, 'Publicar');
      expect(data.attempts.length, 2);
      expect(data.attempts[0], data.attempts[1]);
      expect(data.entries, hasLength(1));

      await _tap(tester, 'Materiais');
      await _tap(tester, 'Resumo');
      await _tap(tester, 'Salvar PDF');
      expect(data.saved.single.name, 'resumo.pdf');
      expect(data.saved.single.bytes, data.pdf);
      expect(find.text('PDF salvo.'), findsOneWidget);
    },
  );
}
