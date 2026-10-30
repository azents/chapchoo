"""Exercise generated Node clients against owned real Rust CLI fixtures on both engines."""

from __future__ import annotations

import os
import secrets
import selectors
import signal
import socket
import subprocess
import tempfile
import time
from pathlib import Path
from urllib.parse import parse_qsl, unquote, urlencode, urlsplit

ROOT = Path(__file__).resolve().parent.parent


def postgres_environment(url: str) -> dict[str, str]:
    """Keep disposable fixture credentials out of command arguments and failures."""
    parts = urlsplit(url)
    if parts.scheme not in {"postgres", "postgresql"}:
        raise RuntimeError("Fixture PostgreSQL URL has an invalid scheme")
    fields = {
        "host": parts.hostname,
        "port": str(parts.port) if parts.port else None,
        "user": unquote(parts.username) if parts.username else None,
        "password": unquote(parts.password) if parts.password else None,
        "dbname": unquote(parts.path.lstrip("/")),
    }
    fields.update(dict(parse_qsl(parts.query, keep_blank_values=True)))
    names = {
        "host": "PGHOST",
        "port": "PGPORT",
        "user": "PGUSER",
        "password": "PGPASSWORD",
        "dbname": "PGDATABASE",
        "sslmode": "PGSSLMODE",
        "connect_timeout": "PGCONNECT_TIMEOUT",
        "options": "PGOPTIONS",
    }
    if set(fields) - set(names):
        raise RuntimeError("Fixture PostgreSQL URL contains unsupported parameters")
    environment = {
        key: value for key, value in os.environ.items() if not key.startswith("PG")
    }
    environment.update(
        {names[key]: value for key, value in fields.items() if value is not None}
    )
    return environment


def fixture_port() -> int:
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def ready(process: subprocess.Popen[str], timeout: float = 30) -> None:
    if process.stdout is None:
        raise RuntimeError("Owned backend stdout is missing")
    deadline = time.monotonic() + timeout
    with selectors.DefaultSelector() as selector:
        selector.register(process.stdout, selectors.EVENT_READ)
        while time.monotonic() < deadline:
            if not selector.select(max(0, deadline - time.monotonic())):
                break
            line = process.stdout.readline()
            if not line:
                raise RuntimeError("Owned backend exited before readiness")
            if "Selected backend roles are ready" in line:
                return
    raise RuntimeError("Owned backend readiness timed out")


def run_fixture(database: dict[str, str]) -> None:
    environment = {
        "PATH": os.environ["PATH"],
        "CHAPCHOO_SECRET_KEY": secrets.token_urlsafe(32) + "=",
        "CHAPCHOO_SETUP_PROOF": secrets.token_urlsafe(32),
        "RUST_LOG": "chapchoo=info",
        "NO_COLOR": "1",
        **database,
    }
    # The Rust key encoding is padded URL-safe base64; bearer encoding is unpadded.
    binary = ROOT / "rust/target/debug/chapchoo"
    subprocess.run(
        [str(binary), "prepare"],
        env=environment,
        check=True,
        capture_output=True,
        timeout=30,
    )
    general, admin = fixture_port(), fixture_port()
    while general == admin:
        admin = fixture_port()
    process = subprocess.Popen(
        [
            str(binary),
            "serve",
            "--general-listen",
            f"127.0.0.1:{general}",
            "--admin-listen",
            f"127.0.0.1:{admin}",
        ],
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    try:
        ready(process)
        subprocess.run(
            ["pnpm", "--filter", "@chapchoo/rpc-client", "test:live"],
            cwd=ROOT / "typescript",
            env={
                **os.environ,
                "CHAPCHOO_TEST_GENERAL_ENDPOINT": f"127.0.0.1:{general}",
                "CHAPCHOO_TEST_ADMIN_ENDPOINT": f"127.0.0.1:{admin}",
                "CHAPCHOO_TEST_SETUP_PROOF": environment["CHAPCHOO_SETUP_PROOF"],
            },
            check=True,
            timeout=120,
        )
    finally:
        if process.poll() is None:
            process.send_signal(signal.SIGTERM)
        try:
            process.communicate(timeout=30)
        except subprocess.TimeoutExpired:
            process.kill()
            process.communicate(timeout=5)
            raise RuntimeError("Owned backend required forced cleanup") from None
        if process.returncode != 0:
            raise RuntimeError("Owned backend did not shut down successfully")


def main() -> None:
    base = os.environ.get("CHAPCHOO_TEST_POSTGRES_URL")
    if not base:
        raise SystemExit("A disposable CHAPCHOO_TEST_POSTGRES_URL is required")
    subprocess.run(
        ["cargo", "build", "--locked", "-p", "chapchoo"],
        cwd=ROOT / "rust",
        check=True,
    )
    with tempfile.TemporaryDirectory(prefix="chapchoo-node-rpc-") as temporary:
        run_fixture({"CHAPCHOO_SQLITE_PATH": str(Path(temporary) / "accounts.sqlite")})
    schema = "node_rpc_" + secrets.token_hex(10)
    subprocess.run(
        ["psql", "-X", "-v", "ON_ERROR_STOP=1", "-c", f"CREATE SCHEMA {schema}"],
        env=postgres_environment(base),
        check=True,
        capture_output=True,
    )
    try:
        parts = urlsplit(base)
        query = parse_qsl(parts.query, keep_blank_values=True)
        if any(key == "options" for key, _ in query):
            raise RuntimeError("Fixture base URL must not contain search-path options")
        url = (
            base.partition("?")[0]
            + "?"
            + urlencode([*query, ("options", f"-csearch_path={schema}")])
        )
        run_fixture({"CHAPCHOO_POSTGRES_URL": url})
    finally:
        subprocess.run(
            [
                "psql",
                "-X",
                "-v",
                "ON_ERROR_STOP=1",
                "-c",
                f"DROP SCHEMA {schema} CASCADE",
            ],
            env=postgres_environment(base),
            check=True,
            capture_output=True,
        )
    print("Node/Rust live RPC checks passed on SQLite and PostgreSQL")


if __name__ == "__main__":
    main()
