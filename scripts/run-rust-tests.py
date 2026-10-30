"""Run the complete Rust suite against an isolated PostgreSQL test prerequisite."""

from __future__ import annotations

import argparse
import getpass
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from urllib.parse import urlencode

ROOT = Path(__file__).resolve().parent.parent


def run_tests(environment: dict[str, str], rpc_clients: bool) -> None:
    if rpc_clients:
        subprocess.run(
            ["python3", str(ROOT / "scripts/test-rpc-clients.py")],
            cwd=ROOT,
            env=environment,
            check=True,
        )
        return
    subprocess.run(
        ["cargo", "test", "--locked", "--workspace"],
        cwd=ROOT / "rust",
        env=environment,
        check=True,
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rpc-clients", action="store_true")
    options = parser.parse_args()
    environment = dict(os.environ)
    if environment.get("CHAPCHOO_TEST_POSTGRES_URL"):
        # This explicit value must identify a disposable test server, never production.
        run_tests(environment, options.rpc_clients)
        return
    if not shutil.which("initdb") or not shutil.which("pg_ctl"):
        raise SystemExit(
            "PostgreSQL integration tests require initdb/pg_ctl or "
            "CHAPCHOO_TEST_POSTGRES_URL pointing to a disposable test server."
        )
    with tempfile.TemporaryDirectory(prefix="chapchoo-rust-tests-") as temporary:
        path = Path(temporary)
        data = path / "data"
        subprocess.run(
            ["initdb", "-D", str(data), "-A", "trust", "--no-locale", "-E", "UTF8"],
            check=True,
            capture_output=True,
        )
        subprocess.run(
            [
                "pg_ctl",
                "-D",
                str(data),
                "-l",
                str(path / "postgres.log"),
                "-o",
                f"-F -k {path} -h '' -p 5432",
                "-w",
                "start",
            ],
            check=True,
            capture_output=True,
        )
        try:
            environment["CHAPCHOO_TEST_POSTGRES_URL"] = (
                "postgresql:///postgres?"
                + urlencode(
                    {"host": str(path), "port": "5432", "user": getpass.getuser()}
                )
            )
            run_tests(environment, options.rpc_clients)
        finally:
            subprocess.run(
                ["pg_ctl", "-D", str(data), "-m", "immediate", "-w", "stop"],
                check=True,
                capture_output=True,
            )


if __name__ == "__main__":
    main()
