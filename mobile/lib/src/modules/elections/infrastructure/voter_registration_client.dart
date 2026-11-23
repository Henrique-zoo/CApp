import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:uuid/uuid.dart';

import '../../../infrastructure/backend/elections/registration_api.dart';

/// Gera e usa a chave privada dentro do armazenamento nativo. O canal devolve
/// somente o DER SubjectPublicKeyInfo da chave pública ou a assinatura RSA-PSS.
final class DeviceVoterKeys {
  static const channel = MethodChannel('br.com.capp/elections/voter-keys');

  static String aliasFor(UuidValue account, UuidValue election) =>
      'capp.elections.v1.${account.uuid.toLowerCase()}.${election.uuid.toLowerCase()}';

  Future<bool> verify(
    String publicKey,
    Uint8List message,
    String signature,
  ) async {
    Uint8List decode(String value) {
      if (value.length > 1024 || !RegExp(r'^[A-Za-z0-9_-]+$').hasMatch(value)) {
        throw const FormatException('Base64url eleitoral inválido');
      }
      final bytes = base64Url.decode(base64Url.normalize(value));
      if (_encode(bytes) != value) {
        throw const FormatException('Base64url não canônico');
      }
      return bytes;
    }

    return await channel.invokeMethod<bool>('verify', {
          'publicKey': decode(publicKey),
          'message': message,
          'signature': decode(signature),
        }) ==
        true;
  }

  Future<String?> publicKey(String alias, {required bool create}) async {
    final der = await channel.invokeMethod<Uint8List>('publicKey', {
      'alias': alias,
      'create': create,
    });
    return der == null ? null : _encode(der);
  }

  Future<String> sign(String alias, Uint8List message) async {
    final signature = await channel.invokeMethod<Uint8List>('sign', {
      'alias': alias,
      'message': message,
    });
    if (signature == null) throw StateError('Assinatura indisponível');
    return _encode(signature);
  }

  static String _encode(List<int> bytes) =>
      base64Url.encode(bytes).replaceAll('=', '');
}

/// Ponto de entrada do cadastro pelo app. O ID da conta interna vem da sessão
/// ativa, e cada eleição possui um par independente no dispositivo.
final class VoterRegistrationClient {
  VoterRegistrationClient(this._api, this._account);

  final RegistrationApi _api;
  final UuidValue? Function() _account;
  final _keys = DeviceVoterKeys();

  UuidValue _requireAccount() =>
      _account() ?? (throw StateError('Conta eleitoral indisponível'));

  void Function() _guard(UuidValue account) => () {
    if (_account() != account) throw StateError('A conta ativa mudou');
  };

  /// Consulta a participação inclusive de outro dispositivo, sem criar chaves.
  Future<VoterRegistration> status(UuidValue electionId) =>
      _api.read(electionId, _guard(_requireAccount()));

  /// Reenvios usam o mesmo par, inclusive após reinicialização ou resposta
  /// perdida. Um cadastro pré-existente em outro aparelho não é substituído.
  Future<VoterRegistration> register(UuidValue electionId) async {
    final account = _requireAccount();
    final guard = _guard(account);
    final state = await _api.read(electionId, guard);
    final id = electionId.uuid.toLowerCase();
    final owner = account.uuid.toLowerCase();
    final alias = DeviceVoterKeys.aliasFor(account, electionId);
    final key = await _keys.publicKey(alias, create: state.publicKey == null);
    guard();
    if (state.publicKey != null) {
      if (state.publicKey != key) {
        throw StateError('A chave cadastrada pertence a outro dispositivo');
      }
      return state;
    }
    if (key == null) throw StateError('Chave eleitoral indisponível');

    // Os cinco campos ASCII estão na ordem lexicográfica de RFC 8785. O
    // payload contém somente UUIDs, base64url, um literal e um inteiro.
    final payload = <String, Object>{
      'accountId': owner,
      'electionId': id,
      'kind': 'voter-registration',
      'publicKey': key,
      'version': 1,
    };
    final bytes = utf8.encode('CAPP-ELECTIONS/1\n${jsonEncode(payload)}');
    final signature = await _keys.sign(alias, bytes);
    final result = await _api.register(electionId, {
      'payload': payload,
      'signature': signature,
    }, guard);
    if (result.publicKey != key || result.voterId == null) {
      throw const FormatException('O servidor não confirmou a chave enviada');
    }
    return result;
  }
}
