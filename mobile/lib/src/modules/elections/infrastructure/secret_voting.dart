import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:uuid/uuid.dart';

import '../../../infrastructure/backend/elections/ballot_api.dart';
import '../../../infrastructure/backend/elections/registration_api.dart';
import '../application/cryptography/elgamal.dart';
import 'voter_registration_client.dart';

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

Uint8List _bytes(Map<String, dynamic> payload) =>
    utf8.encode('CAPP-ELECTIONS/1\n${jsonEncode(_ordered(payload))}');
String _hash(Map<String, dynamic> payload) =>
    sha256.convert(_bytes(payload)).toString();

Future<Map<String, dynamic>> _verified(
  Object? envelope,
  String key,
  String kind,
  UuidValue election,
) async {
  if (envelope is Map<String, dynamic> && envelope.length == 2) {
    final payload = envelope['payload'];
    final signature = envelope['signature'];
    if (payload is Map<String, dynamic> &&
        signature is String &&
        payload['kind'] == kind &&
        payload['version'] == 1 &&
        payload['electionId'] == election.uuid &&
        await DeviceVoterKeys().verify(key, _bytes(payload), signature)) {
      return payload;
    }
  }
  throw FormatException('Assinatura ou contexto inválido: $kind');
}

/// O hash confiável do manifesto vem da comissão por canal independente.
final class SecretVotingClient {
  SecretVotingClient(
    this._api,
    this._registration,
    this._account,
    this._storage,
  );
  final BallotApi _api;
  final RegistrationApi _registration;
  final UuidValue? Function() _account;
  final SharedPreferencesAsync _storage;
  final _preparing = <String>{};

  Future<BallotSession> open(
    UuidValue election, {
    required String trustedManifestHash,
  }) async {
    final account =
        _account() ?? (throw StateError('Conta eleitoral indisponível'));
    void guard() {
      if (_account() != account) throw StateError('A conta ativa mudou');
    }

    final board = await _api.board(election);
    final envelope = board['manifest'];
    if (envelope is! Map || envelope['payload'] is! Map<String, dynamic>) {
      throw const FormatException('Manifesto ausente');
    }
    final payload = envelope['payload'] as Map<String, dynamic>;
    if (_hash(payload) != trustedManifestHash) {
      throw const FormatException('O manifesto diverge do hash da comissão');
    }
    final serverKey = payload['serverSigningKey'] as String;
    await _verified(envelope, serverKey, 'manifest', election);
    final options = (payload['options'] as List)
        .map((option) => UuidValue.withFormatValidation(option['id'] as String))
        .toList();
    if (options.length < 2 ||
        options.length > 51 ||
        options.toSet().length != options.length ||
        options.last.uuid != '00000000-0000-0000-0000-000000000000') {
      throw const FormatException('Opções do manifesto inválidas');
    }
    final encryptionKey = ElGamalPublicKey.fromValue(
      BigInt.parse(payload['combinedPublicKey'] as String),
      groupId: payload['groupId'] as String,
    );
    final status = await _registration.read(election, guard);
    if (status.publicKey == null || status.voterId == null) {
      throw StateError('O eleitor não cadastrou a chave antes da abertura');
    }
    return BallotSession._(
      this,
      election,
      DeviceVoterKeys.aliasFor(account, election),
      guard,
      trustedManifestHash,
      serverKey,
      encryptionKey,
      List.unmodifiable(options),
      status,
    );
  }
}

/// Contém exclusivamente o envelope cifrado e assinado, pronto para reenvio.
final class SignedVote {
  SignedVote._(this.json);
  final String json;
}

/// Recibo do servidor já verificado, acompanhado do índice e do hash para consulta no quadro.
final class AcceptedVote {
  AcceptedVote._(this.receiptJson, this.index, this.entryHash, this.recovered);
  final String receiptJson;
  final int index;
  final String entryHash;
  final bool recovered;
}

final class BallotSession {
  BallotSession._(
    this._client,
    this._election,
    this._alias,
    this._guard,
    this._manifestHash,
    this._serverKey,
    this._encryptionKey,
    this.optionIds,
    this._registration,
  );
  final SecretVotingClient _client;
  final UuidValue _election;
  final String _alias;
  final void Function() _guard;
  final String _manifestHash;
  final String _serverKey;
  final ElGamalPublicKey _encryptionKey;
  final List<UuidValue> optionIds;
  final VoterRegistration _registration;
  String get _draftKey => '$_alias.signed-ballot';

  /// Reabre o pacote cifrado da própria conta após reiniciar o aplicativo.
  Future<SignedVote?> savedVote() async {
    _guard();
    final raw = await _client._storage.getString(_draftKey);
    _guard();
    if (raw == null) return null;
    final vote = SignedVote._(raw);
    await _checkVote(vote);
    _guard();
    return vote;
  }

  Future<SignedVote> prepare(UuidValue choice) async {
    _guard();
    if (!_client._preparing.add(_draftKey)) {
      throw StateError('Já existe um preparo em andamento');
    }
    try {
      if (_registration.participated || await savedVote() != null) {
        throw StateError('Já existe uma cédula para esta conta e eleição');
      }
      final keys = DeviceVoterKeys();
      final key = await keys.publicKey(_alias, create: false);
      if (key == null || key != _registration.publicKey) {
        throw StateError('A chave cadastrada não está neste aparelho');
      }
      final pairs = encryptBallot(
        publicKey: _encryptionKey,
        orderedChoiceIds: optionIds,
        selectedChoiceId: choice,
      );
      final payload = {
        'version': 1,
        'kind': 'ballot',
        'electionId': _election.uuid,
        'manifestHash': _manifestHash,
        'voterId': _registration.voterId!.uuid,
        'submissionId': const Uuid().v4(),
        'publicKey': key,
        'ciphertexts': [
          for (var i = 0; i < pairs.length; i++)
            {
              'optionId': optionIds[i].uuid,
              'a': pairs[i].a.toString(),
              'b': pairs[i].b.toString(),
            },
        ],
      };
      _guard();
      final signature = await keys.sign(_alias, _bytes(payload));
      final vote = SignedVote._(
        jsonEncode({'payload': payload, 'signature': signature}),
      );
      _guard();
      await _client._storage.setString(_draftKey, vote.json);
      _guard();
      return vote;
    } finally {
      _client._preparing.remove(_draftKey);
    }
  }

  Future<Map<String, dynamic>> _checkVote(SignedVote vote) async {
    final payload = await _verified(
      jsonDecode(vote.json),
      _registration.publicKey!,
      'ballot',
      _election,
    );
    if (payload['manifestHash'] != _manifestHash ||
        payload['voterId'] != _registration.voterId!.uuid ||
        payload['publicKey'] != _registration.publicKey) {
      throw const FormatException('Cédula de outro eleitor ou manifesto');
    }
    return payload;
  }

  /// Repete o mesmo pacote após falha de transporte. O recibo só é aceito se
  /// as assinaturas, o índice e os hashes se referirem à própria submissão.
  Future<AcceptedVote> submit(SignedVote vote) async {
    _guard();
    final ballot = await _checkVote(vote);
    final response = await _client._api.submit(_election, vote.json, _guard);
    final entry = await _verified(
      response['entry'],
      _serverKey,
      'board-entry',
      _election,
    );
    final receipt = await _verified(
      response['receipt'],
      _serverKey,
      'receipt',
      _election,
    );
    final recorded = await _verified(
      entry['ballot'],
      _registration.publicKey!,
      'ballot',
      _election,
    );
    final ballotHash = _hash(ballot);
    final index = entry['index'];
    final entryHash = _hash(entry);
    if (entry['manifestHash'] != _manifestHash ||
        receipt['manifestHash'] != _manifestHash ||
        index is! int ||
        index < 1 ||
        index > 100000 ||
        (index == 1 && entry['previousHash'] != _manifestHash) ||
        receipt['index'] != index ||
        receipt['entryHash'] != entryHash ||
        _hash(recorded) != ballotHash ||
        receipt['ballotHash'] != ballotHash) {
      throw const FormatException(
        'O recibo ou registro não corresponde à cédula',
      );
    }
    _guard();
    return AcceptedVote._(
      jsonEncode(response['receipt']),
      index,
      entryHash,
      response['recovered'] as bool,
    );
  }

  /// Verifica a cadeia assinada do manifesto até o índice do recibo.
  Future<bool> isIncluded(AcceptedVote vote) async {
    final board = await _client._api.board(_election);
    final manifest = await _verified(
      board['manifest'],
      _serverKey,
      'manifest',
      _election,
    );
    if (_hash(manifest) != _manifestHash) {
      throw const FormatException('Manifesto do quadro alterado');
    }
    final entries = board['entries'] as List;
    if (entries.length < vote.index) return false;
    var previous = _manifestHash;
    for (var i = 0; i < vote.index; i++) {
      final entry = await _verified(
        entries[i],
        _serverKey,
        'board-entry',
        _election,
      );
      if (entry['index'] != i + 1 ||
          entry['manifestHash'] != _manifestHash ||
          entry['previousHash'] != previous) {
        return false;
      }
      previous = _hash(entry);
    }
    return previous == vote.entryHash;
  }
}
