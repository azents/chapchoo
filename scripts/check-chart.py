#!/usr/bin/env python3
"""Run offline Helm lint and chart contract tests with the existing test parser."""

import shutil
import subprocess
import sys
from pathlib import Path


def main() -> int:
    """Validate fixture rendering without accessing any Kubernetes cluster."""
    root = Path(__file__).resolve().parents[1]
    helm = shutil.which("helm")
    if helm is None:
        raise RuntimeError("Helm is required; missing tooling is not a skipped pass")
    subprocess.run(
        [
            helm,
            "lint",
            "--strict",
            str(root / "infra/charts/chapchoo"),
            "-f",
            str(root / "testenv/chapchoo/chart-values.yaml"),
        ],
        cwd=root,
        check=True,
        timeout=60,
    )
    return subprocess.run(
        [sys.executable, "-m", "unittest", "scripts.tests.test_chart_contracts", "-v"],
        cwd=root,
        check=False,
        timeout=180,
    ).returncode


if __name__ == "__main__":
    raise SystemExit(main())
