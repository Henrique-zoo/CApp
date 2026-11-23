"""Regressões dos contratos eleitorais antes da implementação dos participantes.

O corpus contém mensagens públicas com assinaturas/provas reais e cenários de
contagem conhecidos. Estes testes conferem interoperabilidade e limitações do
contrato; não simulam respostas HTTP, App Check ou regras de um servidor pronto.
Os cenários BDD do produto permanecem pendentes até existir automação real.
"""

import copy
import json
from pathlib import Path
import tempfile
import unittest

from jsonschema import Draft202012Validator, FormatChecker
import rfc8785

from validate_election_contracts import (
    CONTRACTS,
    ContractError,
    binary,
    digest,
    read_json,
    signing_bytes,
    validate_contracts,
    validate_relations,
    verify_signature,
    validate_invalid_vectors,
)


class ElectionContractTests(unittest.TestCase):
    """Exercita bytes fixados, relações entre formatos e falhas independentes."""

    @classmethod
    def setUpClass(cls):
        """Carrega material público uma vez, sem criar identidades ou segredos."""
        cls.profile = read_json(CONTRACTS / "profile.json")
        cls.schema = read_json(CONTRACTS / "schemas.json")
        cls.corpus = read_json(CONTRACTS / "vectors.json")
        cls.cases = {x["name"]: x for x in cls.corpus["cases"]}
        cls.index = {
            digest(x["payload"], cls.profile): x["payload"] for x in cls.corpus["cases"]
        }
        cls.validator = Draft202012Validator(cls.schema, format_checker=FormatChecker())

    def payload(self, name):
        """Entrega cópia editável de um vetor sem contaminar outros testes."""
        return copy.deepcopy(self.cases[name]["payload"])

    def relations(self, payload, index=None):
        """Executa a validação de relações sobre referências públicas conhecidas."""
        validate_relations(
            payload, self.profile, self.index if index is None else index
        )

    def test_shared_vectors(self):
        """Todos os exemplos fixados conservam schema, bytes, hash e assinatura."""
        self.assertEqual(validate_contracts(), len(self.corpus["cases"]))

    def test_every_message_has_a_vector(self):
        """Uma nova variante não fica sem exemplo de intercâmbio compartilhado."""
        names = [
            x["$ref"].split("/")[-1] for x in self.schema["$defs"]["Payload"]["oneOf"]
        ]
        expected = {
            self.schema["$defs"][x]["properties"]["kind"]["const"] for x in names
        }
        self.assertEqual(expected, {x["payload"]["kind"] for x in self.corpus["cases"]})

    def test_invalid_vectors_are_rejected(self):
        """Outras implementações podem reproduzir as mesmas recusas por camada."""
        examples = read_json(CONTRACTS / "invalid-vectors.json")["cases"]
        self.assertEqual(
            validate_invalid_vectors(CONTRACTS, self.corpus, self.profile, self.schema),
            len(examples),
        )

    def test_single_slate_has_three_authorities(self):
        """Chapa única usa servidor, comissão e chapa, além da opção Nulo."""
        value = self.payload("single-slate/manifest")
        self.assertEqual(len(value["options"]), 2)
        self.assertEqual(len(value["authorities"]), 3)
        self.relations(value)

    def test_schema_rejects_invalid_wire_representations(self):
        """Representações numéricas não canônicas e campos extras são recusados."""
        for key, value in [
            ("a", 2),
            ("a", ""),
            ("a", "02"),
            ("a", "+1"),
            ("b", "-1"),
            ("a", "1e3"),
            ("b", "1_000"),
            ("a", "1" + "0" * 617),
        ]:
            payload = self.payload("valid/ballot-0")
            payload["ciphertexts"][0][key] = value
            self.assertTrue(
                list(
                    self.validator.iter_errors(
                        {"payload": payload, "signature": "fixture"}
                    )
                )
            )
        payload = self.payload("valid/ballot-0")
        payload["extra"] = True
        self.assertTrue(
            list(
                self.validator.iter_errors({"payload": payload, "signature": "fixture"})
            )
        )

    def test_json_rejects_duplicate_keys_and_nonfinite_numbers(self):
        """A detecção acontece antes de json.loads perder uma chave repetida."""
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "input.json"
            for text in [
                '{"a":1,"a":2}',
                '{"a":NaN}',
                '{"a":Infinity}',
                '{"a":1.0}',
                '{"a":1e0}',
                '{"a":-0}',
            ]:
                path.write_text(text)
                with self.assertRaises((ContractError, ValueError)):
                    read_json(path)

    def test_jcs_uses_utf16_key_order(self):
        """JCS diverge da ordenação ingênua por pontos de código Unicode."""
        value = {"\ue000": 1, "\U00010000": 2}
        expected = '{"\U00010000":2,"\ue000":1}'.encode()
        self.assertTrue(signing_bytes(value, self.profile).endswith(expected))
        self.assertNotEqual(
            json.dumps(
                value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
            ).encode(),
            expected,
        )

    def test_hash_domain_includes_kind_and_election(self):
        """Uma assinatura/identificação não pode ser transplantada entre contextos."""
        value = self.payload("valid/receipt-0")
        original = digest(value, self.profile)
        value["kind"] = "closure"
        self.assertNotEqual(original, digest(value, self.profile))
        value = self.payload("valid/receipt-0")
        value["electionId"] = "99999999-1111-4111-8111-111111111111"
        self.assertNotEqual(original, digest(value, self.profile))

    def test_signature_rejects_mutation(self):
        """Uma cédula alterada falha com a assinatura original e chave pública real."""
        case = self.cases["valid/ballot-0"]
        value = copy.deepcopy(case["payload"])
        value["submissionId"] = "99999999-1111-4111-8111-111111111111"
        with self.assertRaises(ContractError):
            verify_signature(value, case["signature"], value["publicKey"], self.profile)

    def test_signature_rejects_other_public_key(self):
        """A chave pública participa do digest e precisa corresponder à assinatura."""
        case = self.cases["valid/ballot-0"]
        other_key = self.corpus["keys"]["voter-1"]
        altered = {**case["payload"], "publicKey": other_key}
        self.assertNotEqual(case["sha256"], digest(altered, self.profile))

        with self.assertRaises(ContractError):
            verify_signature(
                case["payload"],
                case["signature"],
                other_key,
                self.profile,
            )

    def test_base64url_is_canonical(self):
        """Padding e bits finais não canônicos são recusados, não corrigidos."""
        for text in ["YQ==", "YR", "a+", "a/"]:
            with self.assertRaises(ContractError):
                binary(text)

    def test_authority_proof_rejects_changed_share(self):
        """A prova associa a contribuição ao segredo e ao agregado declarados."""
        value = self.payload("valid/share-0")
        value["shares"][0]["value"] = "1"
        with self.assertRaises(ContractError):
            self.relations(value)

    def test_authority_registration_rejects_changed_key(self):
        """A prova de conhecimento impede aceitar uma chave alterada no manifesto."""
        value = self.payload("valid/manifest")
        value["authorities"][0]["publicValue"] = "2"
        with self.assertRaises(ContractError):
            self.relations(value)

    def test_option_order_is_binding(self):
        """Reordenar opções muda o significado e é recusado mesmo com cifras válidas."""
        value = self.payload("valid/ballot-0")
        value["ciphertexts"].reverse()
        with self.assertRaises(ContractError):
            self.relations(value)

    def test_ballots_and_decryption_shares_cannot_cross_elections(self):
        """Formato e ordem compatíveis não permitem reaproveitar artefatos de outra eleição."""
        entry = self.payload("valid/entry-0")
        entry["ballot"] = self.payload("excess/entry-0")["ballot"]
        with self.assertRaisesRegex(ContractError, "Cédula de outro manifesto"):
            self.relations(entry)

        share = self.payload("valid/share-0")
        foreign = self.cases["excess/aggregate"]
        share["aggregateHash"] = foreign["sha256"]
        share["closureHash"] = foreign["payload"]["closureHash"]
        with self.assertRaisesRegex(ContractError, "Agregado de outro manifesto"):
            self.relations(share)

    def test_snapshot_rejects_broken_chain(self):
        """A fotografia do encerramento deve alcançar a gênese com índices consecutivos."""
        value = self.payload("valid/entry-1")
        value["previousHash"] = value["manifestHash"]
        with self.assertRaises(ContractError):
            self.relations(value)

    def test_snapshot_rejects_duplicate_voter(self):
        """Recusa duas cédulas com o mesmo UUID de conta CApp no quadro."""
        closure = self.payload("valid/closure")
        last = copy.deepcopy(self.index[closure["headHash"]])
        previous = self.index[last["previousHash"]]
        last["ballot"]["payload"]["voterId"] = previous["ballot"]["payload"]["voterId"]
        index = {**self.index, closure["headHash"]: last}
        with self.assertRaisesRegex(ContractError, "Conta repetida no quadro"):
            self.relations(closure, index)

    def test_receipt_matches_record(self):
        """O recibo identifica a mesma eleição, posição e cédula do quadro público."""
        other_entry = self.cases["excess/entry-0"]
        other_ballot = self.payload("valid/entry-1")["ballot"]["payload"]

        for field, replacement, error in (
            ("index", 2, "Índice do recibo divergente"),
            ("ballotHash", digest(other_ballot, self.profile), "Recibo de outra cédula"),
            ("entryHash", other_entry["sha256"], "Recibo de outro manifesto"),
            ("electionId", other_entry["payload"]["electionId"], "Eleição divergente"),
        ):
            with self.subTest(field=field):
                receipt = self.payload("valid/receipt-0")
                receipt[field] = replacement
                with self.assertRaisesRegex(ContractError, error):
                    self.relations(receipt)

    def test_schedule_rejects_unchanged_or_earlier_end(self):
        """Adiamento exige um novo fim estritamente posterior ao anterior."""
        manifest = self.payload("valid/manifest")
        origin = self.index[manifest["calendarOriginHash"]]
        for end in [origin["endsAt"], "2026-11-21T12:00:00Z"]:
            value = self.payload("valid/delay")
            value["endsAt"] = end
            with self.assertRaises(ContractError):
                self.relations(value)

    def test_schedule_preserves_occurred_start(self):
        """Uma prorrogação durante a votação não altera a abertura já ocorrida."""
        value = self.payload("valid/extension")
        value["startsAt"] = "2026-11-21T13:00:00Z"
        with self.assertRaises(ContractError):
            self.relations(value)

    def test_preparation_delay_can_change_duration(self):
        """A comissão define outro fim, sem obrigação de preservar a duração original."""
        value = self.payload("valid/delay")
        self.assertIsNone(value["manifestHash"])
        self.relations(value)

    def test_preparation_delay_does_not_require_manifest(self):
        """O calendário pode mudar antes de as autoridades prepararem suas chaves."""
        value = self.payload("valid/delay")
        origin_hash = value["previousScheduleHash"]
        self.relations(value, {origin_hash: self.index[origin_hash]})

    def test_closure_rejects_early_end(self):
        """Encerramento utiliza o calendário efetivo depois das prorrogações."""
        value = self.payload("valid/closure")
        value["closedAt"] = "2026-11-24T12:00:00Z"
        with self.assertRaises(ContractError):
            self.relations(value)

    def test_incomplete_has_no_totals(self):
        """Falta de autoridade mantém ausência de totais e não anula automaticamente."""
        value = self.payload("valid/incomplete")
        self.assertIsNone(value["counts"])
        self.assertTrue(value["missingAuthorities"])
        self.assertEqual(value["status"], "incomplete")
        self.relations(value)

    def test_cancelled_has_no_tally(self):
        """Cancelamento em preparação dispensa manifesto e não publica contagens."""
        value = self.payload("cancelled/result")
        self.assertIsNone(value["manifestHash"])
        self.assertIsNone(value["aggregateHash"])
        self.assertIsNone(value["counts"])
        self.relations(value)

    def test_cancelled_result_rejects_unrelated_cancellation(self):
        """O resultado administrativo deve apontar para a mesma eleição cancelada."""
        value = self.payload("cancelled/result")
        value["electionId"] = "99999999-1111-4111-8111-111111111111"
        with self.assertRaises(ContractError):
            self.relations(value)

    def test_complete_tally_vectors(self):
        """Contribuições reais decifram somente os agregados esperados do corpus."""
        prime = int(self.profile["group"]["primeHex"], 16)
        generator = int(self.profile["group"]["generator"])
        for scenario in self.corpus["scenarios"]:
            aggregate = self.index[scenario["aggregateHash"]]
            result = self.index[scenario["resultHash"]]
            shares = [self.index[h] for h in scenario["shareHashes"]]
            decoded = []
            for position, ciphertext in enumerate(aggregate["ciphertexts"]):
                product = 1
                for share in shares:
                    product = product * int(share["shares"][position]["value"]) % prime
                value = int(ciphertext["b"]) * pow(product, -1, prime) % prime
                expected = scenario["expectedTotals"][position]
                self.assertEqual(value, pow(generator, expected, prime))
                total = next(
                    (
                        n
                        for n in range(scenario["voterCount"] + 1)
                        if pow(generator, n, prime) == value
                    ),
                    None,
                )
                decoded.append(total)
            if None in decoded:
                self.assertEqual(result["reason"], "tally_out_of_range")
            else:
                self.assertEqual(decoded, [x["count"] for x in result["counts"]])

    def test_count_mismatch_annuls(self):
        """Excesso e déficit geram anulação sob a mesma regra de igualdade."""
        for name in ["excess", "deficit"]:
            value = self.payload(name + "/result")
            self.assertEqual(value["status"], "annulled")
            self.assertEqual(value["reason"], "aggregate_sum_mismatch")
            self.assertNotEqual(
                sum(x["count"] for x in value["counts"]), value["voterCount"]
            )
            self.relations(value)

    def test_empty_tally_has_zero_totals(self):
        """Sem participantes, a soma zero é consistente e não escolhe vencedor."""
        value = self.payload("empty/result")
        self.assertEqual(value["voterCount"], 0)
        self.assertTrue(all(x["count"] == 0 for x in value["counts"]))
        self.assertEqual(value["status"], "valid")
        self.relations(value)

    def test_profile_rejects_misspelled_signature_setting(self):
        """Uma configuração omitida por erro de digitação não herda um padrão."""
        profile = copy.deepcopy(self.profile)
        profile["signature"]["saltLenghtBytes"] = profile["signature"].pop(
            "saltLengthBytes"
        )
        schema = read_json(CONTRACTS / "profile.schema.json")
        self.assertTrue(list(Draft202012Validator(schema).iter_errors(profile)))

    def test_out_of_range_annuls(self):
        """Um total fora da busca limitada não produz resultado parcial válido."""
        value = self.payload("out-of-range/result")
        self.assertEqual(value["status"], "annulled")
        self.assertIsNone(value["counts"])
        self.relations(value)

    def test_compensated_irregularity_is_an_explicit_limitation(self):
        """A igualdade pode passar apesar de duas cédulas semanticamente inválidas."""
        values = [[1, 1, 0], [0, 0, 0]]
        self.assertTrue(all(sum(x) != 1 for x in values))
        self.assertEqual(sum(map(sum, values)), len(values))
        value = self.payload("compensated/result")
        self.assertEqual(value["status"], "valid")
        self.assertEqual([x["count"] for x in value["counts"]], [1, 1, 0])
        self.relations(value)

    def test_public_messages_exclude_credentials(self):
        """O formato publicado não transporta segredos, tokens ou contatos reais."""
        forbidden = {
            "firebaseUid",
            "firebaseToken",
            "accessToken",
            "email",
            "privateKey",
            "plaintext",
            "r",
            "x",
        }

        def visit(value):
            if isinstance(value, dict):
                self.assertFalse(set(value) & forbidden)
                for item in value.values():
                    visit(item)
            elif isinstance(value, list):
                for item in value:
                    visit(item)

        for case in self.corpus["cases"]:
            visit(case["payload"])

    def test_interfaces_and_matrix_reference_real_contracts_and_tests(self):
        """A rastreabilidade não aponta para variantes ou testes inexistentes."""
        interfaces = read_json(CONTRACTS / "interfaces.json")
        matrix = read_json(CONTRACTS / "security-matrix.json")
        expected = {
            x["$ref"].split("/")[-1] for x in self.schema["$defs"]["Payload"]["oneOf"]
        }
        self.assertEqual(set(interfaces["messageSchemas"]), expected)
        for item in matrix["properties"]:
            for name in item["tests"]:
                self.assertTrue(callable(getattr(self, name, None)), name)


if __name__ == "__main__":
    unittest.main()
