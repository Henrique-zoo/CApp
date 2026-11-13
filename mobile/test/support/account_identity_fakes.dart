/// Dependências controladas do fluxo interno, compartilhadas por testes de aplicação e UI.
///
/// O harness monta o SessionController e o caso de uso reais. As portas registram
/// obtenção de credenciais, consultas e mutações, permitindo testar a separação
/// entre Firebase, perfil interno e sincronização Microsoft sem serviços remotos.
library;

import 'package:capp/src/modules/identity/identity.dart';
import 'package:capp/src/modules/identity/application/institutional_auth_service.dart';

import 'identity_fakes.dart';
export 'identity_fakes.dart' show testInternalUser;

final accountTestNow = DateTime.utc(2026, 11, 11);

const testProfileCompletion = ProfileCompletion(
  nickname: 'Apelido',
  currentSemester: 4,
  institutionEntrySemester: '2023.2',
);

class AccountIdentityHarness {
  AccountIdentityHarness({
    String? uid = 'a',
    String? institutionalEmail,
    String? institutionalName = 'Pessoa Exemplo',
    bool student = false,
    String? courseName,
  }) : auth = FakeAuthSession(uid) {
    accounts = FakeAccountAuthentication(auth);
    proof = testVerification(
      expiresAt: accountTestNow.add(const Duration(days: 120)),
      email: institutionalEmail,
      name: institutionalName,
      student: student,
      courseName: courseName,
    );
    identity = FakeIdentityGateway(proof);
    gateway = FakeAccountGateway(testInternalUser(), proof);
    identity.userProvider = () => gateway.user;
    identity.resolveUid = () => auth.currentUserId;
    gateway.proofProvider = () => identity.verifiedByUid[auth.currentUserId];
    ms = FakeMicrosoftSignIn(auth);
    controller = SessionController(
      authSession: auth,
      sessionStore: store,
      authService: InstitutionalAuthService(
        identityGateway: identity,
        microsoftSignIn: ms,
        authSession: auth,
      ),
      now: () => currentTime,
      accountAuthentication: accounts,
      accountGateway: gateway,
      accountProfileStore: users,
    );
  }

  DateTime currentTime = accountTestNow;
  final FakeAuthSession auth;
  final store = FakeSessionStore();
  final users = FakeAccountProfileStore();
  late final InstitutionalVerification proof;
  late final FakeIdentityGateway identity;
  late final FakeMicrosoftSignIn ms;
  late final FakeAccountAuthentication accounts;
  late final FakeAccountGateway gateway;
  late final SessionController controller;

  Future<void> start() async {
    controller.start();
    await flushSessionEvents();
  }

  Future<void> dispose() async {
    await controller.dispose();
    await auth.close();
  }
}
