"""Generate the public server certificate and its private key for the HTTPS demo."""

import argparse
import ipaddress
import re
import shutil
import subprocess
import tempfile
from pathlib import Path


def dns_name(value):
    labels = value.removesuffix(".").split(".")
    if len(value) > 253 or not all(
        re.fullmatch(r"[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?", label)
        for label in labels
    ):
        raise argparse.ArgumentTypeError(f"nome DNS inválido: {value}")
    return ".".join(labels).lower()


def ip_address(value):
    try:
        address = ipaddress.ip_address(value)
        if isinstance(address, ipaddress.IPv6Address) and address.scope_id:
            raise ValueError
        return str(address)
    except ValueError as error:
        raise argparse.ArgumentTypeError(f"endereço IP inválido: {value}") from error


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dns", action="append", type=dns_name, default=[])
    parser.add_argument("--ip", action="append", type=ip_address, default=[])
    parser.add_argument(
        "--out-dir", type=Path, default=Path(__file__).resolve().parents[2] / ".tls"
    )
    args = parser.parse_args()
    openssl = shutil.which("openssl")
    if openssl is None:
        parser.error("openssl não está no PATH")

    sans = [f"DNS:{name}" for name in ["localhost", *args.dns]]
    sans += [f"IP:{ip}" for ip in ["127.0.0.1", "::1", "10.0.2.2", *args.ip]]
    out = args.out_dir.resolve()
    out.mkdir(parents=True, exist_ok=True)
    cert, key = out / "backend.crt", out / "backend.key"
    if cert.exists() or key.exists():
        parser.error(f"já existe material TLS em {out}; escolha outro diretório")

    with tempfile.TemporaryDirectory(dir=out) as directory:
        new_cert = Path(directory) / cert.name
        new_key = Path(directory) / key.name
        subprocess.run(
            [
                openssl, "req", "-new", "-x509", "-newkey", "ec",
                "-pkeyopt", "ec_paramgen_curve:prime256v1", "-noenc",
                "-days", "3652", "-sha256", "-subj", "/CN=CApp demo backend",
                "-addext", f"subjectAltName={','.join(dict.fromkeys(sans))}",
                "-addext", "basicConstraints=critical,CA:FALSE",
                "-addext", "keyUsage=critical,digitalSignature",
                "-addext", "extendedKeyUsage=serverAuth",
                "-keyout", str(new_key), "-out", str(new_cert),
            ],
            check=True,
        )
        new_key.chmod(0o600)
        # Links exclusivos evitam sobrescrever arquivos de outra execução.
        key.hardlink_to(new_key)
        try:
            cert.hardlink_to(new_cert)
        except OSError:
            key.unlink()
            raise

    print(f"Certificado público: {cert}")
    print(f"Chave privada: {key}")


if __name__ == "__main__":
    main()
