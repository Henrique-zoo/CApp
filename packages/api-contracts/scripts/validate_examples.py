#!/usr/bin/env python3
"""Valida exemplos externalValue contra os schemas do OpenAPI 3.1."""

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
    return "/" + "/".join(escape_pointer_token(part) for part in parts)


def load_spec() -> dict:
    with SPEC_PATH.open(encoding="utf-8") as file:
        spec = yaml.safe_load(file)

    if not isinstance(spec, dict):
        raise ValueError("A especificação OpenAPI não é um objeto YAML.")

    if not str(spec.get("openapi", "")).startswith("3.1."):
        raise ValueError("O validador exige uma especificação OpenAPI 3.1.")

    return spec


def iter_operation_media(spec: dict):
    """Produz os conteúdos de requestBody e responses de cada operação."""
    for path, path_item in spec.get("paths", {}).items():
        if not isinstance(path_item, dict):
            continue

        for method, operation in path_item.items():
            if method.lower() not in HTTP_METHODS or not isinstance(operation, dict):
                continue

            operation_parts = ["paths", path, method]

            request_body = operation.get("requestBody", {})
            if isinstance(request_body, dict):
                content = request_body.get("content", {})
                if isinstance(content, dict):
                    for media_type in content:
                        yield (
                            path,
                            method.upper(),
                            "requestBody",
                            media_type,
                            content[media_type],
                            operation_parts
                            + ["requestBody", "content", media_type],
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

                for media_type in content:
                    yield (
                        path,
                        method.upper(),
                        f"response {status}",
                        media_type,
                        content[media_type],
                        operation_parts
                        + ["responses", status, "content", media_type],
                    )


def validate_examples(spec: dict) -> int:
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

        examples = media.get("examples", {})
        if not isinstance(examples, dict):
            continue

        for example_name, example in examples.items():
            if not isinstance(example, dict) or "externalValue" not in example:
                continue

            external_value = example["externalValue"]

            if not isinstance(external_value, str) or not external_value:
                failures.append(
                    f"{method} {path} ({direction}): "
                    f"{example_name}: externalValue inválido."
                )
                continue

            parsed = urlparse(external_value)
            if parsed.scheme or parsed.netloc or Path(external_value).is_absolute():
                failures.append(
                    f"{method} {path} ({direction}): "
                    f"{example_name}: somente referências locais são permitidas."
                )
                continue

            example_path = (SPEC_PATH.parent / external_value).resolve()

            try:
                example_path.relative_to(root)
            except ValueError:
                failures.append(
                    f"{method} {path} ({direction}): "
                    f"{example_name}: arquivo fora de packages/api-contracts."
                )
                continue

            if not example_path.is_file():
                failures.append(
                    f"{method} {path} ({direction}): "
                    f"{example_name}: arquivo não encontrado: {external_value}"
                )
                continue

            schema_parts = media_parts + ["schema"]
            schema_pointer = pointer(schema_parts)

            schema_ref = f"{spec_uri}#{schema_pointer}"

            try:
                with example_path.open(encoding="utf-8") as file:
                    instance = json.load(file)

                validator = Draft202012Validator(
                    {"$ref": schema_ref},
                    registry=registry,
                    format_checker=FormatChecker(),
                )

                errors = sorted(
                    validator.iter_errors(instance),
                    key=lambda error: list(map(str, error.absolute_path)),
                )

                if errors:
                    for error in errors:
                        location = "/".join(map(str, error.absolute_path))
                        location = location or "<raiz>"

                        failures.append(
                            f"{method} {path} ({direction}, {media_type}): "
                            f"{example_name} ({external_value}) em {location}: "
                            f"{error.message}"
                        )
                    continue

                print(
                    f"OK: {method} {path} ({direction}) "
                    f"{example_name} -> {external_value}"
                )
                validated += 1

            except json.JSONDecodeError as error:
                failures.append(
                    f"{external_value}: JSON inválido na linha "
                    f"{error.lineno}, coluna {error.colno}: {error.msg}"
                )
            except Exception as error:
                failures.append(
                    f"{method} {path} ({direction}): "
                    f"{example_name}: falha na validação: {error}"
                )

    if failures:
        print("\nFalhas na validação dos exemplos:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    if validated == 0:
        print(
            "Nenhum exemplo externalValue foi validado.",
            file=sys.stderr,
        )
        return 1

    print(f"\n{validated} exemplos validados contra os schemas OpenAPI.")
    return 0


def main() -> int:
    try:
        spec = load_spec()
        return validate_examples(spec)
    except Exception as error:
        print(f"Erro ao validar os exemplos: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
