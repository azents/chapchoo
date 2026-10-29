#!/usr/bin/env python3
"""Exercise an extracted native distribution without source or host tools."""

from __future__ import annotations

import argparse
import base64
import os
import platform
import re
import shutil
import signal
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path


def port() -> int:
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def get(url: str) -> bytes:
    with urllib.request.urlopen(url, timeout=2) as response:
        if response.status != 200:
            raise RuntimeError("Installed resource was not successful")
        return response.read()


def validate(root: Path, architecture: str) -> None:
    if platform.machine() != architecture:
        raise RuntimeError("Acceptance requires the actual native architecture")
    binary = root / "chapchoo"
    node = root / "runtime/bin/node"
    expected = {"x86_64": "x64", "aarch64": "arm64"}[architecture]
    actual = subprocess.check_output([node, "-p", "process.arch"], text=True).strip()
    if actual != expected:
        raise RuntimeError("Bundled Node has the wrong native architecture")
    # Retain private state on any failure, especially uncertain child cleanup.
    # Never remove database files from beneath a potentially surviving process.
    with tempfile.TemporaryDirectory(
        prefix="chapchoo-installed-", delete=False
    ) as temporary:
        cwd = Path(temporary)
        ports: set[int] = set()
        while len(ports) != 4:
            ports.add(port())
        general, admin, general_web, admin_web = ports
        env = {
            "HOME": temporary,
            "PATH": str(cwd / "no-host-tools"),
            "LANG": "C.UTF-8",
            "CHAPCHOO_SQLITE_PATH": str(cwd / "accounts.sqlite"),
            "CHAPCHOO_INSTALLATION_KEY": base64.urlsafe_b64encode(
                os.urandom(32)
            ).decode(),
            "CHAPCHOO_SETUP_PROOF": base64.urlsafe_b64encode(os.urandom(32))
            .decode()
            .rstrip("="),
            "CHAPCHOO_GENERAL_WEB_URL": f"http://127.0.0.1:{general_web}",
            "CHAPCHOO_ADMIN_WEB_URL": f"http://127.0.0.1:{admin_web}",
            "RUST_LOG": "chapchoo=info",
        }
        subprocess.run(
            [binary, "prepare"],
            cwd=cwd,
            env=env,
            check=True,
            capture_output=True,
            timeout=60,
        )
        with tempfile.TemporaryFile() as log:
            process = subprocess.Popen(
                [
                    binary,
                    "serve",
                    "--role",
                    "both",
                    "--web",
                    "both",
                    "--general-listen",
                    f"127.0.0.1:{general}",
                    "--admin-listen",
                    f"127.0.0.1:{admin}",
                    "--general-web-listen",
                    f"127.0.0.1:{general_web}",
                    "--admin-web-listen",
                    f"127.0.0.1:{admin_web}",
                ],
                cwd=cwd,
                env=env,
                stdout=log,
                stderr=log,
            )
            try:
                deadline = time.monotonic() + 65
                while True:
                    if process.poll() is not None:
                        raise RuntimeError(
                            "Installed foreground command exited before readiness"
                        )
                    log.seek(0)
                    if b"Selected components are ready" in log.read():
                        break
                    if time.monotonic() >= deadline:
                        raise RuntimeError(
                            "Installed foreground command did not become ready"
                        )
                    time.sleep(0.1)
                for web_port in [general_web, admin_web]:
                    origin = f"http://127.0.0.1:{web_port}"
                    if b'"status":"ok"' not in get(f"{origin}/health"):
                        raise RuntimeError("Installed web health was invalid")
                    html = get(f"{origin}/login").decode()
                    assets = set(
                        re.findall(r'/_next/static/[^"<>\s]+\.(?:css|js)', html)
                    )
                    if not assets:
                        raise RuntimeError(
                            "Installed HTML did not reference static assets"
                        )
                    font_found = False
                    for asset in assets:
                        content = get(origin + asset)
                        if asset.endswith(".css"):
                            for font in re.findall(
                                rb"(?:\.\./media/|/_next/static/media/)([^)\s]+\.woff2)",
                                content,
                            ):
                                get(origin + "/_next/static/media/" + font.decode())
                                font_found = True
                    if not font_found:
                        raise RuntimeError(
                            "Installed page did not serve its bundled fonts"
                        )
                process.send_signal(signal.SIGINT)
                if process.wait(timeout=30) != 0:
                    raise RuntimeError("Installed SIGINT shutdown failed")
                for owned_port in ports:
                    with socket.socket() as probe:
                        probe.settimeout(1)
                        if probe.connect_ex(("127.0.0.1", owned_port)) == 0:
                            raise RuntimeError(
                                "An installed listener survived owned shutdown"
                            )
            finally:
                if process.poll() is None:
                    process.send_signal(signal.SIGTERM)
                    try:
                        process.wait(timeout=30)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)
                        raise RuntimeError(
                            "Installed command required forced test cleanup"
                        ) from None
    shutil.rmtree(temporary)
    print(
        f"Native {architecture}: SQLite preparation, bundled web/static/fonts and SIGINT passed"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--architecture", choices=["x86_64", "aarch64"], required=True)
    args = parser.parse_args()
    validate(args.root.resolve(strict=True), args.architecture)


if __name__ == "__main__":
    main()
