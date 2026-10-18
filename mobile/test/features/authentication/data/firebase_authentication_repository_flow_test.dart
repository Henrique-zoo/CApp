import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:capp/features/authentication/data/firebase_authentication_repository.dart';
import 'package:capp/features/authentication/domain/authentication_repository.dart';
import 'package:dio/dio.dart';
import 'package:firebase_auth/firebase_auth.dart';
import 'package:flutter_test/flutter_test.dart';

class _FakeUser implements User {
  _FakeUser(this.idToken);

  final String? idToken;

  int idTokenCalls = 0;

  @override
  Future<String?> getIdToken([bool forceRefresh = false]) async {
    idTokenCalls++;
    return idToken;
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _FakeUserCredential implements UserCredential {
  const _FakeUserCredential(this.user);

  @override
  final User? user;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _FakeFirebaseAuth implements FirebaseAuth {
  _FakeFirebaseAuth({this.currentUser, this.signInError, this.signOutError});

  @override
  final User? currentUser;

  final Object? signInError;
  final Object? signOutError;

  int signInCalls = 0;
  int signOutCalls = 0;

  @override
  Future<UserCredential> signInWithProvider(AuthProvider provider) async {
    signInCalls++;

    if (signInError != null) throw signInError!;

    final user = currentUser;
    if (user == null) {
      throw StateError('O teste precisa configurar um usuário.');
    }

    return _FakeUserCredential(user);
  }

  @override
  Future<UserCredential> signInWithPopup(AuthProvider provider) async {
    return signInWithProvider(provider);
  }

  @override
  Future<void> signOut() async {
    signOutCalls++;

    if (signOutError != null) throw signOutError!;
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _FakeHttpClientAdapter implements HttpClientAdapter {
  _FakeHttpClientAdapter({required this.statusCode, required this.body});

  final int statusCode;
  final Object? body;

  RequestOptions? lastRequest;

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<Uint8List>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    lastRequest = options;

    return ResponseBody.fromString(
      jsonEncode(body),
      statusCode,
      headers: const {
        'content-type': ['application/json'],
      },
    );
  }

  @override
  void close({bool force = false}) {}
}

const _validIdentityResponse = {
  'uid': 'firebase-user-test',
  'email': '232012345@aluno.unb.br',
  'email_verified': true,
};

FirebaseAuthenticationRepository _createRepository({
  required _FakeFirebaseAuth firebaseAuth,
  required _FakeHttpClientAdapter adapter,
}) {
  final client = Dio();
  client.httpClientAdapter = adapter;

  return FirebaseAuthenticationRepository(
    firebaseAuth: firebaseAuth,
    client: client,
    config: const AuthenticationConfig(
      apiBaseUrl: 'https://api.capp.test',
      microsoftTenantId: 'test-tenant-id',
    ),
  );
}

void main() {
  group('FirebaseAuthenticationRepository flow', () {
    test('sends the Firebase ID token and returns backend identity', () async {
      final user = _FakeUser('firebase-id-token');
      final auth = _FakeFirebaseAuth(currentUser: user);
      final adapter = _FakeHttpClientAdapter(
        statusCode: 200,
        body: _validIdentityResponse,
      );
      final repository = _createRepository(
        firebaseAuth: auth,
        adapter: adapter,
      );

      final identity = await repository.signIn();

      expect(identity.uid, 'firebase-user-test');
      expect(identity.email, '232012345@aluno.unb.br');
      expect(identity.emailVerified, isTrue);
      expect(auth.signInCalls, 1);
      expect(user.idTokenCalls, 1);
      expect(adapter.lastRequest?.uri.path, '/identity/me');

      final authorizationHeader = adapter.lastRequest!.headers.entries
          .where((entry) => entry.key.toLowerCase() == 'authorization')
          .single
          .value;

      expect(authorizationHeader, 'Bearer firebase-id-token');
    });

    test('rejects a backend 401 and clears the Firebase session', () async {
      final auth = _FakeFirebaseAuth(
        currentUser: _FakeUser('firebase-id-token'),
      );
      final adapter = _FakeHttpClientAdapter(
        statusCode: 401,
        body: {'error': 'unauthorized'},
      );
      final repository = _createRepository(
        firebaseAuth: auth,
        adapter: adapter,
      );

      await expectLater(
        repository.signIn(),
        throwsA(
          isA<AuthenticationFailure>().having(
            (failure) => failure.reason,
            'reason',
            AuthenticationFailureReason.institutionalAccountRejected,
          ),
        ),
      );

      expect(auth.signOutCalls, 1);
    });

    test('maps a backend 503 to service unavailable', () async {
      final auth = _FakeFirebaseAuth(
        currentUser: _FakeUser('firebase-id-token'),
      );
      final adapter = _FakeHttpClientAdapter(
        statusCode: 503,
        body: {'error': 'unavailable'},
      );
      final repository = _createRepository(
        firebaseAuth: auth,
        adapter: adapter,
      );

      await expectLater(
        repository.signIn(),
        throwsA(
          isA<AuthenticationFailure>().having(
            (failure) => failure.reason,
            'reason',
            AuthenticationFailureReason.serviceUnavailable,
          ),
        ),
      );

      expect(auth.signOutCalls, 1);
    });

    test('returns null when no Firebase session exists', () async {
      final auth = _FakeFirebaseAuth();
      final adapter = _FakeHttpClientAdapter(
        statusCode: 200,
        body: _validIdentityResponse,
      );
      final repository = _createRepository(
        firebaseAuth: auth,
        adapter: adapter,
      );

      final identity = await repository.restoreSession();

      expect(identity, isNull);
      expect(adapter.lastRequest, isNull);
    });

    test('restores a session only after backend confirmation', () async {
      final auth = _FakeFirebaseAuth(
        currentUser: _FakeUser('firebase-id-token'),
      );
      final adapter = _FakeHttpClientAdapter(
        statusCode: 200,
        body: _validIdentityResponse,
      );
      final repository = _createRepository(
        firebaseAuth: auth,
        adapter: adapter,
      );

      final identity = await repository.restoreSession();

      expect(identity?.uid, 'firebase-user-test');
      expect(identity?.emailVerified, isTrue);
      expect(adapter.lastRequest?.uri.path, '/identity/me');
    });

    test('rejects an empty ID token and clears the session', () async {
      final auth = _FakeFirebaseAuth(currentUser: _FakeUser(''));
      final adapter = _FakeHttpClientAdapter(
        statusCode: 200,
        body: _validIdentityResponse,
      );
      final repository = _createRepository(
        firebaseAuth: auth,
        adapter: adapter,
      );

      await expectLater(
        repository.signIn(),
        throwsA(
          isA<AuthenticationFailure>().having(
            (failure) => failure.reason,
            'reason',
            AuthenticationFailureReason.institutionalAccountRejected,
          ),
        ),
      );

      expect(adapter.lastRequest, isNull);
      expect(auth.signOutCalls, 1);
    });

    test('does not accept an unconfirmed identity response', () async {
      final auth = _FakeFirebaseAuth(
        currentUser: _FakeUser('firebase-id-token'),
      );
      final adapter = _FakeHttpClientAdapter(
        statusCode: 200,
        body: {'uid': '', 'email': null, 'email_verified': true},
      );
      final repository = _createRepository(
        firebaseAuth: auth,
        adapter: adapter,
      );

      await expectLater(
        repository.signIn(),
        throwsA(
          isA<AuthenticationFailure>().having(
            (failure) => failure.reason,
            'reason',
            AuthenticationFailureReason.serviceUnavailable,
          ),
        ),
      );

      expect(auth.signOutCalls, 1);
    });

    test('maps Microsoft login cancellation and clears the session', () async {
      final auth = _FakeFirebaseAuth(
        signInError: FirebaseAuthException(code: 'popup-closed-by-user'),
      );
      final adapter = _FakeHttpClientAdapter(
        statusCode: 200,
        body: _validIdentityResponse,
      );
      final repository = _createRepository(
        firebaseAuth: auth,
        adapter: adapter,
      );

      await expectLater(
        repository.signIn(),
        throwsA(
          isA<AuthenticationFailure>().having(
            (failure) => failure.reason,
            'reason',
            AuthenticationFailureReason.cancelled,
          ),
        ),
      );

      expect(auth.signOutCalls, 1);
      expect(adapter.lastRequest, isNull);
    });

    test('reports cleanup failure when clearing the session fails', () async {
      final auth = _FakeFirebaseAuth(
        signInError: StateError('provider failure'),
        signOutError: StateError('cleanup failure'),
      );
      final adapter = _FakeHttpClientAdapter(
        statusCode: 200,
        body: _validIdentityResponse,
      );
      final repository = _createRepository(
        firebaseAuth: auth,
        adapter: adapter,
      );

      await expectLater(
        repository.signIn(),
        throwsA(
          isA<AuthenticationFailure>().having(
            (failure) => failure.reason,
            'reason',
            AuthenticationFailureReason.sessionOperationFailed,
          ),
        ),
      );

      expect(auth.signOutCalls, 1);
      expect(adapter.lastRequest, isNull);
    });

  });
}
