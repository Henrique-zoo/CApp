"""Run the real Flutter transport against short-lived local TLS fixtures."""

import argparse
import base64
import ipaddress
import json
import shutil
import ssl
import subprocess
import tempfile
import threading
from contextlib import ExitStack
from datetime import datetime, timedelta, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.x509.oid import ExtendedKeyUsageOID, NameOID


def certificate(label, now, issuer=None, address=None, expired=False):
    key = ec.generate_private_key(ec.SECP256R1())
    name = x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, label)])
    signer, issuer_name = (key, name) if issuer is None else (issuer[0], issuer[1].subject)
    builder = (
        x509.CertificateBuilder()
        .subject_name(name)
        .issuer_name(issuer_name)
        .public_key(key.public_key())
        .serial_number(x509.random_serial_number())
        .not_valid_before(now - timedelta(days=2))
        .not_valid_after(now + timedelta(days=-1 if expired else 1))
        .add_extension(x509.BasicConstraints(ca=issuer is None, path_length=None), critical=True)
    )
    if issuer is not None:
        builder = builder.add_extension(x509.SubjectAlternativeName([address]), critical=False)
        builder = builder.add_extension(
            x509.ExtendedKeyUsage([ExtendedKeyUsageOID.SERVER_AUTH]), critical=False
        )
    return key, builder.sign(signer, hashes.SHA256())


class FixtureHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.server.paths.append(self.path)
        if self.path == "/auth/session" and self.server.redirect:
            # A protected GET exercises both production Firebase interceptors.
            authorized = (
                self.headers.get("Authorization") == "Bearer test-id"
                and self.headers.get("X-Firebase-AppCheck") == "test-check"
            )
            self.send_response(307 if authorized else 403)
            if authorized:
                self.send_header("Location", self.server.redirect)
        else:
            self.send_response(200 if self.path == "/ready" else 404)
        self.send_header("Content-Length", "0")
        self.end_headers()

    def log_message(self, fmt, *args):
        pass


def serve(stack, tmp, label, cert=None, redirect=None):
    server = ThreadingHTTPServer(("127.0.0.1", 0), FixtureHandler)
    stack.callback(server.server_close)
    server.paths = []
    server.redirect = redirect
    if cert is not None:
        key, leaf = cert
        key_path, cert_path = tmp / (label + ".key"), tmp / (label + ".pem")
        key_path.write_bytes(key.private_bytes(
            serialization.Encoding.PEM,
            serialization.PrivateFormat.PKCS8,
            serialization.NoEncryption(),
        ))
        cert_path.write_bytes(leaf.public_bytes(serialization.Encoding.PEM))
        tls = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        tls.load_cert_chain(cert_path, key_path)
        server.socket = tls.wrap_socket(server.socket, server_side=True)
    thread = threading.Thread(target=server.serve_forever, args=(0.1,), daemon=True)
    thread.start()
    stack.callback(thread.join)
    stack.callback(server.shutdown)
    return server


def main():
    args = argparse.ArgumentParser(description=__doc__)
    args.add_argument("--device", help="Android emulator ID as shown by flutter devices")
    args.add_argument("--caddy-url", help="optional HTTPS base URL for the running demo")
    args.add_argument("--caddy-ca", type=Path, help="public PEM of the demo root CA")
    opts = args.parse_args()
    if bool(opts.caddy_url) != bool(opts.caddy_ca):
        args.error("supply --caddy-url and --caddy-ca together")
    flutter = shutil.which("flutter")
    if not flutter:
        args.error("flutter is missing from PATH")
    host = "10.0.2.2" if opts.device else "127.0.0.1"
    address = x509.IPAddress(ipaddress.ip_address(host))
    now = datetime.now(timezone.utc)
    trusted = certificate("CApp test CA", now)
    foreign = certificate("CApp unrelated test CA", now)
    with tempfile.TemporaryDirectory() as directory, ExitStack() as stack:
        tmp = Path(directory)
        clear = serve(stack, tmp, "clear")
        clear_url = f"http://{host}:{clear.server_port}"
        config = {
            "TLS_CLEAR": clear_url,
            "CAPP_DEMO_CA_PEM_BASE64": base64.b64encode(
                trusted[1].public_bytes(serialization.Encoding.PEM)
            ).decode(),
        }
        servers = {}
        for label, issuer, san, expired in [
            ("VALID", trusted, address, False),
            ("UNKNOWN", foreign, address, False),
            ("EXPIRED", trusted, address, True),
            ("MISMATCH", trusted, x509.DNSName("other.invalid"), False),
        ]:
            cert = certificate(label, now, issuer, san, expired)
            server = serve(stack, tmp, label, cert, clear_url + "/auth/session")
            servers[label] = server
            config["TLS_" + label] = f"https://{host}:{server.server_port}"
        if opts.caddy_url:
            config["TLS_CADDY"] = opts.caddy_url
            config["TLS_CADDY_CA"] = base64.b64encode(opts.caddy_ca.read_bytes()).decode()
        defines = tmp / "settings.json"
        defines.write_text(json.dumps(config), encoding="utf-8")
        command = [
            flutter, "test", "--no-pub", "--reporter", "expanded",
            f"--dart-define-from-file={defines}",
        ]
        if opts.device:
            command.extend(["-d", opts.device, "integration_test/backend_tls_test.dart"])
        else:
            command.append("tool/backend_tls_test.dart")
        subprocess.run(command, cwd=Path(__file__).resolve().parents[1], check=True)
        if clear.paths or any(server.paths for label, server in servers.items() if label != "VALID"):
            raise RuntimeError("an HTTP request reached an untrusted or cleartext server")
        if servers["VALID"].paths != ["/ready", "/auth/session"]:
            raise RuntimeError("expected /ready and the authenticated redirect over trusted TLS")
        print("No HTTP request reached a rejected TLS endpoint or the cleartext listener.")


if __name__ == "__main__":
    main()
