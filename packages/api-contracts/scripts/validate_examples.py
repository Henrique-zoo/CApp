#!/usr/bin/env python3
"""Valida exemplos OpenAPI externos e inline contra seus schemas."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from urllib.parse import urlparse

import yaml
from jsonschema import Draft202012Validator, FormatChecker
from referencing import Registry, Resource
from referencing.jsonschema import DRAFT202012


SPEC_PATH = Path(__file__).resolve().parents[1] / "openapi.yaml"

HTTP_METHODS = {
    "get",
    "put",
    "post",
    "delete",
    "options",
    "head",
    "patch",
    "trace",
}


def escape_pointer_token(token: str) -> str:
    """Escapa um segmento conforme JSON Pointer."""
    return token.replace("~", "~0").replace("/", "~1")


def pointer(parts: list[str]) -> str:
    """Monta um JSON Pointer absoluto."""
    return "/" + "/".join(
        escape_pointer_token(part) for part in parts
    )


def load_spec() -> dict:
    """Carrega e verifica a versão da especificação OpenAPI."""
    with SPEC_PATH.open(encoding="utf-8") as file:
        spec = yaml.safe_load(file)

    if not isinstance(spec, dict):
        raise ValueError(
            "A especificação OpenAPI não é um objeto YAML."
        )

    if not str(spec.get("openapi", "")).startswith("3.1."):
        raise ValueError(
            "O validador exige uma especificação OpenAPI 3.1."
        )

    return spec


def iter_operation_media(spec: dict):
    """Produz os conteúdos de requestBody e responses."""
    for path, path_item in spec.get("paths", {}).items():
        if not isinstance(path_item, dict):
            continue

        for method, operation in path_item.items():
            if (
                method.lower() not in HTTP_METHODS
                or not isinstance(operation, dict)
            ):
                continue

            operation_parts = ["paths", path, method]

            request_body = operation.get("requestBody", {})

            if isinstance(request_body, dict):
                content = request_body.get("content", {})

                if isinstance(content, dict):
                    for media_type, media in content.items():
                        yield (
                            path,
                            method.upper(),
                            "requestBody",
                            media_type,
                            media,
                            operation_parts
                            + [
                                "requestBody",
                                "content",
                                media_type,
                            ],
                        )

            responses = operation.get("responses", {})

            if not isinstance(responses, dict):
                continue

            for status, response in responses.items():
                if not isinstance(response, dict):
                    continue

                content = response.get("content", {})

                if not isinstance(content, dict):
                    continue

                for media_type, media in content.items():
                    yield (
                        path,
                        method.upper(),
                        f"response {status}",
                        media_type,
                        media,
                        operation_parts
                        + [
                            "responses",
                            status,
                            "content",
                            media_type,
                        ],
                    )


def validate_instance(
    instance: object,
    schema: dict,
    registry: Registry | None = None,
) -> list:
    """Retorna os erros de validação de uma instância JSON."""
    validator = Draft202012Validator(
        schema,
        registry=registry or Registry(),
        format_checker=FormatChecker(),
    )

    return sorted(
        validator.iter_errors(instance),
        key=lambda error: list(
            map(str, error.absolute_path)
        ),
    )


def load_external_example(
    external_value: object,
    example_root: Path,
    root: Path,
) -> object:
    """Carrega um exemplo JSON externo dentro do pacote de contratos."""
    if not isinstance(external_value, str) or not external_value:
        raise ValueError("externalValue inválido.")

    parsed = urlparse(external_value)

    if (
        parsed.scheme
        or parsed.netloc
        or parsed.query
        or parsed.fragment
        or Path(external_value).is_absolute()
    ):
        raise ValueError(
            "Somente referências locais a arquivos são permitidas."
        )

    example_path = (example_root / external_value).resolve()

    try:
        example_path.relative_to(root)
    except ValueError as error:
        raise ValueError(
            "O arquivo externo está fora de packages/api-contracts."
        ) from error

    if not example_path.is_file():
        raise FileNotFoundError(
            f"Arquivo não encontrado: {external_value}"
        )

    with example_path.open(encoding="utf-8") as file:
        return json.load(file)


def validate_one_example(
    *,
    instance: object,
    label: str,
    schema_ref: str,
    registry: Registry,
) -> list[str]:
    """Valida um exemplo contra o schema da operação."""
    errors = validate_instance(
        instance,
        {"$ref": schema_ref},
        registry,
    )

    if not errors:
        print(f"OK: {label}")
        return []

    failures = []

    for error in errors:
        location = "/".join(
            map(str, error.absolute_path)
        ) or "<raiz>"

        failures.append(
            f"{label} em {location}: {error.message}"
        )

    return failures


def validate_examples(spec: dict) -> int:
    """Valida exemplos externos, examples.value e example."""
    spec_uri = SPEC_PATH.resolve().as_uri()

    registry = Registry().with_resource(
        spec_uri,
        Resource.from_contents(
            spec,
            default_specification=DRAFT202012,
        ),
    )

    root = SPEC_PATH.parent.resolve()

    validated = 0
    failures: list[str] = []

    for (
        path,
        method,
        direction,
        media_type,
        media,
        media_parts,
    ) in iter_operation_media(spec):

        if not isinstance(media, dict):
            continue

        schema = media.get("schema")

        examples = media.get("examples", {})

        has_single_example = "example" in media
        has_named_examples = bool(examples)

        if not has_single_example and not has_named_examples:
            continue

        if not isinstance(schema, dict):
            failures.append(
                f"{method} {path} ({direction}, {media_type}): "
                "há exemplos, mas não existe um schema válido."
            )
            continue

        schema_pointer = pointer(
            media_parts + ["schema"]
        )

        schema_ref = f"{spec_uri}#{schema_pointer}"

        context = (
            f"{method} {path} ({direction}, {media_type})"
        )

        # Formato: media.example
        if has_single_example:
            label = f"{context}: example inline"

            failures.extend(
                validate_one_example(
                    instance=media["example"],
                    label=label,
                    schema_ref=schema_ref,
                    registry=registry,
                )
            )

            if not validate_instance(
                media["example"],
                {"$ref": schema_ref},
                registry,
            ):
                validated += 1

        # Formato: media.examples[nome].value/externalValue
        if has_named_examples:
            if not isinstance(examples, dict):
                failures.append(
                    f"{context}: examples precisa ser um objeto."
                )
                continue

            for example_name, example in examples.items():
                label = f"{context}: {example_name}"

                if not isinstance(example, dict):
                    failures.append(
                        f"{label}: definição de exemplo inválida."
                    )
                    continue

                if "$ref" in example:
                    failures.append(
                        f"{label}: referências $ref para objetos "
                        "Example ainda não são suportadas pelo validador."
                    )
                    continue

                has_value = "value" in example
                has_external_value = "externalValue" in example

                if has_value and has_external_value:
                    failures.append(
                        f"{label}: value e externalValue não "
                        "podem ser declarados simultaneamente."
                    )
                    continue

                if not has_value and not has_external_value:
                    failures.append(
                        f"{label}: o exemplo precisa declarar "
                        "value ou externalValue."
                    )
                    continue

                try:
                    if has_external_value:
                        instance = load_external_example(
                            example["externalValue"],
                            SPEC_PATH.parent,
                            root,
                        )
                        label += (
                            f" -> {example['externalValue']}"
                        )
                    else:
                        instance = example["value"]
                        label += " (inline)"

                    errors = validate_one_example(
                        instance=instance,
                        label=label,
                        schema_ref=schema_ref,
                        registry=registry,
                    )

                    failures.extend(errors)

                    if not errors:
                        validated += 1

                except json.JSONDecodeError as error:
                    failures.append(
                        f"{label}: JSON inválido na linha "
                        f"{error.lineno}, coluna {error.colno}: "
                        f"{error.msg}"
                    )

                except Exception as error:
                    failures.append(
                        f"{label}: falha ao carregar ou validar: "
                        f"{error}"
                    )

    if failures:
        print(
            "\nFalhas na validação dos exemplos:",
            file=sys.stderr,
        )

        for failure in failures:
            print(f"- {failure}", file=sys.stderr)

        return 1

    if validated == 0:
        print(
            "Nenhum exemplo foi validado.",
            file=sys.stderr,
        )
        return 1

    print(
        f"\n{validated} exemplos validados "
        "contra os schemas OpenAPI."
    )

    return 0


def main() -> int:
    try:
        spec = load_spec()
        return validate_examples(spec)

    except Exception as error:
        print(
            f"Erro ao validar os exemplos: {error}",
            file=sys.stderr,
        )
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
