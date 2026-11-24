"""Empacota o checkout da demonstração e o relatório revisado pela equipe."""

import argparse
from pathlib import Path
import re
import subprocess
from zipfile import ZIP_DEFLATED, ZipFile


ROOT = Path(__file__).resolve().parents[2]
SOURCE_ROOTS = {"backend", "mobile", "features", "packages", "deploy", ".github"}
ROOT_FILES = {"README.md", ".env.example", ".gitignore"}
EXCLUDED_DIRS = {
    ".git", ".dart_tool", ".gradle", ".idea", ".vscode", "__pycache__",
    "build", "target", "node_modules",
}
EXCLUDED_NAMES = {
    "google-services.json", "firebase_options.dart", "GoogleService-Info.plist",
    "key.properties", "local.properties", "service-account.json",
}
PRIVATE_SUFFIXES = {".pem", ".p12", ".key", ".jks", ".keystore", ".der", ".pfx"}
TEST_KEY = Path("backend/tests/fixtures/jwt/test-only-rsa-private.pem")


def include(path):
    if not path.parts or any(part in EXCLUDED_DIRS for part in path.parts):
        return False
    if path.parts[0] not in SOURCE_ROOTS and path.name not in ROOT_FILES | {
        "compose.yaml", "compose.elections.yaml",
    }:
        return False
    if any(part.startswith(".env") and part != ".env.example" for part in path.parts):
        return False
    if path.name in EXCLUDED_NAMES:
        return False
    return path == TEST_KEY or path.suffix.lower() not in PRIVATE_SUFFIXES


def sources():
    listing = subprocess.check_output(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=ROOT,
    )
    for name in sorted(set(listing.decode().split("\0"))):
        if not name:
            continue
        path = Path(name)
        source = ROOT / path
        if include(path) and source.is_file() and not source.is_symlink():
            if path != TEST_KEY and re.search(rb"^-----BEGIN [A-Z ]*PRIVATE KEY-----", source.read_bytes(), re.M):
                raise ValueError(f"Chave privada inesperada em {path}")
            yield path, source


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    with args.report.open("rb") as report:
        if report.read(5) != b"%PDF-":
            parser.error("--report precisa apontar para um PDF")

    selected = list(sources())
    if not selected:
        parser.error("nenhum arquivo de código encontrado")
    with ZipFile(args.output, "x", ZIP_DEFLATED) as archive:
        for path, source in selected:
            archive.write(source, f"capp/{path.as_posix()}")
        archive.write(args.report, "capp/relatorio.pdf")


if __name__ == "__main__":
    main()
