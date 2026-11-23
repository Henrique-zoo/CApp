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
      'alias': _alias(accountId, electionId),
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
    final signature = await _invoke('sign', {
      'alias': _alias(accountId, electionId),
      'message': message,
    });
    if (signature == null) throw VoterRegistrationFailure.keyUnavailable;

    return VoterRegistrationProof(
      electionId: electionId,
      accountId: accountId,
      publicKey: publicKey,
      signature: _base64Url(signature),
    );
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

  static String _alias(UuidValue accountId, UuidValue electionId) =>
      'capp.elections.v1.${accountId.uuid.toLowerCase()}.'
      '${electionId.uuid.toLowerCase()}';

  static String _base64Url(List<int> bytes) =>
      base64Url.encode(bytes).replaceAll('=', '');
}
