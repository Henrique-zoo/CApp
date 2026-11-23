/// Usa o armazenamento nativo para conservar o par de assinatura por conta e
/// eleição. O canal entrega a chave pública e a assinatura da prova.
library;

import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:uuid/uuid.dart';

import '../application/ports/voter_key_store.dart';
import '../application/voter_registration.dart';
import 'voter_registration_payload.dart';

final class DeviceVoterKeys implements VoterKeyStore {
  static const channel = MethodChannel('br.com.capp/elections/voter-keys');

  @override
  Future<String?> publicKey(
    UuidValue accountId,
    UuidValue electionId, {
    required bool create,
  }) async {
    final der = await _invoke('publicKey', {
      'alias': aliasFor(accountId, electionId),
      'create': create,
    });
    return der == null ? null : _base64Url(der);
  }

  @override
  Future<VoterRegistrationProof> prove(
    UuidValue accountId,
    UuidValue electionId,
    String publicKey,
  ) async {
    final payload = voterRegistrationPayload(accountId, electionId, publicKey);
    final message = utf8.encode('CAPP-ELECTIONS/1\n${jsonEncode(payload)}');
    final signature = await sign(accountId, electionId, message);

    return VoterRegistrationProof(
      electionId: electionId,
      accountId: accountId,
      publicKey: publicKey,
      signature: signature,
    );
  }

  /// Assina usando o par deste dispositivo para a conta e a eleição.
  Future<String> sign(
    UuidValue accountId,
    UuidValue electionId,
    Uint8List message,
  ) async {
    final signature = await _invoke('sign', {
      'alias': aliasFor(accountId, electionId),
      'message': message,
    });
    if (signature == null) throw VoterRegistrationFailure.keyUnavailable;
    return _base64Url(signature);
  }

  /// Verifica uma assinatura eleitoral usando somente a chave pública.
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
      if (_base64Url(bytes) != value) {
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

  Future<Uint8List?> _invoke(String method, Map<String, Object> args) async {
    try {
      return await channel.invokeMethod<Uint8List>(method, args);
    } on PlatformException catch (_, stack) {
      Error.throwWithStackTrace(VoterRegistrationFailure.keyUnavailable, stack);
    } on MissingPluginException catch (_, stack) {
      Error.throwWithStackTrace(VoterRegistrationFailure.keyUnavailable, stack);
    }
  }

  static String aliasFor(UuidValue accountId, UuidValue electionId) =>
      'capp.elections.v1.${accountId.uuid.toLowerCase()}.'
      '${electionId.uuid.toLowerCase()}';

  static String _base64Url(List<int> bytes) =>
      base64Url.encode(bytes).replaceAll('=', '');
}
