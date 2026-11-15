"""Regressões da política SemVer executada pelo OASDiff real.

O CI e a reprodução local precisam reconhecer as mesmas regras. Estes testes
usam a imagem oficial 1.38.0 e a configuração versionada do pacote, em vez de
reimplementar a decisão de compatibilidade em Python. Uma remoção real de campo
obrigatório comprova tanto a detecção da quebra quanto o efeito de info.version
sobre o código de saída. Os arquivos de cada comparação são descartáveis e os
containers executam sem rede; Docker e a imagem oficial são necessários.
"""

import copy
import json
from pathlib import Path
import subprocess
import tempfile
import unittest


CONTRACTS_DIR = Path(__file__).resolve().parents[1]
OASDIFF_IMAGE = "tufin/oasdiff:v1.38.0"


def contract():
    """Contrato pequeno com um campo obrigatório que a revisão pode remover."""
    return {
        "openapi": "3.1.0",
        "info": {"title": "Versioning policy fixture", "version": "1.0.0"},
        "paths": {
            "/resource": {
                "get": {
                    "responses": {
                        "200": {
                            "description": "Resource",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "required": ["id", "name"],
                                        "properties": {
                                            "id": {"type": "string"},
                                            "name": {"type": "string"},
                                        },
                                    }
                                }
                            },
                        }
                    }
                }
            }
        },
    }


class OasdiffPolicyTests(unittest.TestCase):
    def compare(self, revision_version, *, breaking=True):
        """Executa a configuração real com uma revisão controlada da fixture."""
        base = contract()
        revision = copy.deepcopy(base)
        revision["info"]["version"] = revision_version

        if breaking:
            schema = revision["paths"]["/resource"]["get"]["responses"]["200"][
                "content"
            ]["application/json"]["schema"]
            del schema["properties"]["name"]
            schema["required"].remove("name")

        with tempfile.TemporaryDirectory(prefix="capp-oasdiff-policy-") as work:
            for name, document in [("base", base), ("revision", revision)]:
                Path(work, f"{name}.json").write_text(
                    json.dumps(document), encoding="utf-8"
                )

            result = subprocess.run(
                [
                    "docker", "run", "--rm", "--network", "none",
                    "-e", "OASDIFF_CONFIG=/spec/.oasdiff.yaml",
                    "-v", f"{CONTRACTS_DIR}:/spec:ro",
                    "-v", f"{work}:/cases:ro",
                    OASDIFF_IMAGE, "breaking",
                    "/cases/base.json", "/cases/revision.json",
                ],
                capture_output=True,
                text=True,
                timeout=120,
                check=False,
            )

        report = result.stdout + result.stderr

        if breaking:
            self.assertIn("response-required-property-removed", report, report)

        return result.returncode, report

    def test_breaking_change_without_version_increase_fails(self):
        code, report = self.compare("1.0.0")
        self.assertEqual(code, 1, report)

    def test_breaking_change_with_minor_increase_fails(self):
        code, report = self.compare("1.1.0")
        self.assertEqual(code, 1, report)

    def test_breaking_change_with_version_decrease_fails(self):
        code, report = self.compare("0.9.0")
        self.assertEqual(code, 1, report)

    def test_breaking_change_with_non_semantic_version_fails(self):
        code, report = self.compare("not-semver")
        self.assertEqual(code, 1, report)

    def test_breaking_change_with_major_increase_passes(self):
        code, report = self.compare("2.0.0")
        self.assertEqual(code, 0, report)

    def test_unchanged_contract_passes(self):
        code, report = self.compare("1.0.0", breaking=False)
        self.assertEqual(code, 0, report)

    def test_non_breaking_minor_increase_passes(self):
        code, report = self.compare("1.1.0", breaking=False)
        self.assertEqual(code, 0, report)


if __name__ == "__main__":
    unittest.main()
