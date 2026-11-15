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
    """Cria uma fixture OpenAPI independente para cada comparação.

    Retorna um dicionário JSON-serializável, com versão inicial 1.0.0 e uma
    resposta que exige id e name. Remover name caracteriza uma quebra real de
    contrato, permitindo distinguir o veredito SemVer de erro ao carregar a
    configuração. Cada chamada cria também os objetos internos novamente;
    não há leitura de arquivo, Docker ou alteração do contrato do aplicativo.

    Exemplo de inspeção da fixture antes de preparar a revisão:

    >>> base = contract()
    >>> base["info"]["version"]
    '1.0.0'
    """
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
    """Executa a política configurada usando o mesmo OASDiff fixado no CI.

    Os casos variam a versão e a existência de uma remoção obrigatória. Eles
    verificam o resultado do processo real, não uma implementação Python de
    SemVer, e continuam cobrindo a configuração quando ela for alterada.
    Requerem o executável Docker, daemon acessível e a imagem oficial indicada
    em OASDIFF_IMAGE; indisponibilidade da infraestrutura falha o teste.
    """

    def compare(self, revision_version, *, breaking=True):
        """Compara uma revisão da fixture e devolve código de saída e relatório.

        Parâmetros:
            revision_version: texto atribuído a info.version. Valores inválidos
                são deliberadamente admitidos para exercitar a recusa do OASDiff.
            breaking: remove o campo obrigatório name quando verdadeiro. Quando
                falso, conserva o schema e permite testar um contrato inalterado
                ou somente uma mudança compatível da versão declarada.

        Retorno e efeitos:
            Tupla (returncode, report), cujo relatório reúne stdout e stderr.
            Código não zero é devolvido para os casos negativos, em vez de virar
            exceção por check=True. Usa a configuração real montada somente para
            leitura e contratos em diretório temporário, removido ao sair do
            contexto. O container usa --rm e rede desabilitada; não grava no repo.

        Falhas:
            FileNotFoundError se Docker não está disponível como comando;
            subprocess.TimeoutExpired se a execução ultrapassa 120 segundos.
            Em comparação com quebra, AssertionError se a remoção esperada não
            aparece no relatório: um erro de configuração não pode simular uma
            recusa correta da política. Os casos concretos conferem o código de
            saída, incluindo falhas operacionais de Docker retornadas pelo CLI.
        """
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
        """Uma remoção com a mesma versão é recusada, mesmo sem mudar metadados."""
        code, report = self.compare("1.0.0")
        self.assertEqual(code, 1, report)

    def test_breaking_change_with_minor_increase_fails(self):
        """Aumentar só MINOR não autoriza a quebra entre versões estáveis."""
        code, report = self.compare("1.1.0")
        self.assertEqual(code, 1, report)

    def test_breaking_change_with_version_decrease_fails(self):
        """A revisão não pode encobrir a quebra declarando uma versão anterior."""
        code, report = self.compare("0.9.0")
        self.assertEqual(code, 1, report)

    def test_breaking_change_with_non_semantic_version_fails(self):
        """Uma versão não comparável falha, em vez de dispensar a política."""
        code, report = self.compare("not-semver")
        self.assertEqual(code, 1, report)

    def test_breaking_change_with_major_increase_passes(self):
        """O aumento MAJOR permite a quebra e mantém sua detecção no relatório."""
        code, report = self.compare("2.0.0")
        self.assertEqual(code, 0, report)

    def test_unchanged_contract_passes(self):
        """Sem alteração de contrato ou versão, não há aumento obrigatório."""
        code, report = self.compare("1.0.0", breaking=False)
        self.assertEqual(code, 0, report)

    def test_non_breaking_minor_increase_passes(self):
        """Só alterar a versão MINOR, sem quebra, não exige aumento MAJOR."""
        code, report = self.compare("1.1.0", breaking=False)
        self.assertEqual(code, 0, report)


if __name__ == "__main__":
    unittest.main()
