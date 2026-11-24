/// Canonicaliza o payload e aplica a separação de domínio do protocolo eleitoral.
library;

import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';

const _domains = {
  'ballot': 'CAPP-ELECTIONS/BALLOT/1\u0000',
  'voter-registration': 'CAPP-ELECTIONS/REGISTRATION/1\u0000',
  'manifest': 'CAPP-ELECTIONS/MANIFEST/1\u0000',
  'board-entry': 'CAPP-ELECTIONS/BOARD-ENTRY/1\u0000',
  'receipt': 'CAPP-ELECTIONS/RECEIPT/1\u0000',
  'cancellation': 'CAPP-ELECTIONS/CANCELLATION/1\u0000',
  'closure': 'CAPP-ELECTIONS/CLOSURE/1\u0000',
  'aggregate': 'CAPP-ELECTIONS/AGGREGATE/1\u0000',
  'authority-registration': 'CAPP-ELECTIONS/AUTHORITY-REGISTRATION/1\u0000',
  'decryption-share': 'CAPP-ELECTIONS/DECRYPTION-SHARE/1\u0000',
  'result': 'CAPP-ELECTIONS/RESULT/1\u0000',
  'authority-key-proof': 'CAPP-ELECTIONS/AUTHORITY-KEY-PROOF/1\u0000',
  'decryption-proof': 'CAPP-ELECTIONS/DECRYPTION-PROOF/1\u0000',
};

// O protocolo v1 usa objetos, arrays, strings, nulos e inteiros pequenos.
// RFC 8785 ordena os nomes por unidades UTF-16 e preserva a ordem dos arrays.
Object? _ordered(Object? value) => switch (value) {
  Map<String, dynamic> map => {
    for (final k in map.keys.toList()..sort()) k: _ordered(map[k]),
  },
  List list => list.map(_ordered).toList(),
  null || String() || int() || bool() => value,
  _ => throw const FormatException('Valor fora do protocolo eleitoral v1'),
};

/// Produz os bytes usados pela assinatura, pelo digest ou pelo desafio da prova.
Uint8List electionSigningBytes(Map<String, dynamic> payload) {
  final domain = _domains[payload['kind']];
  if (domain == null || payload['version'] is! int || payload['version'] != 1) {
    throw const FormatException('Domínio criptográfico eleitoral inválido');
  }

  return utf8.encode('$domain${jsonEncode(_ordered(payload))}');
}

/// SHA-256 dos bytes do payload, sem a assinatura do envelope.
String electionDigest(Map<String, dynamic> payload) =>
    sha256.convert(electionSigningBytes(payload)).toString();


/// Mensagem da cédula: sem `kind`/`version`, com contexto de conta e eleição.
/// A API recebe um DTO mínimo; o quadro conserva este payload assinado.
Uint8List ballotSigningBytes(Map<String, dynamic> payload) {
  const domain = 'CAPP-ELECTIONS/BALLOT/1\u0000';
  const fields = {
    'electionId', 'accountId', 'manifestHash', 'publicKey', 'ciphertexts',
  };
  if (payload.keys.toSet().difference(fields).isNotEmpty ||
      fields.difference(payload.keys.toSet()).isNotEmpty) {
    throw const FormatException('Campos da cédula assinada inválidos');
  }
  return utf8.encode('$domain${jsonEncode(_ordered(payload))}');
}

/// SHA-256 do conteúdo assinado, sem a assinatura RSA-PSS probabilística.
String ballotDigest(Map<String, dynamic> payload) =>
    sha256.convert(ballotSigningBytes(payload)).toString();
