"""Confere os formatos e vetores compartilhados do protocolo eleitoral.

As implementações de domínio, criptografia, registro e auditoria precisam usar
os mesmos bytes antes de existirem endpoints de votação. Este programa valida
o corpus versionado: schemas, JCS, hashes, assinaturas e relações matemáticas
dos exemplos. Não autentica eleitores, não aceita votos e não substitui o
auditor independente da eleição real.

Requer as dependências de requirements-ci.txt e OpenSSL no PATH. O OpenSSL
opera somente sobre chaves públicas e dados sintéticos em diretório temporário.
Nenhuma chave privada é carregada ou publicada por este validador.
"""

import base64
import copy
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tempfile

from jsonschema import Draft202012Validator, FormatChecker
import rfc8785

CONTRACTS = Path(__file__).resolve().parents[1] / "elections"


class ContractError(ValueError):
    """Falha de formato, referência ou verificação de um vetor compartilhado."""


def _object(pairs):
    """Reconstrói um objeto JSON recusando duplicatas antes de perder informação."""
    result = {}

    for name, value in pairs:
        if name in result:
            raise ContractError(f"Propriedade duplicada: {name}")

        result[name] = value

    return result


def _nonfinite(value):
    """Recusa NaN e infinitos, que não são números JSON admitidos pelo protocolo."""
    raise ContractError(f"Número JSON não finito: {value}")


def _fractional(value):
    """Recusa ponto decimal/expoente em campos cujo contrato usa inteiros JSON."""
    raise ContractError(f"Literal numérico não inteiro: {value}")


def _integer(value):
    """Mantém inteiros exatos e recusa o zero negativo, sem perda por double."""
    if value == "-0":
        raise ContractError("Zero negativo não canônico")
    return int(value)


def read_json(path):
    """Lê UTF-8 estrito, preservando a detecção de campos repetidos.

    Erros de arquivo e decodificação mantêm sua causa. Conteúdo JSON inválido
    ou ambíguo é recusado; nenhuma correção ou normalização é gravada no arquivo.
    """
    return json.loads(
        Path(path).read_text(encoding="utf-8"),
        object_pairs_hook=_object,
        parse_constant=_nonfinite,
        parse_float=_fractional,
        parse_int=_integer,
    )


def signing_bytes(payload, profile):
    """Produz prefixo versionado seguido de UTF-8 JCS, sem a assinatura externa.

    A canonicalização é executada pela implementação RFC 8785 fixada no pacote.
    Valores fora de seu domínio são recusados; não há fallback para json.dumps.
    """
    return bytes.fromhex(profile["signedPrefixHex"]) + rfc8785.dumps(payload)


def digest(payload, profile):
    """Calcula SHA-256 dos mesmos bytes usados pela assinatura do payload."""
    return hashlib.sha256(signing_bytes(payload, profile)).hexdigest()


def binary(value):
    """Decodifica Base64url canônico sem padding ou lança ContractError."""
    try:
        raw = base64.b64decode(
            value + "=" * (-len(value) % 4), altchars=b"-_", validate=True
        )
    except (ValueError, TypeError) as error:
        raise ContractError("Base64url inválido") from error

    if base64.urlsafe_b64encode(raw).rstrip(b"=").decode("ascii") != value:
        raise ContractError("Base64url não canônico")

    return raw


def _openssl(arguments):
    """Executa uma operação pública com limite de tempo e diagnóstico seguro."""
    try:
        result = subprocess.run(
            ["openssl", *map(str, arguments)],
            capture_output=True,
            timeout=30,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise ContractError(
            "OpenSSL indisponível ou operação excedeu o prazo"
        ) from error

    if result.returncode:
        raise ContractError("Verificação de chave ou assinatura RSA-PSS recusada")

    return result.stdout


def verify_signature(payload, signature, public_key, profile):
    """Verifica RSA-PSS/SHA-256 com o perfil fixado e chave DER pública.

    Além da assinatura, exige módulo de tamanho autorizado, expoente 65537 e
    DER SPKI canônico com rsaEncryption. O diretório temporário contém somente
    material público e é removido inclusive em erro. PSS não é comparado byte
    a byte com assinaturas novas, pois o salt é aleatório.
    """
    key_der = binary(public_key)
    signature_bytes = binary(signature)

    with tempfile.TemporaryDirectory(prefix="capp-election-vector-") as name:
        directory = Path(name)
        key_path = directory / "public.der"
        pem_path = directory / "public.pem"
        message_path = directory / "message.bin"
        signature_path = directory / "signature.bin"
        key_path.write_bytes(key_der)
        message_path.write_bytes(signing_bytes(payload, profile))
        signature_path.write_bytes(signature_bytes)

        _openssl(
            [
                "pkey",
                "-pubin",
                "-inform",
                "DER",
                "-in",
                key_path,
                "-pubout",
                "-out",
                pem_path,
            ]
        )
        normalized = _openssl(
            [
                "pkey",
                "-pubin",
                "-in",
                pem_path,
                "-pubout",
                "-outform",
                "DER",
            ]
        )

        if normalized != key_der:
            raise ContractError("DER público não canônico")

        # OID rsaEncryption seguido de NULL no AlgorithmIdentifier SPKI.
        if bytes.fromhex("300d06092a864886f70d0101010500") not in key_der[:24]:
            raise ContractError("Algoritmo SPKI diferente de rsaEncryption")

        text = _openssl(["pkey", "-pubin", "-in", pem_path, "-text", "-noout"])
        match = re.search(rb"Public-Key: \((\d+) bit\)", text)

        if not match or int(match[1]) not in profile["signature"]["modulusBits"]:
            raise ContractError("Tamanho do módulo RSA não permitido")

        if b"Exponent: 65537 " not in text:
            raise ContractError("Expoente RSA não permitido")

        if len(signature_bytes) != int(match[1]) // 8:
            raise ContractError("Comprimento da assinatura incompatível")

        _openssl(
            [
                "dgst",
                "-sha256",
                "-verify",
                pem_path,
                "-sigopt",
                "rsa_padding_mode:pss",
                "-sigopt",
                "rsa_mgf1_md:sha256",
                "-sigopt",
                f"rsa_pss_saltlen:{profile['signature']['saltLengthBytes']}",
                "-signature",
                signature_path,
                message_path,
            ]
        )


def _require(condition, message):
    """Transforma a violação de uma invariante em erro estável do corpus."""
    if not condition:
        raise ContractError(message)


def _group(profile):
    """Obtém p/q/g do grupo fixado, sem parâmetros fornecidos pelo eleitor."""
    prime = int(profile["group"]["primeHex"], 16)
    return prime, (prime - 1) // 2, int(profile["group"]["generator"])


def _element(value, profile, nonidentity=False):
    """Confere faixa/subgrupo e, quando exigido, recusa a identidade."""
    prime, order, _ = _group(profile)
    number = int(value)
    _require(1 <= number < prime, "Elemento fora da faixa do grupo")
    _require(pow(number, order, prime) == 1, "Elemento fora do subgrupo")
    _require(not nonidentity or number != 1, "Identidade não permitida")
    return number


def _challenge(payload, profile):
    """Deriva o desafio Fiat-Shamir com domínio, eleição e contexto explícitos."""
    _, order, _ = _group(profile)
    return (
        int.from_bytes(hashlib.sha256(signing_bytes(payload, profile)).digest(), "big")
        % order
    )


def _artifact(index, value, kind):
    """Resolve uma referência pelo hash do payload e exige o tipo esperado."""
    result = index.get(value)
    _require(
        result is not None and result["kind"] == kind,
        "Referência ausente ou de outro tipo",
    )
    return result


def _ordered_options(items, manifest):
    """Exige o mesmo vetor de opções, sem omissões, duplicações ou reordenação."""
    _require(
        [item["optionId"] for item in items]
        == [item["id"] for item in manifest["options"]],
        "Vetor de opções diferente do manifesto",
    )


def _snapshot(head, count, manifest, index, profile):
    """Percorre somente a fotografia referenciada, conferindo índices e eleitores.

    Não consulta autenticação ou elegibilidade real. A identidade pública é
    tomada como associação fornecida pelo backend no modelo aprovado.
    """
    entries = []
    seen = set()
    manifest_hash = digest(manifest, profile)

    for expected in range(count, 0, -1):
        entry = _artifact(index, head, "board-entry")
        _require(entry["index"] == expected, "Índice do quadro divergente")
        _require(entry["manifestHash"] == manifest_hash, "Quadro de outro manifesto")
        voter = entry["ballot"]["payload"]["voterId"]
        _require(voter not in seen, "Eleitor repetido na fotografia")
        seen.add(voter)
        entries.append(entry)
        head = entry["previousHash"]

    _require(head == manifest_hash, "Início do quadro divergente")
    return list(reversed(entries))


def validate_relations(payload, profile, index):
    """Confere relações locais dos vetores, sem executar políticas do servidor.

    index contém payloads sintéticos identificados por digest. As verificações
    servem como referências de interoperabilidade. Não inferem elegibilidade
    acadêmica a partir de arquivos públicos nem autenticam uma conta Firebase.
    """
    kind = payload["kind"]
    prime, order, generator = _group(profile)

    if kind == "result" and payload["cancellationHash"] is not None:
        cancellation = _artifact(index, payload["cancellationHash"], "cancellation")
        _require(
            all(
                payload[field] == cancellation[field]
                for field in ("electionId", "manifestHash", "voterCount")
            ),
            "Resultado de outro cancelamento",
        )

    if kind == "calendar-origin":
        _require(payload["startsAt"] < payload["endsAt"], "Período inicial inválido")
        return

    if kind == "schedule-change":
        previous = index.get(payload["previousScheduleHash"])
        _require(
            previous is not None
            and previous["kind"] in {"calendar-origin", "schedule-change"},
            "Calendário anterior ausente",
        )
        _require(
            previous["electionId"] == payload["electionId"],
            "Calendário de outra eleição",
        )
        if payload["manifestHash"] is not None:
            configured = _artifact(index, payload["manifestHash"], "manifest")
            _require(
                configured["electionId"] == payload["electionId"],
                "Manifesto de outra eleição",
            )
        _require(
            payload["revision"] == previous.get("revision", 0) + 1,
            "Revisão não consecutiva",
        )
        _require(payload["startsAt"] < payload["endsAt"], "Período inválido")
        _require(
            payload["changedAt"] < previous["endsAt"], "Prorrogação depois do prazo"
        )
        _require(
            payload["endsAt"] > previous["endsAt"], "Novo encerramento não foi adiado"
        )
        if payload["changedAt"] >= previous["startsAt"]:
            _require(
                payload["startsAt"] == previous["startsAt"],
                "Abertura já ocorrida foi alterada",
            )
        else:
            _require(payload["startsAt"] >= previous["startsAt"], "Abertura antecipada")
        return

    if kind == "manifest":
        options = payload["options"]
        authorities = payload["authorities"]
        slates = [x["id"] for x in options if x["kind"] == "slate"]
        _require(len(slates) >= profile["limits"]["minSlates"], "Chapas insuficientes")
        _require(len(slates) == len(set(slates)), "Chapas duplicadas")
        _require(
            options[-1]["id"] == profile["nullOptionId"], "Nulo fora da última posição"
        )
        calendar = _artifact(index, payload["calendarOriginHash"], "calendar-origin")
        _require(
            calendar["electionId"] == payload["electionId"],
            "Calendário de outra eleição",
        )
        ids = [x["id"] for x in authorities]
        _require(len(ids) == len(set(ids)), "Autoridades duplicadas")
        _require(
            sum(x["role"] == "server" for x in authorities) == 1,
            "Autoridade do servidor inválida",
        )
        _require(
            sum(x["role"] == "commission" for x in authorities) == 1,
            "Autoridade da comissão inválida",
        )
        _require(
            sorted(x["slateId"] for x in authorities if x["role"] == "slate")
            == sorted(slates),
            "Autoridades das chapas divergentes",
        )
        keys = [x["signingKey"] for x in authorities]
        values = [x["publicValue"] for x in authorities]
        _require(
            len(set(keys)) == len(keys) and len(set(values)) == len(values),
            "Chaves de autoridades repetidas",
        )
        combined = 1

        for authority in authorities:
            public = _element(authority["publicValue"], profile, True)
            commitment = _element(authority["keyProof"]["commitment"], profile, True)
            response = int(authority["keyProof"]["response"])
            _require(0 <= response < order, "Resposta Schnorr fora da faixa")
            challenge_payload = {
                "version": 1,
                "kind": "authority-key-proof",
                "electionId": payload["electionId"],
                "groupId": payload["groupId"],
                "authorityId": authority["id"],
                "role": authority["role"],
                "slateId": authority["slateId"],
                "signingKey": authority["signingKey"],
                "publicValue": authority["publicValue"],
                "commitment": authority["keyProof"]["commitment"],
            }
            challenge = _challenge(challenge_payload, profile)
            _require(
                pow(generator, response, prime)
                == commitment * pow(public, challenge, prime) % prime,
                "Prova Schnorr inválida",
            )
            combined = combined * public % prime

        _require(
            _element(payload["combinedPublicKey"], profile, True) == combined,
            "Chave conjunta divergente",
        )
        return

    if payload.get("manifestHash") is None:
        _require(kind in {"cancellation", "result"}, "Manifesto ausente")
        _require(payload["voterCount"] == 0, "Participação sem manifesto")
        if kind == "cancellation":
            _require(payload["headHash"] is None, "Quadro sem manifesto")
            if payload["scheduleHash"] is not None:
                calendar = index.get(payload["scheduleHash"])
                _require(
                    calendar is not None
                    and calendar["kind"] in {"calendar-origin", "schedule-change"},
                    "Calendário ausente",
                )
                _require(
                    calendar["electionId"] == payload["electionId"],
                    "Calendário de outra eleição",
                )
        return

    manifest = _artifact(index, payload["manifestHash"], "manifest")
    _require(payload["electionId"] == manifest["electionId"], "Eleição divergente")

    if kind == "board-entry":
        if payload["index"] == 1:
            _require(
                payload["previousHash"] == payload["manifestHash"],
                "Gênese do quadro divergente",
            )
        else:
            previous = _artifact(index, payload["previousHash"], "board-entry")
            _require(
                previous["index"] + 1 == payload["index"],
                "Índice do quadro não consecutivo",
            )
            _require(
                previous["manifestHash"] == payload["manifestHash"],
                "Quadro de outro manifesto",
            )
        ballot = payload["ballot"]
        _require(
            ballot["payload"]["manifestHash"] == payload["manifestHash"],
            "Cédula de outro manifesto",
        )
        verify_signature(
            ballot["payload"],
            ballot["signature"],
            ballot["payload"]["publicKey"],
            profile,
        )
        validate_relations(ballot["payload"], profile, index)

    if kind == "receipt":
        entry = _artifact(index, payload["entryHash"], "board-entry")
        _require(
            entry["manifestHash"] == payload["manifestHash"],
            "Recibo de outro manifesto",
        )
        _require(entry["index"] == payload["index"], "Índice do recibo divergente")
        _require(
            digest(entry["ballot"]["payload"], profile) == payload["ballotHash"],
            "Recibo de outra cédula",
        )

    if kind in {"closure", "cancellation"}:
        schedule = index.get(payload["scheduleHash"])
        _require(
            schedule is not None
            and schedule["kind"] in {"calendar-origin", "schedule-change"},
            "Calendário ausente",
        )
        _require(
            schedule["electionId"] == payload["electionId"],
            "Calendário de outra eleição",
        )
        _snapshot(payload["headHash"], payload["voterCount"], manifest, index, profile)

        if kind == "closure":
            _require(
                payload["closedAt"] >= schedule["endsAt"],
                "Encerramento antes do prazo vigente",
            )

    if kind in {"ballot", "aggregate"}:
        _ordered_options(payload["ciphertexts"], manifest)

        for ciphertext in payload["ciphertexts"]:
            _element(ciphertext["a"], profile, kind == "ballot")
            _element(ciphertext["b"], profile)

    if kind == "decryption-share":
        aggregate = _artifact(index, payload["aggregateHash"], "aggregate")
        _require(
            aggregate["closureHash"] == payload["closureHash"],
            "Encerramento divergente",
        )
        _require(
            aggregate["manifestHash"] == payload["manifestHash"],
            "Agregado de outro manifesto",
        )
        authority = next(
            (x for x in manifest["authorities"] if x["id"] == payload["authorityId"]),
            None,
        )
        _require(authority is not None, "Autoridade não cadastrada")
        _ordered_options(payload["shares"], manifest)
        public = int(authority["publicValue"])

        for share, ciphertext in zip(payload["shares"], aggregate["ciphertexts"]):
            value = _element(share["value"], profile)
            proof = share["proof"]
            commitment_g = _element(proof["commitmentG"], profile, True)
            commitment_a = _element(proof["commitmentA"], profile)
            response = int(proof["response"])
            _require(0 <= response < order, "Resposta de decifração fora da faixa")
            challenge_payload = {
                "version": 1,
                "kind": "decryption-proof",
                "electionId": payload["electionId"],
                "groupId": manifest["groupId"],
                "manifestHash": payload["manifestHash"],
                "closureHash": payload["closureHash"],
                "aggregateHash": payload["aggregateHash"],
                "authorityId": payload["authorityId"],
                "optionId": share["optionId"],
                "publicValue": authority["publicValue"],
                "a": ciphertext["a"],
                "b": ciphertext["b"],
                "value": share["value"],
                "commitmentG": proof["commitmentG"],
                "commitmentA": proof["commitmentA"],
            }
            challenge = _challenge(challenge_payload, profile)
            _require(
                pow(generator, response, prime)
                == commitment_g * pow(public, challenge, prime) % prime,
                "Prova de decifração inválida",
            )
            _require(
                pow(int(ciphertext["a"]), response, prime)
                == commitment_a * pow(value, challenge, prime) % prime,
                "Contribuição de decifração inválida",
            )

    if kind == "aggregate":
        closure = _artifact(index, payload["closureHash"], "closure")
        _require(
            closure["manifestHash"] == payload["manifestHash"],
            "Encerramento de outro manifesto",
        )
        entries = _snapshot(
            closure["headHash"], closure["voterCount"], manifest, index, profile
        )

        for position, ciphertext in enumerate(payload["ciphertexts"]):
            expected_a = expected_b = 1
            for entry in entries:
                vote = entry["ballot"]["payload"]["ciphertexts"][position]
                expected_a = expected_a * int(vote["a"]) % prime
                expected_b = expected_b * int(vote["b"]) % prime
            _require(
                (int(ciphertext["a"]), int(ciphertext["b"]))
                == (expected_a, expected_b),
                "Agregado diferente do quadro",
            )

    if kind == "result" and payload["closureHash"] is not None:
        closure = _artifact(index, payload["closureHash"], "closure")
        aggregate = _artifact(index, payload["aggregateHash"], "aggregate")
        _require(
            closure["manifestHash"] == payload["manifestHash"],
            "Resultado de outro manifesto",
        )
        _require(
            aggregate["closureHash"] == payload["closureHash"],
            "Resultado de outro agregado",
        )
        _require(
            payload["voterCount"] == closure["voterCount"],
            "Contagem divergente do encerramento",
        )
        known = {x["id"] for x in manifest["authorities"]}
        _require(
            set(payload["missingAuthorities"]) <= known,
            "Autoridade pendente desconhecida",
        )

    if kind == "result" and payload["counts"] is not None:
        _ordered_options(payload["counts"], manifest)
        _require(
            all(x["count"] <= payload["voterCount"] for x in payload["counts"]),
            "Total fora do intervalo",
        )
        matches = sum(x["count"] for x in payload["counts"]) == payload["voterCount"]
        _require(
            matches == (payload["status"] == "valid"),
            "Situação incompatível com a soma",
        )


def validate_case(case, corpus, profile, schema):
    """Verifica estrutura, bytes, hash, assinatura e relações de um exemplo.

    As referências vêm do corpus público versionado. Campos esperados não são
    regenerados ao validar: uma alteração requer atualizar/revisar os vetores.
    """
    message = {"payload": case["payload"], "signature": case["signature"]}
    errors = list(
        Draft202012Validator(schema, format_checker=FormatChecker()).iter_errors(
            message
        )
    )

    if errors:
        raise ContractError(f"Schema incompatível: {errors[0].json_path}")

    payload = case["payload"]
    _require(
        rfc8785.dumps(payload).decode("utf-8") == case["canonical"],
        "Bytes canônicos divergentes",
    )
    _require(digest(payload, profile) == case["sha256"], "Hash divergente")
    index = {digest(x["payload"], profile): x["payload"] for x in corpus["cases"]}
    signer = case.get("signer", "legacy")
    keys = {"legacy": corpus["verificationKey"], **corpus.get("keys", {})}
    expected_key = keys[signer]

    if payload["kind"] == "ballot":
        _require(payload["publicKey"] == expected_key, "Chave do eleitor divergente")
    elif payload["kind"] == "decryption-share":
        manifest = _artifact(index, payload["manifestHash"], "manifest")
        authority = next(
            (x for x in manifest["authorities"] if x["id"] == payload["authorityId"]),
            None,
        )
        _require(authority is not None, "Autoridade não cadastrada")
        _require(
            authority["signingKey"] == expected_key,
            "Assinante não é a autoridade indicada",
        )
    elif signer != "legacy":
        _require(signer == "server", "Assinante não é o servidor")
        if payload["kind"] == "manifest":
            _require(
                payload["serverSigningKey"] == expected_key,
                "Chave do servidor divergente",
            )

    verify_signature(payload, case["signature"], expected_key, profile)
    if case.get("scope", "protocol") == "protocol":
        validate_relations(payload, profile, index)


def validate_invalid_vectors(directory, corpus, profile, schema):
    """Reproduz mutações versionadas na camada explicitamente indicada.

    Uma mutação de relações pode também invalidar a assinatura original; o
    objetivo desse vetor é exercitar isoladamente a regra de referência. Não
    é uma declaração de que o pacote modificado passou pelas camadas anteriores.
    """
    examples = read_json(directory / "invalid-vectors.json")["cases"]
    cases = {x["name"]: x for x in corpus["cases"]}
    index = {digest(x["payload"], profile): x["payload"] for x in corpus["cases"]}
    keys = {"legacy": corpus["verificationKey"], **corpus["keys"]}
    validator = Draft202012Validator(schema, format_checker=FormatChecker())

    for example in examples:
        stage = example["stage"]
        if stage == "json":
            with tempfile.TemporaryDirectory() as name:
                path = Path(name) / "invalid.json"
                path.write_text(example["raw"], encoding="utf-8")
                try:
                    read_json(path)
                except (ContractError, ValueError):
                    continue
            raise ContractError(f"{example['name']}: JSON inválido foi aceito")

        base = cases[example["base"]]
        payload = copy.deepcopy(base["payload"])
        target = payload
        for part in example["path"][:-1]:
            target = target[part]
        target[example["path"][-1]] = example["value"]
        errors = list(
            validator.iter_errors({"payload": payload, "signature": base["signature"]})
        )

        if stage == "schema":
            _require(bool(errors), f"{example['name']}: schema aceitou o exemplo")
            continue

        _require(not errors, f"{example['name']}: camada esperada diverge do schema")
        try:
            if stage == "signature":
                verify_signature(
                    payload, base["signature"], keys[base["signer"]], profile
                )
            elif stage == "relations":
                validate_relations(payload, profile, index)
            else:
                raise ValueError("Camada de exemplo desconhecida")
        except ContractError:
            continue
        raise ContractError(f"{example['name']}: exemplo inválido foi aceito")

    return len(examples)


def validate_contracts(directory=CONTRACTS):
    """Confere os schemas e o corpus fixado e devolve a quantidade de exemplos.

    Falhas interrompem a execução com ContractError ou erro de schema/arquivo.
    Não modifica exemplos para fazê-los concordar com a implementação.
    """
    profile = read_json(directory / "profile.json")
    profile_schema = read_json(directory / "profile.schema.json")
    Draft202012Validator.check_schema(profile_schema)
    profile_errors = list(Draft202012Validator(profile_schema).iter_errors(profile))
    _require(not profile_errors, "Perfil incompatível com a versão fixada")
    schema = read_json(directory / "schemas.json")
    Draft202012Validator.check_schema(schema)
    corpus = read_json(directory / "vectors.json")
    _require(
        hashlib.sha256(rfc8785.dumps(profile)).hexdigest() == corpus["profileSha256"],
        "Perfil e vetores pertencem a revisões diferentes",
    )
    _require(profile["signature"]["saltLengthBytes"] == 32, "Perfil RSA-PSS inválido")
    _require(profile["limits"]["minSlates"] == 1, "Chapa única deve ser permitida")
    _require(
        len({x["name"] for x in corpus["cases"]}) == len(corpus["cases"]),
        "Nomes de vetores repetidos",
    )

    for case in corpus["cases"]:
        try:
            validate_case(case, corpus, profile, schema)
        except (ContractError, KeyError, ValueError) as error:
            raise ContractError(f"{case['name']}: {error}") from error

    validate_invalid_vectors(directory, corpus, profile, schema)
    return len(corpus["cases"])


if __name__ == "__main__":
    print(f"{validate_contracts()} vetores eleitorais conferidos.")
