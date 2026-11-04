#!/usr/bin/env python3
"""Own one disposable kind cluster for real chart/TokenReview/Computer acceptance."""

from __future__ import annotations

import argparse
import base64
import json
import os
import secrets
import socket
import subprocess
import tempfile
import time
from builtins import BaseExceptionGroup
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def run(arguments: list[str], *, stdin: str | None = None) -> str:
    result = subprocess.run(
        arguments, input=stdin, text=True, capture_output=True, check=False
    )
    if result.returncode:
        # Kubernetes input/output can contain synthetic credentials. Never echo it.
        raise RuntimeError(f"Disposable acceptance command failed: {arguments[0]}")
    return result.stdout


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--image", required=True, help="Existing current-run Docker image"
    )
    options = parser.parse_args()
    cluster = "provider-auth-" + secrets.token_hex(4)
    with tempfile.TemporaryDirectory(prefix="chapchoo-provider-auth-") as folder:
        private = Path(folder)
        os.chmod(private, 0o700)
        kubeconfig = private / "config"
        created = False
        forward: subprocess.Popen[str] | None = None
        primary: BaseException | None = None
        try:
            # Record attempted ownership before creating: partial startup still needs cleanup.
            created = True
            run(
                [
                    "kind",
                    "create",
                    "cluster",
                    "--name",
                    cluster,
                    "--image",
                    "kindest/node:v1.35.0",
                    "--kubeconfig",
                    str(kubeconfig),
                    "--wait",
                    "120s",
                ]
            )
            os.chmod(kubeconfig, 0o600)
            run(["kind", "load", "docker-image", options.image, "--name", cluster])
            kubectl = ["kubectl", "--kubeconfig", str(kubeconfig)]
            namespace = "provider-acceptance"
            password = secrets.token_urlsafe(32)
            resources = [
                {
                    "apiVersion": "v1",
                    "kind": "Namespace",
                    "metadata": {"name": namespace},
                },
                {
                    "apiVersion": "v1",
                    "kind": "Secret",
                    "metadata": {"name": "backend", "namespace": namespace},
                    "stringData": {
                        "secret-key": base64.urlsafe_b64encode(
                            secrets.token_bytes(32)
                        ).decode(),
                        "password": password,
                        "postgres-url": f"postgresql://fixture:{password}@postgres.{namespace}.svc:5432/chapchoo",
                    },
                },
                {
                    "apiVersion": "v1",
                    "kind": "Service",
                    "metadata": {"name": "postgres", "namespace": namespace},
                    "spec": {
                        "selector": {"app": "postgres"},
                        "ports": [{"port": 5432}],
                    },
                },
                {
                    "apiVersion": "apps/v1",
                    "kind": "Deployment",
                    "metadata": {"name": "postgres", "namespace": namespace},
                    "spec": {
                        "selector": {"matchLabels": {"app": "postgres"}},
                        "template": {
                            "metadata": {"labels": {"app": "postgres"}},
                            "spec": {
                                "automountServiceAccountToken": False,
                                "containers": [
                                    {
                                        "name": "postgres",
                                        "image": "postgres:18-alpine",
                                        "env": [
                                            {
                                                "name": "POSTGRES_USER",
                                                "value": "fixture",
                                            },
                                            {
                                                "name": "POSTGRES_DB",
                                                "value": "chapchoo",
                                            },
                                            {
                                                "name": "POSTGRES_PASSWORD",
                                                "valueFrom": {
                                                    "secretKeyRef": {
                                                        "name": "backend",
                                                        "key": "password",
                                                    }
                                                },
                                            },
                                        ],
                                        "readinessProbe": {
                                            "exec": {
                                                "command": [
                                                    "pg_isready",
                                                    "-U",
                                                    "fixture",
                                                    "-d",
                                                    "chapchoo",
                                                ]
                                            },
                                            "periodSeconds": 2,
                                        },
                                    }
                                ],
                            },
                        },
                    },
                },
            ]
            run(
                kubectl + ["apply", "-f", "-"],
                stdin=json.dumps(
                    {"apiVersion": "v1", "kind": "List", "items": resources}
                ),
            )
            run(
                kubectl
                + [
                    "-n",
                    namespace,
                    "rollout",
                    "status",
                    "deployment/postgres",
                    "--timeout=120s",
                ]
            )
            repository, separator, tag = options.image.rpartition(":")
            if not separator or not repository or not tag:
                raise ValueError("Acceptance image requires an explicit tag")
            values = {
                "prerequisitesConfirmed": True,
                "fullnameOverride": "acceptance",
                "database": {"existingSecret": "backend"},
                "secretKey": {"existingSecret": "backend"},
                "backend": {
                    "image": {
                        "repository": repository,
                        "tag": tag,
                        "pullPolicy": "Never",
                    }
                },
                "general": {
                    "enabled": False,
                    "publicUrl": "https://acceptance.example.invalid",
                },
                "admin": {"enabled": False},
                "kubernetesProvider": {"enabled": True},
            }
            path = private / "values.json"
            path.write_text(json.dumps(values))
            run(
                [
                    "helm",
                    "install",
                    "acceptance",
                    str(ROOT / "infra/charts/chapchoo"),
                    "--kubeconfig",
                    str(kubeconfig),
                    "--namespace",
                    namespace,
                    "--values",
                    str(path),
                    "--wait",
                    "--timeout",
                    "180s",
                ]
            )
            for name, args in [
                (
                    "setup",
                    [
                        "get",
                        "secret",
                        "acceptance-setup-proof",
                        "-o",
                        "jsonpath={.data.setup-proof}",
                    ],
                ),
                (
                    "sa",
                    [
                        "create",
                        "token",
                        "acceptance-provider",
                        "--audience=chapchoo-provider-auth",
                        "--duration=10m",
                    ],
                ),
                (
                    "wrong-audience",
                    [
                        "create",
                        "token",
                        "acceptance-provider",
                        "--audience=wrong-audience",
                        "--duration=10m",
                    ],
                ),
            ]:
                content = run(kubectl + ["-n", namespace] + args)
                if name == "setup":
                    content = base64.b64decode(content, validate=True).decode()
                target = private / name
                target.write_text(content)
                target.chmod(0o600)
            with socket.socket() as listener:
                listener.bind(("127.0.0.1", 0))
                port = listener.getsockname()[1]
            forward = subprocess.Popen(
                kubectl
                + [
                    "-n",
                    namespace,
                    "port-forward",
                    "service/acceptance-general-grpc",
                    f"{port}:50051",
                ],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                text=True,
            )
            deadline = time.monotonic() + 15
            while True:
                if forward.poll() is not None:
                    raise RuntimeError("Owned port forward exited")
                try:
                    with socket.create_connection(("127.0.0.1", port), timeout=0.5):
                        break
                except OSError:
                    if time.monotonic() >= deadline:
                        raise RuntimeError(
                            "Owned port forward readiness failed"
                        ) from None
                    time.sleep(0.1)
            output = run(
                [
                    "node",
                    str(
                        ROOT
                        / "typescript/packages/rpc-client/test/provider-acceptance.mjs"
                    ),
                    f"127.0.0.1:{port}",
                    "acceptance-kubernetes",
                    str(private / "setup"),
                    str(private / "sa"),
                    str(private / "wrong-audience"),
                ]
            )
            print(output.strip())
            print("Disposable SA/TokenReview chart acceptance passed")
        except (RuntimeError, ValueError, OSError, KeyboardInterrupt) as error:
            primary = error
        finally:
            if forward is not None:
                forward.terminate()
                try:
                    forward.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    forward.kill()
                    forward.wait(timeout=5)
            if created:
                try:
                    run(["kind", "delete", "cluster", "--name", cluster])
                except (RuntimeError, OSError) as cleanup:
                    if primary is not None:
                        raise BaseExceptionGroup(
                            "Acceptance and owned cleanup failed", [primary, cleanup]
                        ) from None
                    raise
        if primary is not None:
            raise primary


if __name__ == "__main__":
    main()
