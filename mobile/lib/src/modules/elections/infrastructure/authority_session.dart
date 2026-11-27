/// Usa a conta Firebase designada e guarda o segredo ElGamal no aparelho.
library;

import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:uuid/uuid.dart';

import '../../../infrastructure/backend/elections/ballot_api.dart';
import '../application/cryptography/elgamal.dart';
import '../application/election_authority.dart';
import 'cryptography/election_auditor.dart';
import 'cryptography/election_wire.dart';
import 'device_voter_keys.dart';

/// Sessão de autoridade vinculada a uma eleição e à conta ativa.
final class DeviceAuthoritySession implements AuthoritySession {
  DeviceAuthoritySession(
    this._api,
    this._storage,
    this._account,
    this._election,
  );

  static const _secrets = MethodChannel(
    'br.com.capp/elections/authority-secrets',
  );
  final BallotApi _api;
  final SharedPreferencesAsync _storage;
  final UuidValue? Function() _account;
  final UuidValue _election;

  UuidValue get _active =>
      _account() ?? (throw StateError('Conta da autoridade indisponível'));

  void Function() _guard(UuidValue account) => () {
    if (_account() != account) throw StateError('A conta ativa mudou');
  };

  String _alias(UuidValue account, UuidValue authority) =>
      'capp.authority.v1.${account.uuid.toLowerCase()}.'
      '${_election.uuid.toLowerCase()}.${authority.uuid.toLowerCase()}';

  AuthorityAssignment _read(Map<String, dynamic> value) {
    final id = UuidValue.withFormatValidation(value['authorityId'] as String);
    final role = value['role'];
    final slate = value['slateId'];
    if ((role != 'commission' && role != 'slate') ||
        (role == 'slate') != (slate is String)) {
      throw const FormatException('Papel de autoridade inválido');
    }
    return AuthorityAssignment(
      id: id,
      isCommission: role == 'commission',
      slate: slate is String ? UuidValue.withFormatValidation(slate) : null,
      registered: value['registered'] as bool,
      startsAt: DateTime.parse(value['startsAt'] as String),
      endsAt: DateTime.parse(value['endsAt'] as String),
      cancelled: value['cancelled'] as bool,
      manifestHash: value['manifestHash'] as String?,
    );
  }

  @override
  Future<AuthorityAssignment> inspect() async =>
      _read(await _api.authority(_election, _guard(_active)));

  @override
  Future<AuthorityAssignment> register() async {
    final account = _active;
    final guard = _guard(account);
    final current = _read(await _api.authority(_election, guard));
    if (current.registered) return current;

    final alias = _alias(account, current.id);
    final pair = await _keyPair(alias, create: true);
    guard();
    await _storage.setString('$alias.endsAt', current.endsAt.toIso8601String());
    final keys = DeviceVoterKeys();
    final signingKey = await keys.authorityPublicKey(alias, create: true);
    if (signingKey == null) {
      throw StateError('Chave de assinatura indisponível');
    }

    final authority = <String, dynamic>{
      'id': current.id.uuid,
      'role': current.isCommission ? 'commission' : 'slate',
      'slateId': current.slate?.uuid,
      'signingKey': signingKey,
      'publicValue': pair.publicValue.toString(),
    };
    final proof = pair.proveKey(
      (commitment) => _challenge({
        'version': 1,
        'kind': 'authority-key-proof',
        'electionId': _election.uuid,
        'groupId': 'rfc3526-modp14',
        'authorityId': current.id.uuid,
        'role': authority['role'],
        'slateId': authority['slateId'],
        'signingKey': signingKey,
        'publicValue': authority['publicValue'],
        'commitment': commitment.toString(),
      }),
    );
    authority['keyProof'] = {
      'commitment': proof.commitment.toString(),
      'response': proof.response.toString(),
    };
    final payload = {
      'version': 1,
      'kind': 'authority-registration',
      'electionId': _election.uuid,
      'authority': authority,
    };
    guard();
    final signature = await keys.signAsAuthority(
      alias,
      electionSigningBytes(payload),
    );
    return _read(
      await _api.registerAuthority(
        _election,
        jsonEncode({'payload': payload, 'signature': signature}),
        guard,
      ),
    );
  }

  @override
  Future<AuthorityAssignment> publish() async =>
      _read(await _api.publishManifest(_election, _guard(_active)));

  @override
  Future<AuthorityAssignment> reschedule({
    required DateTime startsAt,
    required DateTime endsAt,
    required String reason,
  }) async {
    final guard = _guard(_active);
    await _api.reschedule(_election, startsAt, endsAt, reason, guard);
    return _read(await _api.authority(_election, guard));
  }

  @override
  Future<AuthorityAssignment> cancel(String reason) async {
    final guard = _guard(_active);
    await _api.cancel(_election, reason, guard);
    return _read(await _api.authority(_election, guard));
  }

  @override
  Future<void> observeCheckpoint(String trustedManifestHash) async {
    final account = _active;
    final guard = _guard(account);
    final assignment = _read(await _api.authority(_election, guard));
    final alias = _alias(account, assignment.id);
    final board = await _api.board(_election);
    final manifest = board['manifest']['payload'] as Map<String, dynamic>;
    if (electionDigest(manifest) != trustedManifestHash) {
      throw const FormatException(
        'Manifesto divergente da referência confiável',
      );
    }
    final serverKey = manifest['serverSigningKey'] as String;
    await verifyElectionEnvelope(
      board['manifest'],
      serverKey,
      'manifest',
      _election,
    );
    final checkpoint = board['checkpoint'];
    if (checkpoint == null) return;
    final receipt = await verifyElectionEnvelope(
      checkpoint,
      serverKey,
      'receipt',
      _election,
    );
    if (receipt['manifestHash'] != trustedManifestHash) {
      throw const FormatException('Checkpoint de outro manifesto');
    }
    final previous = await _storage.getString('$alias.checkpoint');
    if (previous != null) {
      final previousIndex = jsonDecode(previous)['payload']['index'] as int;
      if ((receipt['index'] as int) <= previousIndex) {
        throw const FormatException(
          'O checkpoint novo não avança a referência',
        );
      }
      final entry = (board['entries'] as List)[previousIndex - 1];
      if (electionDigest(entry['payload'] as Map<String, dynamic>) !=
          jsonDecode(previous)['payload']['entryHash']) {
        throw const FormatException('O quadro diverge do checkpoint guardado');
      }
    }
    guard();
    await _storage.setString('$alias.checkpoint', jsonEncode(checkpoint));
  }

  @override
  Future<void> contribute(String trustedManifestHash) async {
    final account = _active;
    final guard = _guard(account);
    final assignment = _read(await _api.authority(_election, guard));
    if (assignment.manifestHash != trustedManifestHash ||
        !RegExp(r'^[0-9a-f]{64}$').hasMatch(trustedManifestHash)) {
      throw const FormatException('Manifesto sem referência confiável');
    }
    final alias = _alias(account, assignment.id);
    final key = '$alias.contribution';
    final tally = await _api.tally(_election);
    final board = await _api.board(_election);
    final retained = await _storage.getString('$alias.closure');
    final checkpoint = await _storage.getString('$alias.checkpoint');
    await auditElection(
      board: board,
      tally: tally,
      election: _election,
      trustedManifestHash: trustedManifestHash,
      externalClosure: retained == null ? null : jsonDecode(retained),
      externalReceipt: checkpoint == null ? null : jsonDecode(checkpoint),
    );
    final manifest = board['manifest']['payload'] as Map<String, dynamic>;
    final closure = board['closure']['payload'] as Map<String, dynamic>;
    final aggregate = board['aggregate']['payload'] as Map<String, dynamic>;
    final endsAt = DateTime.parse(board['endsAt'] as String);
    final closedAt = DateTime.parse(closure['closedAt'] as String);
    final retainedEnd = await _storage.getString('$alias.endsAt');
    if (retainedEnd == null ||
        DateTime.now().isBefore(endsAt) ||
        closedAt.isBefore(endsAt) ||
        closedAt.isBefore(DateTime.parse(retainedEnd))) {
      throw const FormatException('Encerramento anterior ao prazo público');
    }
    guard();
    final replay = await _storage.getString(key);
    if (replay != null) {
      await _api.contribute(_election, assignment.id, replay, guard);
      return;
    }
    final pair = await _keyPair(alias, create: false);
    final keys = DeviceVoterKeys();
    final signingKey = await keys.authorityPublicKey(alias, create: false);
    final authority = (manifest['authorities'] as List).firstWhere(
      (row) => row['id'] == assignment.id.uuid,
    );
    if (pair.publicValue.toString() != authority['publicValue'] ||
        signingKey != authority['signingKey']) {
      throw const FormatException('As chaves locais divergem do manifesto');
    }

    final closureHash = electionDigest(closure);
    final aggregateHash = electionDigest(aggregate);
    final shares = [
      for (final row in aggregate['ciphertexts'] as List)
        _contribution(
          pair,
          row as Map<String, dynamic>,
          assignment.id,
          trustedManifestHash,
          closureHash,
          aggregateHash,
        ),
    ];
    final payload = {
      'version': 1,
      'kind': 'decryption-share',
      'electionId': _election.uuid,
      'manifestHash': trustedManifestHash,
      'closureHash': closureHash,
      'aggregateHash': aggregateHash,
      'authorityId': assignment.id.uuid,
      'shares': shares,
    };
    guard();
    final signature = await keys.signAsAuthority(
      alias,
      electionSigningBytes(payload),
    );
    final envelope = jsonEncode({'payload': payload, 'signature': signature});
    await _storage.setString('$alias.closure', jsonEncode(board['closure']));
    await _storage.setString(key, envelope);
    await _api.contribute(_election, assignment.id, envelope, guard);
  }

  Map<String, dynamic> _contribution(
    ElGamalAuthorityKeyPair pair,
    Map<String, dynamic> cipher,
    UuidValue authority,
    String manifest,
    String closure,
    String aggregate,
  ) {
    final share = pair.contribute(
      readElectionElement(cipher['a'] as String),
      (value, commitmentG, commitmentA) => _challenge({
        'version': 1,
        'kind': 'decryption-proof',
        'electionId': _election.uuid,
        'groupId': 'rfc3526-modp14',
        'manifestHash': manifest,
        'closureHash': closure,
        'aggregateHash': aggregate,
        'authorityId': authority.uuid,
        'optionId': cipher['optionId'],
        'publicValue': pair.publicValue.toString(),
        'a': cipher['a'],
        'b': cipher['b'],
        'value': value.toString(),
        'commitmentG': commitmentG.toString(),
        'commitmentA': commitmentA.toString(),
      }),
    );
    return {
      'optionId': cipher['optionId'],
      'value': share.value.toString(),
      'proof': {
        'commitmentG': share.commitmentG.toString(),
        'commitmentA': share.commitmentA.toString(),
        'response': share.response.toString(),
      },
    };
  }

  BigInt _challenge(Map<String, dynamic> transcript) =>
      BigInt.parse(electionDigest(transcript), radix: 16);

  Future<ElGamalAuthorityKeyPair> _keyPair(
    String alias, {
    required bool create,
  }) async {
    var bytes = await _secrets.invokeMethod<Uint8List>('read', {
      'alias': alias,
    });
    if (bytes == null && create) {
      final secret = generateAuthorityKeyPair().secret
          .toRadixString(16)
          .padLeft(512, '0');
      bytes = await _secrets.invokeMethod<Uint8List>('storeIfAbsent', {
        'alias': alias,
        'secret': Uint8List.fromList([
          for (var i = 0; i < secret.length; i += 2)
            int.parse(secret.substring(i, i + 2), radix: 16),
        ]),
      });
    }
    if (bytes == null || bytes.length != 256) {
      throw StateError('Use o dispositivo que cadastrou a chave da autoridade');
    }
    final hex = bytes
        .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
        .join();
    return ElGamalAuthorityKeyPair.fromSecret(BigInt.parse(hex, radix: 16));
  }
}
