import 'dart:async';
import 'dart:typed_data';

import 'package:capp/src/app/app.dart';
import 'package:capp/src/infrastructure/backend/materials/materials_api.dart';
import 'package:capp/src/modules/identity/identity.dart';
import 'package:capp/src/modules/materials/infrastructure/backend_materials_gateway.dart';
import 'package:capp/src/modules/materials/materials.dart';
import 'package:capp/src/modules/materials/materials_widgets.dart';
import 'package:dio/dio.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../test/support/identity_fakes.dart';
import 'step_registry.dart';

const computingCenter = 'CA de Computação';
const mathCenter = 'CA de Matemática';

final _scenarios = Expando<MaterialsScenario>();

bool isMaterialsScenario(Set<String> tags) =>
    tags.contains('@material_viewing') ||
    tags.contains('@material_publication') ||
    tags.contains('@material_organization');

Future<void> runMaterialsScenario(
  WidgetTester tester,
  Future<void> Function() body,
) async {
  final scenario = MaterialsScenario(tester);
  _scenarios[tester] = scenario;
  try {
    await body();
  } finally {
    _scenarios[tester] = null;
    await scenario.close();
  }
}

typedef MaterialsAction = FutureOr<void> Function(
  MaterialsScenario scenario,
  RegExpMatch match,
);

StepDefinition materialsStep(String pattern, MaterialsAction action) =>
    StepDefinition(RegExp(pattern), (tester, match) async {
      final scenario = _scenarios[tester];
      if (scenario == null) {
        throw StateError('Cenário de materiais sem suporte inicializado.');
      }
      await action(scenario, match);
    });

final class MaterialsScenario {
  MaterialsScenario(this.tester)
    : _dio = Dio(
        BaseOptions(
          baseUrl: 'http://127.0.0.1:$_fixturePort',
          connectTimeout: const Duration(seconds: 5),
          receiveTimeout: const Duration(seconds: 20),
        ),
      ) {
    _dio.interceptors.add(
      InterceptorsWrapper(
        onRequest: (request, handler) {
          final key = request.headers['Idempotency-Key'];
          if (request.method == 'POST' && key is String) {
            publicationKeys.add(key);
          }
          handler.next(request);
        },
      ),
    );
  }

  static const _fixturePort = int.fromEnvironment(
    'MATERIALS_BDD_PORT',
    defaultValue: 38475,
  );

  final WidgetTester tester;
  final Dio _dio;
  final device = ScenarioMaterialDevice();

  late final BackendMaterialsGateway gateway = BackendMaterialsGateway(
    MaterialsApi(_dio),
    () {},
  );
  late Map<String, dynamic> _session;
  IdentityHarness? _identity;
  BackendReadinessController? _readiness;
  bool _mounted = false;
  bool _author = false;
  late String activeCenter;
  String? selectedDiscipline;
  String? lastTitle;
  final archivedIds = <String, String>{};
  final publicationKeys = <String>[];

  Future<void> reset(String center, {bool publication = false}) async {
    final response = await _dio.post<Map<String, dynamic>>(
      '/__fixture/reset',
      data: {'center': center, 'publication': publication},
    );
    _session = response.data!;
    activeCenter = center;
    _setRole(author: publication);
  }

  Future<void> withReader(Future<void> Function() body) async {
    final wasAuthor = _author;
    _setRole(author: false);
    try {
      await body();
    } finally {
      _setRole(author: wasAuthor);
    }
  }

  void _setRole({required bool author}) {
    _author = author;
    _dio.options.headers['Authorization'] =
        'Bearer ${_session[author ? 'author' : 'reader']}';
    _dio.options.headers['X-Firebase-AppCheck'] = _session['appCheck'];
  }

  Future<Response<Map<String, dynamic>>> change(
    String action, [
    Map<String, Object?> data = const {},
  ]) => _dio.post<Map<String, dynamic>>(
    '/__fixture/change',
    data: {'action': action, ...data},
  );

  Future<void> seed(
    String title, {
    String? center,
    String? discipline,
    bool pdf = false,
    bool archived = false,
  }) async {
    final response = await change('seed', {
      'center': center ?? activeCenter,
      'title': title,
      'discipline': discipline,
      'pdf': pdf,
      'archived': archived,
    });
    lastTitle = title;
    if (archived) archivedIds[title] = response.data!['id'] as String;
  }

  Future<void> addDiscipline(String name, {String? center}) async {
    await change('add_discipline', {
      'center': center ?? activeCenter,
      'name': name,
    });
  }

  Future<Uint8List> pdf([int? size]) async {
    final response = await _dio.get<List<int>>(
      '/__fixture/pdf',
      queryParameters: {'size': ?size},
      options: Options(responseType: ResponseType.bytes),
    );
    return Uint8List.fromList(response.data!);
  }

  Future<int?> downloadStatus(String caId, String materialId) async {
    final response = await _dio.get<List<int>>(
      '/academic-centers/$caId/materials/$materialId/file',
      options: Options(
        responseType: ResponseType.bytes,
        validateStatus: (_) => true,
      ),
    );
    return response.statusCode;
  }

  Future<MaterialCenter> center(String name) async =>
      (await gateway.centers()).singleWhere((center) => center.name == name);

  Future<MaterialDiscipline> discipline(
    String name, {
    String? centerName,
  }) async {
    final ca = await center(centerName ?? activeCenter);
    return (await gateway.disciplines(ca.id))
        .singleWhere((discipline) => discipline.name == name);
  }

  Future<void> _mount() async {
    if (_mounted) return;
    final identity = IdentityHarness(
      uid: _author ? 'fixture-author' : 'fixture-reader',
    );
    identity.store.results[identity.auth.currentUserId!] = SessionCacheFound(
      testVerification(student: true),
    );
    final readiness = BackendReadinessController(checkReadiness: () async {});
    _identity = identity;
    _readiness = readiness;
    identity.controller.start();
    await tester.pumpWidget(
      CApp(
        sessionController: identity.controller,
        backendReadinessController: readiness,
        materialsBuilder: (_) =>
            MaterialsPage(gateway: gateway, device: device),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.byType(CApp), findsOneWidget);
    _mounted = true;
  }

  Future<void> assumeReader() async {
    if (!_author) return;
    await _unmount();
    _setRole(author: false);
  }

  Future<void> _unmount() async {
    if (!_mounted) return;
    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pumpAndSettle();
    await _identity?.dispose();
    _readiness?.dispose();
    _mounted = false;
  }

  Future<void> close() async {
    try {
      await _unmount();
    } finally {
      _dio.close(force: true);
    }
  }

  Future<void> tap(Finder finder) async {
    await tester.ensureVisible(finder);
    await tester.tap(finder);
    await tester.pumpAndSettle();
  }

  Future<void> openMaterials() async {
    await _mount();
    if (find.byType(MaterialsPage).evaluate().isEmpty) {
      await tap(find.byIcon(Icons.school_outlined));
      expect(find.byType(MaterialsPage), findsOneWidget);
      await selectCenter(activeCenter);
    }
  }

  Future<void> backToList() async {
    await openMaterials();
    final back = find.widgetWithText(TextButton, 'Materiais');
    if (back.evaluate().isNotEmpty) await tap(back);
  }

  Future<void> leaveMaterials() async {
    await tap(find.byIcon(Icons.home_outlined));
    expect(find.byType(MaterialsPage), findsNothing);
    selectedDiscipline = null;
  }

  Finder get centerMenu => find.descendant(
    of: find.byWidgetPredicate(
      (widget) =>
          widget is InputDecorator &&
          widget.decoration.labelText == 'Centro Acadêmico',
    ),
    matching: find.byType(DropdownButton<String>),
  );

  Future<void> selectCenter(String name) async {
    final ca = await center(name);
    final dropdown = tester.widget<DropdownButton<String>>(centerMenu);
    if (dropdown.value != ca.id) {
      await selectOption(centerMenu, name);
    }
    expect(tester.widget<DropdownButton<String>>(centerMenu).value, ca.id);
    if (activeCenter != name) selectedDiscipline = null;
    activeCenter = name;
  }

  Finder get disciplineMenu =>
      find.widgetWithText(DropdownButtonFormField<String>, 'Disciplina');

  Future<void> selectOption(Finder field, String name) async {
    await tap(field);
    await tap(find.text(name).hitTestable().last);
  }

  Future<void> selectDiscipline(String name) async {
    await openMaterials();
    await selectOption(disciplineMenu, name);
    selectedDiscipline = name;
  }

  Future<void> openDetails(String title) async {
    await openMaterials();
    await tap(find.widgetWithText(ListTile, title));
    expect(find.text(title), findsOneWidget);
    lastTitle = title;
  }

  void expectListed(String title, {bool present = true}) {
    expect(
      find.widgetWithText(ListTile, title),
      present ? findsOneWidget : findsNothing,
    );
  }

  Future<void> openContent(String title, {required bool pdf}) async {
    await openDetails(title);
    await tap(find.text(pdf ? 'Salvar PDF' : 'Abrir link'));
  }

  Future<List<PublishedMaterial>> published({
    String? centerName,
    String? disciplineName,
  }) async {
    final ca = await center(centerName ?? activeCenter);
    final disciplineId = disciplineName == null
        ? null
        : (await discipline(disciplineName, centerName: centerName)).id;
    return (await gateway.list(ca.id, disciplineId: disciplineId)).items;
  }
}

/// Registra efeitos na fronteira do dispositivo sem abrir navegador ou seletor.
final class ScenarioMaterialDevice implements MaterialDevice {
  PdfSelection? input;
  PdfSelection? saved;
  Uri? opened;

  @override
  Future<PdfSelection?> choosePdf() async => input;

  @override
  Future<bool> savePdf(String name, Uint8List bytes) async {
    saved = (name: name, bytes: bytes);
    return true;
  }

  @override
  Future<bool> openLink(Uri uri) async {
    opened = uri;
    return true;
  }
}
