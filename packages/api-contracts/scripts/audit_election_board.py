"""Audit a real exported board against independently retained signed evidence.

This consumer never contacts the backend, imports its Rust implementation, or
uses the fixture corpus at runtime. Wire primitives are shared with the Python
contract validator. A valid live prefix is not proof of current completeness.
"""

import argparse
from datetime import datetime
import hashlib
import json
import os
from pathlib import Path
import tempfile

from jsonschema import Draft202012Validator, FormatChecker

from validate_election_contracts import (
    CONTRACTS, ContractError, binary, digest, read_json, signing_bytes,
    validate_relations, verify_signature,
)


def require(condition, message):
    if not condition:
        raise ContractError(message)


def election_time(value):
    """Read election metadata as canonical UTC with second precision."""
    if isinstance(value, str) and value.endswith("Z"):
        try:
            parsed = datetime.fromisoformat(value)
            if parsed.isoformat(timespec="seconds").replace("+00:00", "Z") == value:
                return parsed
        except ValueError:
            pass
    raise ContractError("Horário eleitoral inválido")


class BoardAuditor:
    """Verify signatures, the complete supplied chain and external anchors."""

    def __init__(self, trusted_key):
        self.key = trusted_key
        self.key_hash = hashlib.sha256(binary(trusted_key)).hexdigest()
        self.profile = read_json(CONTRACTS / "profile.json")
        self.validator = Draft202012Validator(
            read_json(CONTRACTS / "schemas.json"), format_checker=FormatChecker()
        )

    def signed(self, envelope, kind):
        error = next(self.validator.iter_errors(envelope), None)
        require(error is None, f"Schema inválido para {kind}")
        payload = envelope["payload"]
        require(payload["kind"] == kind, f"Esperado objeto {kind}")
        require(
            len(signing_bytes(payload, self.profile))
            <= self.profile["limits"]["maxSignedMessageBytes"],
            "Mensagem assinada excede o limite do perfil",
        )
        verify_signature(payload, envelope["signature"], self.key, self.profile)
        return payload

    def hash(self, payload):
        return digest(payload, self.profile)

    def inspect(self, snapshot):
        """Validate the export itself, without inferring historical continuity."""
        required = {
            "manifest", "startsAt", "endsAt", "entries", "checkpoint", "closure",
            "aggregate",
        }
        require(isinstance(snapshot, dict), "Exportação deve ser objeto JSON")
        require(required <= snapshot.keys(), "Exportação incompleta")
        require(snapshot.keys() == required, "Campo desconhecido na exportação")
        starts_at = election_time(snapshot["startsAt"])
        ends_at = election_time(snapshot["endsAt"])
        require(starts_at < ends_at, "Período da votação inválido")
        entries = snapshot["entries"]
        require(isinstance(entries, list), "entries deve ser lista")
        require(len(entries) <= self.profile["limits"]["maxVoters"], "Quadro excessivo")
        manifest = self.signed(snapshot["manifest"], "manifest")
        require(manifest["serverSigningKey"] == self.key, "Chave do manifesto difere da chave confiável")
        manifest_hash = self.hash(manifest)
        election_id = manifest["electionId"]
        index = {manifest_hash: manifest}
        validate_relations(manifest, self.profile, index)

        head = manifest_hash
        voters = set()
        for position, envelope in enumerate(entries, 1):
            entry = self.signed(envelope, "board-entry")
            require(entry["index"] == position, "Entrada ausente, inserida ou reordenada")
            require(entry["previousHash"] == head, "Cadeia do quadro divergente")
            require(entry["manifestHash"] == manifest_hash, "Entrada de outro manifesto")
            # Includes ballot schema, voter signature, group and option order.
            validate_relations(entry, self.profile, index)
            ballot = entry["ballot"]["payload"]
            require(ballot["accountId"] not in voters, "Eleitor repetido no quadro")
            voters.add(ballot["accountId"])
            head = self.hash(entry)
            index[head] = entry

        receipt = snapshot["checkpoint"]
        if entries:
            require(receipt is not None, "Último recibo ausente")
            checkpoint = self.signed(receipt, "receipt")
            validate_relations(checkpoint, self.profile, index)
            require(checkpoint["index"] == len(entries) and checkpoint["entryHash"] == head,
                    "Checkpoint não corresponde ao fim do quadro")
        else:
            require(receipt is None, "Quadro vazio não tem recibo")

        closure = snapshot["closure"]
        aggregate = snapshot["aggregate"]
        require((closure is None) == (aggregate is None), "Fechamento/agregado incompletos")
        if closure is not None:
            closed = self.signed(closure, "closure")
            require(closed["manifestHash"] == manifest_hash, "Fechamento de outro manifesto")
            require(closed["headHash"] == head and closed["voterCount"] == len(entries),
                    "Fechamento não cobre exatamente o quadro fornecido")
            # As datas vêm da exportação. A assinatura cobre closedAt.
            require(election_time(closed["closedAt"]) >= ends_at,
                    "Fechamento anterior ao fim informado na exportação")
            validate_relations(closed, self.profile, index)
            index[self.hash(closed)] = closed
            total = self.signed(aggregate, "aggregate")
            require(total["closureHash"] == self.hash(closed), "Agregado de outro fechamento")
            validate_relations(total, self.profile, index)

        return {
            "electionId": election_id, "manifestHash": manifest_hash,
            "headHash": head, "entryCount": len(entries),
            "scope": "closed" if closure is not None else "live-prefix",
            "index": index,
        }

    def evidence(self, snapshot):
        """The local wrapper is not a new signed wire kind or a server file."""
        return {
            "format": "capp-board-anchor-v2", "trustedKeySha256": self.key_hash,
            "manifest": snapshot["manifest"],
            "checkpoint": snapshot["checkpoint"], "closure": snapshot["closure"],
        }

    def compare_receipt(self, receipt, state):
        # /checkpoint returns the manifest when there is no receipt yet. This
        # anchors configuration only; the manifest does not attest to emptiness.
        is_manifest = (isinstance(receipt, dict)
                       and isinstance(receipt.get("payload"), dict)
                       and receipt["payload"].get("kind") == "manifest")
        payload = self.signed(receipt, "manifest" if is_manifest else "receipt")
        if payload["electionId"] != state["electionId"]:
            return False
        if is_manifest:
            require(self.hash(payload) == state["manifestHash"], "Manifesto congelado foi substituído")
            return True
        require(payload["manifestHash"] == state["manifestHash"], "Manifesto foi substituído desde o recibo")
        require(payload["index"] <= state["entryCount"], "Quadro truncado antes do recibo externo")
        validate_relations(payload, self.profile, state["index"])
        return True

    def compare_anchor(self, anchor, snapshot, state):
        fields = {"format", "trustedKeySha256", "manifest", "checkpoint", "closure"}
        require(isinstance(anchor, dict) and anchor.keys() == fields,
                "Arquivo de âncora inválido")
        require(anchor["format"] == "capp-board-anchor-v2", "Versão de âncora desconhecida")
        require(anchor["trustedKeySha256"] == self.key_hash, "Âncora fixou outra chave confiável")
        old_manifest = self.signed(anchor["manifest"], "manifest")
        if old_manifest["electionId"] != state["electionId"]:
            return False
        require(self.hash(old_manifest) == state["manifestHash"], "Manifesto congelado foi substituído")
        if anchor["checkpoint"] is not None:
            require(self.compare_receipt(anchor["checkpoint"], state), "Recibo de outra eleição na âncora")
        if anchor["closure"] is not None:
            old_closure = self.signed(anchor["closure"], "closure")
            require(snapshot["closure"] is not None, "Regressão de eleição fechada para aberta")
            require(self.hash(old_closure) == self.hash(snapshot["closure"]["payload"]),
                    "Fechamento foi substituído")
        return True

    def audit(self, snapshot, anchor=None, receipt=None):
        state = self.inspect(snapshot)
        relevant = 0
        irrelevant = False
        if anchor is not None:
            matched = self.compare_anchor(anchor, snapshot, state)
            relevant += matched
            irrelevant |= not matched
        if receipt is not None:
            matched = self.compare_receipt(receipt, state)
            relevant += matched
            irrelevant |= not matched
        report = {key: value for key, value in state.items() if key != "index"}
        anchored_indices = []
        if anchor is not None and anchor["manifest"]["payload"]["electionId"] == state["electionId"]:
            anchored_indices.append(anchor["checkpoint"]["payload"].get("index", 0)
                                    if anchor["checkpoint"] is not None else 0)
        if receipt is not None and receipt["payload"]["electionId"] == state["electionId"]:
            anchored_indices.append(receipt["payload"].get("index", 0))
        report.update({
            "integrity": "valid",
            "continuity": "verified" if relevant else "inconclusive",
            "irrelevantEvidence": irrelevant,
            "anchoredThroughIndex": max(anchored_indices) if anchored_indices else None,
            "matchesPublishedClosure": state["scope"] == "closed",
            "previousClosureVerified": (
                anchor is not None
                and anchor["closure"] is not None
                and anchor["manifest"]["payload"]["electionId"] == state["electionId"]
                and state["scope"] == "closed"
            ),
            "limitations": [
                "Sem âncora prévia relevante não é possível excluir reescrita ou truncamento coerentes.",
                "Um prefixo ativo não prova que o servidor mostrou todos os votos atuais.",
                "Assinaturas não demonstram elegibilidade Firebase nem cédula individual 0/1; valem as premissas do perfil.",
            ],
        })
        return report, self.evidence(snapshot)


def persist_anchor(path, evidence, expected):
    """Serialize compare-and-replace with an exclusive portable sidecar lock."""
    path = Path(path)
    lock = path.with_name(path.name + ".lock")
    try:
        descriptor = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    except FileExistsError as error:
        raise ContractError("Âncora em uso: lock exclusivo existente; execute novamente") from error
    temporary = None
    try:
        os.close(descriptor)
        current = path.read_bytes() if path.exists() else None
        require(current == expected, "Âncora mudou durante a auditoria; execute novamente")
        data = (json.dumps(evidence, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
        with tempfile.NamedTemporaryFile(dir=path.parent, prefix=".capp-anchor-", delete=False) as output:
            temporary = Path(output.name)
            output.write(data)
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, path)
    finally:
        if temporary is not None and temporary.exists():
            temporary.unlink()
        lock.unlink()


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--snapshot", required=True, type=Path)
    parser.add_argument("--trusted-key", required=True, type=Path,
                        help="Arquivo de texto SPKI DER base64url, obtido por canal independente")
    parser.add_argument("--checkpoint", required=True, type=Path,
                        help="Âncora local fora do servidor (criada se ainda não existir)")
    parser.add_argument("--receipt", type=Path,
                        help="Recibo prévio assinado, ou manifesto do checkpoint vazio")
    args = parser.parse_args(argv)
    try:
        checkpoint_path = args.checkpoint.resolve()
        inputs = [args.snapshot, args.trusted_key] + ([args.receipt] if args.receipt else [])
        require(all(checkpoint_path != path.resolve() for path in inputs),
                "Âncora deve ser arquivo distinto das entradas")
        previous = args.checkpoint.read_bytes() if args.checkpoint.exists() else None
        anchor = read_json(args.checkpoint) if previous is not None else None
        auditor = BoardAuditor(args.trusted_key.read_text(encoding="utf-8").strip())
        report, evidence = auditor.audit(
            read_json(args.snapshot), anchor,
            read_json(args.receipt) if args.receipt else None,
        )
        # An unrelated prior file must never silently change election scope.
        if report["irrelevantEvidence"]:
            report["anchorUpdated"] = False
        else:
            persist_anchor(args.checkpoint, evidence, previous)
            report["anchorUpdated"] = True
        print(json.dumps(report, ensure_ascii=False, indent=2))
        return 0 if report["continuity"] == "verified" else 3
    except (ContractError, OSError, ValueError) as error:
        print(json.dumps({"integrity": "invalid", "error": str(error), "anchorUpdated": False},
                         ensure_ascii=False))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
