"""Testes unitários do validador de exemplos OpenAPI."""

import json
import unittest

from validate_examples import SPEC_PATH, contract_registry, load_spec, pointer, validate_instance


class ElectionBallotEndpointTests(unittest.TestCase):
    def test_submission_and_receipt_match_the_signed_protocol(self):
        spec = load_spec()
        registry = contract_registry(spec)
        path = ["paths", "/elections/{id}/ballots", "post"]
        with (SPEC_PATH.parent / "elections" / "vectors.json").open(encoding="utf-8") as f:
            envelopes = {
                c["name"]: {"payload": c["payload"], "signature": c["signature"]}
                for c in json.load(f)["cases"]
            }

        def errors(value, *parts):
            location = pointer([*path, *parts, "content", "application/json", "schema"])
            return validate_instance(value, {"$ref": f"{SPEC_PATH.as_uri()}#{location}"}, registry)

        evidence = envelopes["valid/ballot-0"]
        ballot = {
            "payload": {
                field: evidence["payload"][field]
                for field in ("manifestHash", "publicKey", "ciphertexts")
            },
            "signature": evidence["signature"],
        }
        self.assertFalse(errors(ballot, "requestBody"))
        self.assertTrue(errors(evidence, "requestBody"))
        for field in ("accountId", "electionId"):
            with self.subTest(field=field):
                self.assertTrue(errors(
                    {**ballot, "payload": {**ballot["payload"], field: evidence["payload"][field]}},
                    "requestBody",
                ))
        self.assertTrue(
            errors(
                {**ballot, "payload": {**ballot["payload"], "submissionId": "removed"}},
                "requestBody",
            )
        )
        entry = envelopes["valid/entry-0"]
        receipt = envelopes["valid/receipt-0"]
        for status, recovered in (("201", False), ("200", True)):
            result = {"entry": entry, "receipt": receipt, "recovered": recovered}
            self.assertFalse(errors(result, "responses", status))
            self.assertTrue(errors({**result, "receipt": entry}, "responses", status))
            self.assertTrue(errors({**result, "recovered": not recovered}, "responses", status))


class ValidateInstanceTests(unittest.TestCase):
    def setUp(self):
        self.schema = {
            "type": "object",
            "required": [
                "id",
                "name",
                "status",
            ],
            "properties": {
                "id": {
                    "type": "string",
                    "format": "uuid",
                },
                "name": {
                    "type": "string",
                },
                "status": {
                    "type": "string",
                    "const": "active",
                },
                "expiresAt": {
                    "type": "string",
                    "format": "date-time",
                },
            },
            "additionalProperties": False,
        }

    def test_accepts_valid_example(self):
        example = {
            "id": "00000000-0000-0000-0000-000000000001",
            "name": "Pessoa Exemplo",
            "status": "active",
            "expiresAt": "2026-10-22T12:00:00Z",
        }

        errors = validate_instance(
            example,
            self.schema,
        )

        self.assertEqual(errors, [])

    def test_rejects_missing_required_field(self):
        example = {
            "id": "00000000-0000-0000-0000-000000000001",
            "name": "Pessoa Exemplo",
        }

        errors = validate_instance(
            example,
            self.schema,
        )

        self.assertTrue(errors)

    def test_rejects_wrong_field_type(self):
        example = {
            "id": "00000000-0000-0000-0000-000000000001",
            "name": 123,
            "status": "active",
        }

        errors = validate_instance(
            example,
            self.schema,
        )

        self.assertTrue(errors)

    def test_rejects_invalid_const(self):
        example = {
            "id": "00000000-0000-0000-0000-000000000001",
            "name": "Pessoa Exemplo",
            "status": "inactive",
        }

        errors = validate_instance(
            example,
            self.schema,
        )

        self.assertTrue(errors)

    def test_rejects_invalid_uuid(self):
        example = {
            "id": "not-a-uuid",
            "name": "Pessoa Exemplo",
            "status": "active",
        }

        errors = validate_instance(
            example,
            self.schema,
        )

        self.assertTrue(errors)

    def test_rejects_invalid_date_time(self):
        example = {
            "id": "00000000-0000-0000-0000-000000000001",
            "name": "Pessoa Exemplo",
            "status": "active",
            "expiresAt": "not-a-date",
        }

        errors = validate_instance(
            example,
            self.schema,
        )

        self.assertTrue(errors)


if __name__ == "__main__":
    unittest.main()
