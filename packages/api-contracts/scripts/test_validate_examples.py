"""Testes unitários do validador de exemplos OpenAPI."""

import unittest

from validate_examples import validate_instance


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
