/// Verifica e recalcula uma eleição no dispositivo apenas com evidência pública.
///
/// A chave confiável vem do hash do manifesto recebido por outro canal. O
/// recálculo verifica a cadeia fornecida. Um recibo independente ancora o
/// prefixo correspondente e um termo externo ancora o estado final.
library;

import 'package:uuid/uuid.dart';

import '../../application/cryptography/elgamal.dart';
import '../../application/election_audit.dart';
import 'election_wire.dart';

typedef _Json = Map<String, dynamic>;

final _idPattern = RegExp(
  r'^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$',
);
const _nil = '00000000-0000-0000-0000-000000000000';
const _group = 'rfc3526-modp14';

_Json _shape(Object? value, Set<String> names, String kind) {
  if (value is! _Json ||
      value.length != names.length ||
      !value.keys.every(names.contains)) {
    throw FormatException('Campos de $kind inválidos');
  }
  return value;
}

String _text(_Json json, String key) {
  final value = json[key];
  if (value is! String || value.isEmpty) {
    throw FormatException('Campo $key inválido');
  }
  return value;
}

String _id(_Json value, String key, {bool allowNil = false}) {
  final id = _text(value, key);
  if (!_idPattern.hasMatch(id) || (!allowNil && id == _nil)) {
    throw FormatException('Identificador $key inválido');
  }
  return id;
}

void _require(bool condition, String message) {
  if (!condition) throw FormatException(message);
}

List<dynamic> _rows(Object? value, String label) {
  if (value is! List) throw FormatException('Lista $label inválida');
  return value;
}

Future<_Json> _signed(
  Object? envelope,
  String key,
  String kind,
  UuidValue election,
  Set<String> fields,
) async {
  final payload = await verifyElectionEnvelope(envelope, key, kind, election);
  return _shape(payload, {'version', 'kind', 'electionId', ...fields}, kind);
}

BigInt _challenge(_Json transcript) =>
    BigInt.parse(electionDigest(transcript), radix: 16);

/// Recalcula o agregado, confere as contribuições e compara os totais publicados.
///
/// O verificador usa apenas os argumentos e a assinatura RSA do dispositivo.
/// Evidência malformada ou adulterada causa [FormatException]. A ausência de
/// uma contribuição produz apuração incompleta. Recibos e termos obtidos por
/// outro canal permitem verificar continuidade.
Future<ElectionAudit> auditElection({
  required Map<String, dynamic> board,
  required Map<String, dynamic> tally,
  required UuidValue election,
  required String trustedManifestHash,
  Object? externalReceipt,
  Object? externalClosure,
}) async {
  _shape(board, {
    'manifest',
    'startsAt',
    'endsAt',
    'entries',
    'checkpoint',
    'closure',
    'aggregate',
    if (board.containsKey('cancellation')) 'cancellation',
  }, 'quadro');
  _shape(tally, {'shares', 'result'}, 'apuração');
  final manifestEnvelope = _shape(board['manifest'], {
    'payload',
    'signature',
  }, 'manifesto');
  final manifest = _shape(manifestEnvelope['payload'], {
    'version',
    'kind',
    'electionId',
    'academicCenterId',
    'groupId',
    'options',
    'authorities',
    'combinedPublicKey',
    'serverSigningKey',
  }, 'manifesto');
  _require(
    electionDigest(manifest) == trustedManifestHash,
    'Manifesto divergente da referência confiável',
  );
  final serverKey = _text(manifest, 'serverSigningKey');
  await verifyElectionEnvelope(
    manifestEnvelope,
    serverKey,
    'manifest',
    election,
  );
  _id(manifest, 'academicCenterId');
  _require(manifest['groupId'] == _group, 'Grupo eleitoral não suportado');
  final options = _options(manifest);
  final optionNames = List<String>.unmodifiable(
    (manifest['options'] as List).map((row) => row['name'] as String),
  );
  final authorities = _authorities(manifest, options, election);

  final entries = _rows(board['entries'], 'registros');
  _require(entries.length <= 100000, 'Limite de cédulas excedido');
  final accumulator = List<ElGamalCiphertext>.filled(options.length, (
    a: BigInt.one,
    b: BigInt.one,
  ));
  final heads = <String>[];
  final ballots = <String>[];
  final voters = <String>{};
  var previous = trustedManifestHash;
  for (var index = 0; index < entries.length; index++) {
    final entry = await _signed(
      entries[index],
      serverKey,
      'board-entry',
      election,
      {'manifestHash', 'index', 'previousHash', 'ballot'},
    );
    _require(
      entry['index'] == index + 1 &&
          entry['previousHash'] == previous &&
          entry['manifestHash'] == trustedManifestHash,
      'Cadeia de registros divergente',
    );
    final envelope = _shape(entry['ballot'], {
      'payload',
      'signature',
    }, 'cédula');
    final ballot = _shape(envelope['payload'], {
      'electionId',
      'accountId',
      'manifestHash',
      'publicKey',
      'ciphertexts',
    }, 'cédula');
    await verifyBallotEnvelope(
      envelope,
      _text(ballot, 'publicKey'),
      election,
      _id(ballot, 'accountId'),
    );
    _require(
      ballot['manifestHash'] == trustedManifestHash,
      'Cédula de outro manifesto',
    );
    _require(voters.add(_id(ballot, 'accountId')), 'Eleitor repetido');
    final ciphers = _ciphertexts(
      ballot['ciphertexts'],
      options,
      individual: true,
    );
    for (var i = 0; i < ciphers.length; i++) {
      accumulator[i] = (
        a: multiplyElectionElements(accumulator[i].a, ciphers[i].a),
        b: multiplyElectionElements(accumulator[i].b, ciphers[i].b),
      );
    }
    previous = electionDigest(entry);
    heads.add(previous);
    ballots.add(ballotDigest(ballot));
  }

  Future<int> verifyReceipt(Object? receipt) async {
    final payload = await _signed(receipt, serverKey, 'receipt', election, {
      'manifestHash',
      'ballotHash',
      'index',
      'entryHash',
    });
    final index = payload['index'];
    _require(
      index is int && index > 0 && index <= heads.length,
      'O quadro omite a posição do recibo',
    );
    final position = (index as int) - 1;
    _require(
      payload['manifestHash'] == trustedManifestHash &&
          payload['ballotHash'] == ballots[position] &&
          payload['entryHash'] == heads[position],
      'Recibo incompatível com a entrada do quadro',
    );
    return index;
  }

  if (entries.isEmpty) {
    _require(board['checkpoint'] == null, 'Quadro vazio com recibo');
  } else {
    _require(
      await verifyReceipt(board['checkpoint']) == entries.length,
      'Recibo final incompatível com o quadro',
    );
  }
  final anchoredThrough = externalReceipt == null
      ? 0
      : await verifyReceipt(externalReceipt);
  if (board.containsKey('cancellation')) {
    final cancellation = _shape(board['cancellation'], {
      'cancelledAt',
      'cancelledBy',
      'reason',
    }, 'cancelamento');
    _id(cancellation, 'cancelledBy');
    final reason = _text(cancellation, 'reason');
    _require(
      DateTime.tryParse(_text(cancellation, 'cancelledAt'))?.isUtc ?? false,
      'Data do cancelamento inválida',
    );
    _require(
      board['closure'] == null &&
          board['aggregate'] == null &&
          externalClosure == null &&
          tally['result'] == null &&
          _rows(tally['shares'], 'contribuições').isEmpty,
      'O cancelamento diverge das evidências de apuração',
    );
    return ElectionAudit(
      tally: AuditedTally.cancelled,
      totals: null,
      optionNames: optionNames,
      anchoredThrough: anchoredThrough,
      finalClosureAnchored: false,
      cancellationReason: reason,
    );
  }
  final closure = await _signed(
    board['closure'],
    serverKey,
    'closure',
    election,
    {'manifestHash', 'voterCount', 'headHash', 'closedAt'},
  );
  _require(
    closure['manifestHash'] == trustedManifestHash &&
        closure['voterCount'] == entries.length &&
        closure['headHash'] == previous,
    'O encerramento não corresponde ao quadro',
  );
  final closureHash = electionDigest(closure);
  if (externalClosure != null) {
    final retained = await verifyElectionEnvelope(
      externalClosure,
      serverKey,
      'closure',
      election,
    );
    _require(
      electionDigest(retained) == closureHash,
      'Encerramento diverge da evidência externa',
    );
  }

  final aggregate = await _signed(
    board['aggregate'],
    serverKey,
    'aggregate',
    election,
    {'manifestHash', 'closureHash', 'ciphertexts'},
  );
  _require(
    aggregate['manifestHash'] == trustedManifestHash &&
        aggregate['closureHash'] == closureHash,
    'Agregado de outro encerramento',
  );
  final expected = _ciphertexts(aggregate['ciphertexts'], options);
  _require(
    List.generate(
      options.length,
      (i) => accumulator[i] == expected[i],
    ).every((same) => same),
    'Agregado diverge do produto das cédulas',
  );

  final aggregateHash = electionDigest(aggregate);
  final contributions = <String, List<BigInt>>{};
  for (final envelope in _rows(tally['shares'], 'contribuições')) {
    final unsigned = _shape(envelope, {
      'payload',
      'signature',
    }, 'contribuição')['payload'];
    final payload = _shape(unsigned, {
      'version',
      'kind',
      'electionId',
      'manifestHash',
      'closureHash',
      'aggregateHash',
      'authorityId',
      'shares',
    }, 'contribuição');
    final id = _id(payload, 'authorityId');
    final authority = authorities[id];
    if (authority == null || contributions.containsKey(id)) {
      throw const FormatException(
        'Autoridade ausente ou repetida no manifesto',
      );
    }
    await verifyElectionEnvelope(
      envelope,
      _text(authority, 'signingKey'),
      'decryption-share',
      election,
    );
    _require(
      payload['manifestHash'] == trustedManifestHash &&
          payload['closureHash'] == closureHash &&
          payload['aggregateHash'] == aggregateHash,
      'Contribuição de outro agregado',
    );
    final rows = _rows(payload['shares'], 'valores da contribuição');
    _require(
      rows.length == options.length,
      'Dimensão da contribuição inválida',
    );
    final values = <BigInt>[];
    for (var i = 0; i < options.length; i++) {
      final row = _shape(rows[i], {
        'optionId',
        'value',
        'proof',
      }, 'valor parcial');
      _require(row['optionId'] == options[i], 'Ordem da contribuição inválida');
      final proof = _shape(row['proof'], {
        'commitmentG',
        'commitmentA',
        'response',
      }, 'prova de decifração');
      final value = readElectionElement(_text(row, 'value'));
      final challenge = _challenge({
        'version': 1,
        'kind': 'decryption-proof',
        'electionId': election.uuid,
        'groupId': _group,
        'manifestHash': trustedManifestHash,
        'closureHash': closureHash,
        'aggregateHash': aggregateHash,
        'authorityId': id,
        'optionId': options[i],
        'publicValue': authority['publicValue'],
        'a': accumulator[i].a.toString(),
        'b': accumulator[i].b.toString(),
        'value': row['value'],
        'commitmentG': proof['commitmentG'],
        'commitmentA': proof['commitmentA'],
      });
      _require(
        verifyElectionDecryption(
          publicValue: readElectionElement(
            _text(authority, 'publicValue'),
            allowIdentity: false,
          ),
          base: accumulator[i].a,
          value: value,
          commitmentG: readElectionElement(
            _text(proof, 'commitmentG'),
            allowIdentity: false,
          ),
          commitmentA: readElectionElement(_text(proof, 'commitmentA')),
          challenge: challenge,
          response: _text(proof, 'response'),
        ),
        'Prova de decifração inválida',
      );
      values.add(value);
    }
    contributions[id] = values;
  }

  final missing = authorities.keys
      .where((id) => !contributions.containsKey(id))
      .toList();
  final totals = missing.isEmpty
      ? recoverElectionTotals(
          accumulator,
          contributions.values.toList(),
          entries.length,
        )
      : null;
  final (status, reason) = missing.isNotEmpty
      ? ('incomplete', 'missing_authority')
      : totals == null
      ? ('annulled', 'tally_out_of_range')
      : totals.reduce((a, b) => a + b) != entries.length
      ? ('annulled', 'aggregate_sum_mismatch')
      : ('valid', null);
  final published = tally['result'];
  if (published != null) {
    final result = await verifyElectionEnvelope(
      published,
      serverKey,
      'result',
      election,
    );
    final computed = {
      'version': 1,
      'kind': 'result',
      'electionId': election.uuid,
      'manifestHash': trustedManifestHash,
      'closureHash': closureHash,
      'aggregateHash': aggregateHash,
      'cancellationHash': null,
      'voterCount': entries.length,
      'status': status,
      'reason': reason,
      'counts': totals == null
          ? null
          : [
              for (var i = 0; i < options.length; i++)
                {'optionId': options[i], 'count': totals[i]},
            ],
      'missingAuthorities': missing,
    };
    _require(
      electionDigest(result) == electionDigest(computed),
      'O resultado publicado diverge do recálculo',
    );
  } else {
    _require(missing.isNotEmpty, 'Resultado definitivo ausente');
  }

  return ElectionAudit(
    tally: AuditedTally.values.byName(status),
    totals: totals == null ? null : List.unmodifiable(totals),
    optionNames: optionNames,
    anchoredThrough: anchoredThrough,
    finalClosureAnchored: externalClosure != null,
  );
}

List<String> _options(_Json manifest) {
  final rows = _rows(manifest['options'], 'opções');
  _require(
    rows.length >= 2 && rows.length <= 51,
    'Dimensão do manifesto inválida',
  );
  final ids = <String>[];
  for (var index = 0; index < rows.length; index++) {
    final row = _shape(rows[index], {'id', 'name', 'kind'}, 'opção');
    final last = index == rows.length - 1;
    final id = _id(row, 'id', allowNil: last);
    final name = _text(row, 'name');
    _require(
      name.trim().isNotEmpty &&
          name.runes.length <= 200 &&
          (last
              ? (row['kind'] == 'null' && id == _nil && name == 'Nulo')
              : row['kind'] == 'slate'),
      'Opção inválida no manifesto',
    );
    ids.add(id);
  }
  _require(ids.toSet().length == ids.length, 'Opção repetida');
  return ids;
}

Map<String, _Json> _authorities(
  _Json manifest,
  List<String> options,
  UuidValue election,
) {
  final rows = _rows(manifest['authorities'], 'autoridades');
  _require(rows.length == options.length + 1, 'Autoridades insuficientes');
  final result = <String, _Json>{};
  final roles = <String>{};
  final signingKeys = <String>{};
  final publicValues = <BigInt>{};
  var combined = BigInt.one;
  for (final value in rows) {
    final authority = _shape(value, {
      'id',
      'role',
      'slateId',
      'signingKey',
      'publicValue',
      'keyProof',
    }, 'autoridade');
    final id = _id(authority, 'id');
    final role = _text(authority, 'role');
    final slate = authority['slateId'];
    if (role == 'slate') {
      _require(
        slate is String && slate != _nil && options.contains(slate),
        'Autoridade de chapa desconhecida',
      );
    } else {
      _require(
        (role == 'server' || role == 'commission') && slate == null,
        'Papel da autoridade inválido',
      );
    }
    _require(
      roles.add(slate == null ? role : 'slate/$slate') &&
          !result.containsKey(id),
      'Autoridade ou papel repetido',
    );
    final signingKey = _text(authority, 'signingKey');
    final publicValue = readElectionElement(
      _text(authority, 'publicValue'),
      allowIdentity: false,
    );
    _require(
      signingKeys.add(signingKey) && publicValues.add(publicValue),
      'Chave de autoridade repetida',
    );
    final proof = _shape(authority['keyProof'], {
      'commitment',
      'response',
    }, 'prova de chave');
    _require(
      verifyAuthorityKeyProof(
        publicValue: publicValue,
        commitment: readElectionElement(
          _text(proof, 'commitment'),
          allowIdentity: false,
        ),
        challenge: _challenge({
          'version': 1,
          'kind': 'authority-key-proof',
          'electionId': election.uuid,
          'groupId': _group,
          'authorityId': id,
          'role': role,
          'slateId': slate,
          'signingKey': signingKey,
          'publicValue': authority['publicValue'],
          'commitment': proof['commitment'],
        }),
        response: _text(proof, 'response'),
      ),
      'Prova de posse da autoridade inválida',
    );
    combined = multiplyElectionElements(combined, publicValue);
    result[id] = authority;
  }
  _require(
    combined ==
        readElectionElement(
          _text(manifest, 'combinedPublicKey'),
          allowIdentity: false,
        ),
    'Chave conjunta inválida',
  );
  return result;
}

List<ElGamalCiphertext> _ciphertexts(
  Object? value,
  List<String> options, {
  bool individual = false,
}) {
  final rows = _rows(value, 'cifras');
  _require(
    rows.length == options.length,
    'Dimensão do vetor de cifras inválida',
  );
  return List.generate(rows.length, (index) {
    final row = _shape(rows[index], {'optionId', 'a', 'b'}, 'cifra');
    _require(
      row['optionId'] == options[index],
      'Ordem do vetor de cifras inválida',
    );
    return (
      a: readElectionElement(_text(row, 'a'), allowIdentity: !individual),
      b: readElectionElement(_text(row, 'b')),
    );
  });
}
