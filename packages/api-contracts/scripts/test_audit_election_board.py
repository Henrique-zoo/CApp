"""Consumer regressions using independent signed contracts and hostile exports."""

import base64
from concurrent.futures import ThreadPoolExecutor
import contextlib
import copy
import io
import json
from pathlib import Path
import subprocess
import tempfile
import threading
import unittest

from audit_election_board import BoardAuditor, main, persist_anchor
from validate_election_contracts import CONTRACTS, ContractError, read_json, signing_bytes


class BoardAuditTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.corpus = read_json(CONTRACTS / "vectors.json")
        cls.cases = {case["name"]: case for case in cls.corpus["cases"]}
        cls.key = cls.corpus["keys"]["server"]
        cls.auditor = BoardAuditor(cls.key)

    def envelope(self, name):
        case = self.cases[name]
        return copy.deepcopy({"payload": case["payload"], "signature": case["signature"]})

    def snapshot(self, count=2, closed=False, scope="valid"):
        return {
            "manifest": self.envelope(f"{scope}/manifest"),
            **self.corpus["periods"][scope],
            "entries": [self.envelope(f"{scope}/entry-{i}") for i in range(count)],
            "checkpoint": self.envelope(f"{scope}/receipt-{count - 1}") if count else None,
            "closure": self.envelope(f"{scope}/closure") if closed else None,
            "aggregate": self.envelope(f"{scope}/aggregate") if closed else None,
        }

    def test_unanchored_live_prefix_is_inconclusive(self):
        report, _ = self.auditor.audit(self.snapshot(1))
        self.assertEqual(report["integrity"], "valid")
        self.assertEqual(report["continuity"], "inconclusive")
        self.assertFalse(report["matchesPublishedClosure"])
        self.assertFalse(report["previousClosureVerified"])

    def test_legitimate_append_and_close_preserve_prefix(self):
        _, anchor = self.auditor.audit(self.snapshot(1))
        report, anchor = self.auditor.audit(self.snapshot(2), anchor)
        self.assertEqual(report["continuity"], "verified")
        report, _ = self.auditor.audit(self.snapshot(2, closed=True), anchor)
        self.assertTrue(report["matchesPublishedClosure"])
        self.assertFalse(report["previousClosureVerified"])

    def test_receipt_is_independent_evidence(self):
        report, _ = self.auditor.audit(self.snapshot(2), receipt=self.envelope("valid/receipt-0"))
        self.assertEqual(report["continuity"], "verified")
        self.assertEqual(report["anchoredThroughIndex"], 1)

    def test_external_empty_checkpoint_manifest_anchors_only_configuration(self):
        report, _ = self.auditor.audit(self.snapshot(2), receipt=self.envelope("valid/manifest"))
        self.assertEqual(report["continuity"], "verified")
        self.assertEqual(report["anchoredThroughIndex"], 0)
        self.assertEqual(report["scope"], "live-prefix")

    def test_empty_manifest_anchor_allows_genuine_first_vote(self):
        _, anchor = self.auditor.audit(self.snapshot(0))
        report, _ = self.auditor.audit(self.snapshot(1), anchor)
        self.assertEqual(report["continuity"], "verified")

    def test_closed_empty_election(self):
        report, _ = self.auditor.audit(self.snapshot(0, closed=True, scope="empty"))
        self.assertEqual(report["entryCount"], 0)
        self.assertTrue(report["matchesPublishedClosure"])
        self.assertFalse(report["previousClosureVerified"])
        self.assertEqual(report["continuity"], "inconclusive")

    def test_truncated_coherent_prefix_needs_relevant_anchor(self):
        full = self.snapshot(2)
        _, anchor = self.auditor.audit(full)
        shorter = self.snapshot(1)
        report, _ = self.auditor.audit(shorter)
        self.assertEqual(report["continuity"], "inconclusive")
        with self.assertRaisesRegex(ContractError, "truncado"):
            self.auditor.audit(shorter, anchor)
        with self.assertRaisesRegex(ContractError, "truncado"):
            self.auditor.audit(shorter, receipt=self.envelope("valid/receipt-1"))

    def test_unrelated_anchor_is_inconclusive(self):
        _, unrelated = self.auditor.audit(self.snapshot(0, scope="empty"))
        report, _ = self.auditor.audit(self.snapshot(1), unrelated)
        self.assertEqual(report["continuity"], "inconclusive")
        self.assertTrue(report["irrelevantEvidence"])

    def test_missing_reordered_inserted_and_tampered_entries(self):
        original = self.snapshot(2)
        for attack in ("missing", "reordered", "inserted", "tampered"):
            with self.subTest(attack=attack):
                bad = copy.deepcopy(original)
                if attack == "missing":
                    bad["entries"].pop(0)
                elif attack == "reordered":
                    bad["entries"].reverse()
                elif attack == "inserted":
                    bad["entries"].insert(1, copy.deepcopy(bad["entries"][0]))
                else:
                    bad["entries"][0]["payload"]["ballot"]["payload"]["ciphertexts"][0]["b"] = "1"
                with self.assertRaises(ContractError):
                    self.auditor.audit(bad)

    def test_export_cannot_hide_tail_covered_by_closure(self):
        bad = self.snapshot(2, closed=True)
        bad["entries"].pop()
        bad["checkpoint"] = self.envelope("valid/receipt-0")
        with self.assertRaisesRegex(ContractError, "exatamente"):
            self.auditor.audit(bad)

    def test_closed_anchor_rejects_open_rollback(self):
        _, anchor = self.auditor.audit(self.snapshot(2, closed=True))
        report, _ = self.auditor.audit(self.snapshot(2, closed=True), anchor)
        self.assertTrue(report["previousClosureVerified"])
        with self.assertRaisesRegex(ContractError, "fechada para aberta"):
            self.auditor.audit(self.snapshot(2), anchor)

    def test_checkpoint_must_be_signed_and_must_cover_last_entry(self):
        bad = self.snapshot(2)
        bad["checkpoint"] = self.envelope("valid/receipt-0")
        with self.assertRaisesRegex(ContractError, "fim do quadro"):
            self.auditor.audit(bad)
        bad = self.snapshot(2)
        bad["checkpoint"]["signature"] = bad["entries"][0]["signature"]
        with self.assertRaises(ContractError):
            self.auditor.audit(bad)

    def test_external_receipt_signature_is_checked(self):
        receipt = self.envelope("valid/receipt-0")
        receipt["signature"] = self.envelope("valid/receipt-1")["signature"]
        with self.assertRaises(ContractError):
            self.auditor.audit(self.snapshot(2), receipt=receipt)

    def test_period_must_be_consistent_with_the_export(self):
        inverted = self.snapshot()
        inverted["endsAt"] = inverted["startsAt"]
        with self.assertRaisesRegex(ContractError, "Período"):
            self.auditor.audit(inverted)

        closed = self.snapshot(closed=True)
        closed["endsAt"] = "2026-12-31T12:00:00Z"
        with self.assertRaisesRegex(ContractError, "Fechamento anterior ao fim informado"):
            self.auditor.audit(closed)

    def test_cli_keeps_anchor_unchanged_after_failure_and_other_scope(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            export, key, anchor = (directory / name for name in ("export.json", "server.key", "anchor.json"))
            key.write_text(self.key, encoding="utf-8")
            argv = ["--snapshot", str(export), "--trusted-key", str(key), "--checkpoint", str(anchor)]
            export.write_text(json.dumps(self.snapshot(2)), encoding="utf-8")
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(main(argv), 3)
            saved = anchor.read_bytes()
            export.write_text(json.dumps(self.snapshot(1)), encoding="utf-8")
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(main(argv), 2)
            self.assertEqual(anchor.read_bytes(), saved)
            export.write_text(json.dumps(self.snapshot(0, scope="empty")), encoding="utf-8")
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(main(argv), 3)
            self.assertEqual(anchor.read_bytes(), saved)
            export.write_text(json.dumps(self.snapshot(2, closed=True)), encoding="utf-8")
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(main(argv), 0)
            self.assertIsNotNone(read_json(anchor)["closure"])

    def test_duplicate_json_and_input_alias_do_not_create_checkpoint(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            export, key, anchor = (directory / name for name in ("export.json", "server.key", "anchor.json"))
            export.write_text('{"entries": [], "entries": []}', encoding="utf-8")
            key.write_text(self.key, encoding="utf-8")
            argv = ["--snapshot", str(export), "--trusted-key", str(key), "--checkpoint", str(anchor)]
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(main(argv), 2)
            self.assertFalse(anchor.exists())
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(main(argv[:-1] + [str(export)]), 2)
            self.assertIn('"entries"', export.read_text(encoding="utf-8"))

    def test_duplicate_json_in_checkpoint_is_preserved(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            export, key, anchor = (directory / name for name in ("export.json", "server.key", "anchor.json"))
            export.write_text(json.dumps(self.snapshot(1)), encoding="utf-8")
            key.write_text(self.key, encoding="utf-8")
            duplicate = b'{"format": "capp-board-anchor-v1", "format": "different"}'
            anchor.write_bytes(duplicate)
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(main(["--snapshot", str(export), "--trusted-key", str(key),
                                       "--checkpoint", str(anchor)]), 2)
            self.assertEqual(anchor.read_bytes(), duplicate)

    def test_concurrent_and_stale_writers_cannot_replace_newer_anchor(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "anchor.json"
            previous = b'{"old": true}'
            path.write_bytes(previous)
            barrier = threading.Barrier(2)

            def writer(value):
                barrier.wait(timeout=5)
                try:
                    persist_anchor(path, {"winner": value}, previous)
                    return value
                except ContractError:
                    return None

            with ThreadPoolExecutor(max_workers=2) as executor:
                outcomes = list(executor.map(writer, (1, 2)))
            winners = [value for value in outcomes if value is not None]
            self.assertEqual(len(winners), 1)
            self.assertEqual(read_json(path), {"winner": winners[0]})
            saved = path.read_bytes()
            with self.assertRaisesRegex(ContractError, "mudou"):
                persist_anchor(path, {"stale": True}, previous)
            self.assertEqual(path.read_bytes(), saved)
            self.assertFalse(path.with_name(path.name + ".lock").exists())


class SignedForkTests(unittest.TestCase):
    """A malicious signer can rewrite a coherent chain; local evidence exposes it.

    Fixture payloads/proofs originate in the independent contract corpus. Only
    temporary RSA keys and signatures are generated here, never production keys.
    """

    envelope = BoardAuditTests.envelope
    snapshot = BoardAuditTests.snapshot

    @classmethod
    def setUpClass(cls):
        BoardAuditTests.setUpClass.__func__(cls)
        cls.temporary = tempfile.TemporaryDirectory()
        cls.directory = Path(cls.temporary.name)
        cls.private = cls.directory / "server.pem"
        cls.voter_private = cls.directory / "voter.pem"
        for private in (cls.private, cls.voter_private):
            cls.openssl("genpkey", "-algorithm", "RSA", "-pkeyopt", "rsa_keygen_bits:2048", "-out", private)
        cls.new_key = cls.public(cls.private)
        cls.voter_key = cls.public(cls.voter_private)
        cls.fork_auditor = BoardAuditor(cls.new_key)

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    @staticmethod
    def openssl(*args):
        return subprocess.run(["openssl", *map(str, args)], check=True, capture_output=True, timeout=30).stdout

    @classmethod
    def public(cls, private):
        return base64.urlsafe_b64encode(cls.openssl("pkey", "-in", private, "-pubout", "-outform", "DER")).rstrip(b"=").decode()

    def sign(self, payload, private=None):
        message = self.directory / "message.bin"
        message.write_bytes(signing_bytes(payload, self.auditor.profile))
        signature = self.openssl(
            "dgst", "-sha256", "-sign", private or self.private,
            "-sigopt", "rsa_padding_mode:pss", "-sigopt", "rsa_pss_saltlen:32",
            "-sigopt", "rsa_mgf1_md:sha256", message,
        )
        return {"payload": payload, "signature": base64.urlsafe_b64encode(signature).rstrip(b"=").decode()}

    def signed_snapshot(self, reverse=False, closed=False, same_voter=False):
        result = self.snapshot(2)
        manifest = result["manifest"]["payload"]
        manifest["serverSigningKey"] = self.new_key
        result["manifest"] = self.sign(manifest)
        manifest_hash = self.auditor.hash(manifest)
        head = manifest_hash
        if reverse:
            result["entries"].reverse()
        first_ballot = copy.deepcopy(result["entries"][0]["payload"]["ballot"]["payload"])
        for position, envelope in enumerate(result["entries"], 1):
            entry = envelope["payload"]
            entry.update(manifestHash=manifest_hash, previousHash=head, index=position)
            ballot = entry["ballot"]["payload"]
            ballot.update(manifestHash=manifest_hash, publicKey=self.voter_key)
            if same_voter:
                ballot["accountId"] = first_ballot["accountId"]
            entry["ballot"] = self.sign(ballot, self.voter_private)
            result["entries"][position - 1] = self.sign(entry)
            head = self.auditor.hash(entry)
        receipt = result["checkpoint"]["payload"]
        receipt.update(manifestHash=manifest_hash, entryHash=head,
                       ballotHash=self.auditor.hash(result["entries"][-1]["payload"]["ballot"]["payload"]))
        result["checkpoint"] = self.sign(receipt)
        if closed:
            closure = self.envelope("valid/closure")["payload"]
            closure.update(manifestHash=manifest_hash, headHash=head)
            result["closure"] = self.sign(closure)
            aggregate = self.envelope("valid/aggregate")["payload"]
            aggregate.update(manifestHash=manifest_hash, closureHash=self.auditor.hash(closure))
            result["aggregate"] = self.sign(aggregate)
        return result

    def test_fully_resigned_rewritten_chain_requires_anchor_to_detect(self):
        honest = self.signed_snapshot()
        rewritten = self.signed_snapshot(reverse=True)
        _, anchor = self.fork_auditor.audit(honest)
        report, _ = self.fork_auditor.audit(rewritten)
        self.assertEqual(report["continuity"], "inconclusive")
        with self.assertRaises(ContractError):
            self.fork_auditor.audit(rewritten, anchor)

    def test_current_store_shape_with_unsigned_period(self):
        report, _ = self.fork_auditor.audit(self.signed_snapshot(closed=True))
        self.assertTrue(report["matchesPublishedClosure"])

    def test_signed_duplicate_voter_is_rejected(self):
        with self.assertRaisesRegex(ContractError, "Eleitor repetido"):
            self.fork_auditor.audit(self.signed_snapshot(same_voter=True))


if __name__ == "__main__":
    unittest.main()
