import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:uuid/uuid.dart';

import '../../../infrastructure/backend/elections/ballot_api.dart';
import '../../../infrastructure/backend/elections/registration_api.dart';
import '../application/cryptography/elgamal.dart';
import '../application/secret_voting.dart';
import 'voter_registration_client.dart';

export '../application/secret_voting.dart';

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

ElectionOption _readOption(Object? value) {
  if (value case {'id': String id, 'name': String name, 'kind': String kind}) {
    const nil = '00000000-0000-0000-0000-000000000000';
    if (name.trim().isNotEmpty &&
        name.runes.length <= 200 &&
        ((kind == 'slate' && id != nil) ||
            (kind == 'null' && id == nil && name == 'Nulo'))) {
      return ElectionOption(
        UuidValue.withFormatValidation(id),
        name,
        isNull: kind == 'null',
      );
    }
  }
  throw const FormatException('Opção do manifesto inválida');
}

DateTime _utcTime(Object? value) {
  // O contrato usa UTC com quatro dígitos no ano e precisão de segundos.
  if (value is String && value.length == 20 && value.endsWith('Z')) {
    final time = DateTime.tryParse(value);
    if (time != null &&
        time.toIso8601String().replaceFirst('.000Z', 'Z') == value) {
      return time;
    }
  }
  throw const FormatException('Horário UTC inválido no calendário eleitoral');
}

/// O hash confiável do manifesto vem da comissão por canal independente.
final class SecretVotingClient {
  SecretVotingClient(
    this._api,
    this._registration,
    this._account,
    this._storage, {
    DateTime Function()? clock,
  }) : _clock = clock ?? DateTime.now;
  final BallotApi _api;
  final RegistrationApi _registration;
  final UuidValue? Function() _account;
  final SharedPreferencesAsync _storage;
  final DateTime Function() _clock;
  final _preparing = <String>{};

  Future<BallotSession> open(
    UuidValue election, {
    required String trustedManifestHash,
  }) async {
    final state = await inspect(
      election,
      trustedManifestHash: trustedManifestHash,
    );
    if (!state.registered) {
      throw StateError('O eleitor não cadastrou a chave antes da abertura');
    }
    return state.session!;
  }

  Future<ElectionInspection> inspect(
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
    final calendar = await _verified(
      board['calendarOrigin'],
      serverKey,
      'calendar-origin',
      election,
    );
    final calendarHash = _hash(calendar);
    if (calendarHash != payload['calendarOriginHash']) {
      throw const FormatException('Calendário divergente do manifesto');
    }
    final startsAt = _utcTime(calendar['startsAt']);
    final endsAt = _utcTime(calendar['endsAt']);
    if (!startsAt.isBefore(endsAt)) {
      throw const FormatException('Período de votação inválido');
    }
    final closed = board['closure'] != null;
    if (closed) {
      final closure = await _verified(
        board['closure'],
        serverKey,
        'closure',
        election,
      );
      if (closure['manifestHash'] != trustedManifestHash ||
          closure['scheduleHash'] != calendarHash) {
        throw const FormatException(
          'Encerramento divergente do manifesto ou calendário',
        );
      }
    }
    final options = (payload['options'] as List).map(_readOption).toList();
    final ids = options.map((o) => o.id).toList();
    if (options.length < 2 ||
        options.length > 51 ||
        ids.toSet().length != ids.length ||
        !options.last.isNull) {
      throw const FormatException('Opções do manifesto inválidas');
    }
    final encryptionKey = ElGamalPublicKey.fromValue(
      BigInt.parse(payload['combinedPublicKey'] as String),
      groupId: payload['groupId'] as String,
    );
    final status = await _registration.read(election, guard);
    final alias = DeviceVoterKeys.aliasFor(account, election);
    final hasSavedReceipt =
        await _storage.getString(_BallotSession._receiptKey(alias)) != null;
    final registered = status.voterId != null;
    final session = !registered && !hasSavedReceipt
        ? null
        : _BallotSession(
            this,
            election,
            alias,
            guard,
            trustedManifestHash,
            serverKey,
            encryptionKey,
            List.unmodifiable(ids),
            status,
          );
    final localKeyMatches =
        registered &&
        await DeviceVoterKeys().publicKey(alias, create: false) ==
            status.publicKey;
    guard();
    return ElectionInspection(
      startsAt: startsAt,
      endsAt: endsAt,
      options: options,
      registered: registered,
      participated: status.participated,
      localKeyMatches: localKeyMatches,
      hasSavedReceipt: hasSavedReceipt,
      session: session,
      closed: closed,
      clock: _clock,
    );
  }
}

final class _BallotSession implements BallotSession {
  _BallotSession(
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
  @override
  final List<UuidValue> optionIds;
  final VoterRegistration _registration;
  String get _draftKey => '$_alias.signed-ballot';
  static String _receiptKey(String alias) => '$alias.signed-receipt';

  /// Reabre o pacote cifrado da própria conta após reiniciar o aplicativo.
  @override
  Future<SignedVote?> savedVote() async {
    _guard();
    final raw = await _client._storage.getString(_draftKey);
    _guard();
    if (raw == null) return null;
    final vote = SignedVote(raw);
    await _checkVote(vote);
    _guard();
    return vote;
  }

  @override
  Future<SignedVote> prepare(UuidValue choice) async {
    _guard();
    if (!_client._preparing.add(_draftKey)) {
      throw StateError('Já existe um preparo em andamento');
    }
    try {
      if (_registration.voterId == null ||
          _registration.participated ||
          await _client._storage.getString(_receiptKey(_alias)) != null ||
          await savedVote() != null) {
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
      final vote = SignedVote(
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

  Future<Map<String, dynamic>> _checkVote(SignedVote vote) =>
      _checkEnvelope(jsonDecode(vote.json));

  Future<Map<String, dynamic>> _checkEnvelope(Object? envelope) async {
    // Se o cadastro remoto desapareceu, só a chave desta conta e eleição
    // no Android pode autenticar a cédula usada para recuperar o recibo.
    final key =
        _registration.publicKey ??
        await DeviceVoterKeys().publicKey(_alias, create: false);
    if (key == null) {
      throw StateError(
        'Cadastro e chave local indisponíveis para conferir o recibo guardado',
      );
    }
    final payload = await _verified(envelope, key, 'ballot', _election);
    final voter = _registration.voterId;
    if (payload['manifestHash'] != _manifestHash ||
        (voter != null && payload['voterId'] != voter.uuid) ||
        payload['publicKey'] != key) {
      throw const FormatException('Cédula de outro eleitor ou manifesto');
    }
    return payload;
  }

  /// Repete o mesmo pacote após falha de transporte. O recibo só é aceito se
  /// as assinaturas, o índice e os hashes se referirem à própria submissão.
  @override
  Future<AcceptedVote> submit(SignedVote vote) async {
    _guard();
    if (_registration.voterId == null) {
      throw StateError('O cadastro do eleitor está indisponível');
    }
    if (await _client._storage.getString(_draftKey) != vote.json) {
      throw StateError(
        'A submissão exige o mesmo pacote preparado no aparelho',
      );
    }
    final ballot = await _checkVote(vote);
    final response = await _client._api.submit(_election, vote.json, _guard);
    final accepted = await _readReceipt(
      response['receipt'],
      recovered: response['recovered'] as bool,
    );
    if (accepted.ballotHash != _hash(ballot)) {
      throw const FormatException('Recibo de outra submissão');
    }
    await _bindEntry(response['entry'], accepted);
    await _retain(accepted);
    return accepted;
  }

  Future<AcceptedVote> _readReceipt(
    Object? envelope, {
    required bool recovered,
  }) async {
    final receipt = await _verified(envelope, _serverKey, 'receipt', _election);
    final index = receipt['index'];
    final entryHash = receipt['entryHash'];
    final ballotHash = receipt['ballotHash'];
    final digest = RegExp(r'^[0-9a-f]{64}$');
    if (receipt['manifestHash'] != _manifestHash ||
        index is! int ||
        index < 1 ||
        index > 100000 ||
        entryHash is! String ||
        !digest.hasMatch(entryHash) ||
        ballotHash is! String ||
        !digest.hasMatch(ballotHash)) {
      throw const FormatException('Recibo inválido para esta eleição');
    }
    return AcceptedVote(
      receiptJson: jsonEncode(envelope),
      index: index,
      entryHash: entryHash,
      ballotHash: ballotHash,
      recovered: recovered,
    );
  }

  Future<void> _bindEntry(Object? envelope, AcceptedVote vote) async {
    final entry = await _verified(
      envelope,
      _serverKey,
      'board-entry',
      _election,
    );
    final ballot = await _checkEnvelope(entry['ballot']);
    if (entry['manifestHash'] != _manifestHash ||
        entry['index'] != vote.index ||
        (vote.index == 1 && entry['previousHash'] != _manifestHash) ||
        _hash(entry) != vote.entryHash ||
        _hash(ballot) != vote.ballotHash) {
      throw const FormatException(
        'O recibo ou registro não corresponde à cédula',
      );
    }
  }

  Future<void> _retain(AcceptedVote vote) async {
    _guard();
    final stored = await _client._storage.getString(_receiptKey(_alias));
    if (stored == null) {
      _guard();
      await _client._storage.setString(_receiptKey(_alias), vote.receiptJson);
    } else {
      final original = await _readReceipt(jsonDecode(stored), recovered: true);
      if (!_sameReceipt(original, vote)) {
        throw const FormatException(
          'O recibo diverge do original neste aparelho',
        );
      }
    }
    _guard();
  }

  bool _sameReceipt(AcceptedVote signed, AcceptedVote displayed) =>
      signed.index == displayed.index &&
      signed.entryHash == displayed.entryHash &&
      signed.ballotHash == displayed.ballotHash;

  /// Usa o recibo guardado ou busca o original da conta. A cédula local ou
  /// a publicada deve pertencer ao eleitor e corresponder ao hash assinado.
  @override
  Future<AcceptedVote> recoverReceipt() async {
    _guard();
    final stored = await _client._storage.getString(_receiptKey(_alias));
    if (stored == null && _registration.voterId == null) {
      throw StateError(
        'O recibo local e o cadastro do eleitor estão indisponíveis',
      );
    }
    final envelope = stored == null
        ? await _client._api.receipt(_election, _guard)
        : jsonDecode(stored);
    final accepted = await _readReceipt(envelope, recovered: true);
    final draft = await _client._storage.getString(_draftKey);
    if (draft != null) {
      final ballot = await _checkVote(SignedVote(draft));
      if (_hash(ballot) != accepted.ballotHash) {
        throw const FormatException('Recibo de outra cédula');
      }
    } else {
      final entries = await _boardEntries();
      if (entries.length < accepted.index) {
        throw const FormatException(
          'A cédula do recibo está ausente do quadro',
        );
      }
      await _bindEntry(entries[accepted.index - 1], accepted);
    }
    await _retain(accepted);
    return accepted;
  }

  Future<List<dynamic>> _boardEntries() async {
    _guard();
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
    _guard();
    return board['entries'] as List;
  }

  /// Verifica a cadeia assinada do manifesto até o índice do recibo.
  @override
  Future<bool> isIncluded(AcceptedVote vote) async {
    _guard();
    final signed = await _readReceipt(
      jsonDecode(vote.receiptJson),
      recovered: vote.recovered,
    );
    if (!_sameReceipt(signed, vote)) {
      throw const FormatException(
        'O índice ou os hashes divergem do recibo assinado',
      );
    }
    final entries = await _boardEntries();
    if (entries.length < vote.index) return false;
    var previous = _manifestHash;
    Object? recordedBallot;
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
      recordedBallot = entry['ballot'];
    }
    if (previous != vote.entryHash) return false;
    final ballot = await _checkEnvelope(recordedBallot);
    _guard();
    return _hash(ballot) == signed.ballotHash;
  }
}
