#!/usr/bin/env python3
"""Check independently built images with owned private Docker resources only."""

from __future__ import annotations

import argparse
import base64
import json
import os
import re
import secrets
import subprocess
import time
import urllib.error
import urllib.request

WEB_MOUNTS = (
    ("general", "3003", "50051", ""),
    ("admin", "3004", "50052", ""),
    ("general", "3003", "50051", "/app"),
    ("admin", "3004", "50052", "/admin"),
)


def docker(*arguments: str, env: dict[str, str] | None = None) -> str:
    result = subprocess.run(
        ["docker", *arguments],
        env=env,
        capture_output=True,
        text=True,
        timeout=90,
        check=False,
    )
    if result.returncode:
        raise RuntimeError("An owned image acceptance command failed; output withheld")
    return result.stdout.strip()


def request(port: str, path: str, origin: str | None = None) -> bytes:
    url = f"http://127.0.0.1:{port}{path}"
    req = urllib.request.Request(
        url,
        data=b'{"json":null}' if origin else None,
        headers={"Content-Type": "application/json", "Origin": origin}
        if origin
        else {},
    )
    with urllib.request.urlopen(req, timeout=3) as response:
        if response.status != 200:
            raise RuntimeError("An independent image response was unsuccessful")
        return response.read()


def validate(backend: str, general: str, admin: str) -> None:
    # Pin the local image identities once; each deployment must use these bytes.
    backend, general, admin = [
        json.loads(docker("image", "inspect", image))[0]["Id"]
        for image in (backend, general, admin)
    ]
    suffix = secrets.token_hex(6)
    network = f"chapchoo-images-{suffix}"
    containers: list[str] = []
    docker("network", "create", "--internal", network)
    try:
        env = {
            **os.environ,
            "CHAPCHOO_SECRET_KEY": base64.urlsafe_b64encode(os.urandom(32)).decode(),
        }
        backend_id = docker(
            "run",
            "--detach",
            "--read-only",
            "--network",
            network,
            "--network-alias",
            "accounts-backend",
            "--tmpfs",
            "/tmp",
            "--tmpfs",
            "/data:uid=10001,gid=10001,mode=700",
            "--env",
            "CHAPCHOO_SECRET_KEY",
            "--env",
            "CHAPCHOO_SQLITE_PATH=/data/accounts.sqlite",
            "--entrypoint",
            "/bin/sh",
            backend,
            "-c",
            "/opt/chapchoo/chapchoo prepare && exec /opt/chapchoo/chapchoo serve --role both --general-listen 0.0.0.0:50051 --admin-listen 0.0.0.0:50052",
            env=env,
        )
        containers.append(backend_id)
        docker(
            "exec",
            backend_id,
            "/usr/bin/test",
            "!",
            "-e",
            "/opt/chapchoo/runtime/bin/node",
        )
        version = json.loads(
            docker(
                "exec",
                backend_id,
                "/usr/local/bin/kubectl",
                "version",
                "--client=true",
                "--output=json",
            )
        )
        if version["clientVersion"]["gitVersion"] != "v1.35.0":
            raise RuntimeError("Bootstrap Kubernetes client version is not pinned")
        # Only web ports are loopback-published; private RPC never gets a host port.
        # A second normal bridge permits those loopback mappings while the backend
        # remains on the explicitly internal application network.
        images = {"general": general, "admin": admin}
        for app, listen, rpc, prefix in WEB_MOUNTS:
            image = images[app]
            origin = f"https://{app}.example.invalid"
            identifier = docker(
                "run",
                "--detach",
                "--read-only",
                "--tmpfs",
                "/tmp",
                "--publish",
                f"127.0.0.1::{listen}",
                "--env",
                f"CHAPCHOO_{app.upper()}_WEB_URL={origin}{prefix}",
                "--env",
                f"CHAPCHOO_{app.upper()}_BACKEND_URL=http://accounts-backend:{rpc}",
                image,
            )
            containers.append(identifier)
            docker("network", "connect", network, identifier)
            inspected = json.loads(docker("inspect", identifier))[0]
            if (
                inspected["Config"]["User"] != "10001:10001"
                or not inspected["HostConfig"]["ReadonlyRootfs"]
                or inspected["Image"] != image
            ):
                raise RuntimeError(
                    "Independent images must actually run nonroot and read-only"
                )
            published = inspected["NetworkSettings"]["Ports"][f"{listen}/tcp"][0][
                "HostPort"
            ]
            deadline = time.monotonic() + 65
            while True:
                try:
                    if b'"status":"ok"' in request(published, f"{prefix}/health"):
                        break
                except (OSError, urllib.error.URLError):
                    pass
                if time.monotonic() >= deadline:
                    raise RuntimeError(
                        "An independent web image did not become healthy"
                    )
                time.sleep(0.1)
            html = request(published, f"{prefix}/login")
            assets = set(
                re.findall(
                    rb'(?:"|href=|src=)(/[^"<>\s]*_next/static/[^"<>\s]+\.(?:css|js))',
                    html,
                )
            )
            if not assets:
                raise RuntimeError("Independent image HTML has no static assets")
            for asset in assets:
                request(published, asset.decode())
            fonts = set(re.findall(rb'href="([^"]+\.woff2)"', html))
            if len(fonts) != 2:
                raise RuntimeError("Independent image did not serve bundled fonts")
            for font in fonts:
                if not font.decode().startswith(f"{prefix}/fonts/") or not request(
                    published, font.decode()
                ).startswith(b"wOF2"):
                    raise RuntimeError(
                        "Independent image font path or bytes are invalid"
                    )
            if prefix:
                for outside in ("/health", f"{prefix}-outside/health"):
                    try:
                        request(published, outside)
                    except urllib.error.HTTPError as error:
                        if error.code != 404:
                            raise RuntimeError(
                                "Runtime mount returned an unexpected status"
                            ) from None
                    else:
                        raise RuntimeError("Runtime mount accepted an outside path")
            result = json.loads(
                request(published, f"{prefix}/api/trpc/auth.methods", origin)
            )
            if "error" in result or "result" not in result:
                raise RuntimeError(
                    "Independent web image did not reach its private RPC backend"
                )
            opposite = "chapchoo-admin-web" if app == "general" else "chapchoo-web"
            docker(
                "exec",
                identifier,
                "/usr/bin/test",
                "!",
                "-e",
                f"/opt/chapchoo/web/{'admin' if app == 'general' else 'general'}/apps/{opposite}",
            )
        for identifier in reversed(containers):
            docker("stop", "--time", "30", identifier)
            inspected = json.loads(docker("inspect", identifier))[0]
            if inspected["State"]["ExitCode"] != 0:
                raise RuntimeError("An independent image did not stop gracefully")
        print(
            "Same-image root/non-root deployments: nonroot, read-only, static/fonts, private RPC and graceful stop passed"
        )
    finally:
        failures: list[Exception] = []
        for identifier in reversed(containers):
            try:
                docker("rm", "--force", identifier)
            except (RuntimeError, OSError, subprocess.TimeoutExpired) as error:
                failures.append(error)
        try:
            docker("network", "rm", network)
        except (RuntimeError, OSError, subprocess.TimeoutExpired) as error:
            failures.append(error)
        if failures:
            raise RuntimeError("Owned image cleanup failed") from failures[0]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--backend", required=True)
    parser.add_argument("--general", required=True)
    parser.add_argument("--admin", required=True)
    args = parser.parse_args()
    validate(args.backend, args.general, args.admin)


if __name__ == "__main__":
    main()
