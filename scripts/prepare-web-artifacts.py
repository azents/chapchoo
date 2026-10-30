#!/usr/bin/env python3
"""Export validated Next build outputs for Docker without copying app sources."""

from __future__ import annotations

import argparse
import importlib.util
import shutil
import sys
from pathlib import Path

spec = importlib.util.spec_from_file_location(
    "web_artifact_validation", Path(__file__).with_name("package-linux.py")
)
if spec is None or spec.loader is None:
    raise RuntimeError("Web artifact validation is unavailable")
package = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = package
spec.loader.exec_module(package)


def prepare(typescript_root: Path, output: Path, architecture: str) -> None:
    root = typescript_root.resolve(strict=True)
    destination = output.resolve()
    if destination.is_relative_to(root):
        raise package.PackageError("Artifact output must be outside TypeScript sources")
    if destination.exists():
        raise package.PackageError("Artifact output already exists")
    sources = []
    for component, app in [
        ("general", "web"),
        ("admin", "admin-web"),
    ]:
        app_root = root / "apps" / app
        artifacts = package.WebArtifacts(
            app_root / ".next/standalone",
            app_root / ".next/static",
            app_root / "public",
            app,
        )
        for source in [artifacts.standalone, artifacts.static, artifacts.public]:
            package.validate_tree(source, architecture)
        if not (
            artifacts.standalone / "typescript/apps" / app / "server.mjs"
        ).is_file():
            raise package.PackageError("A built standalone entrypoint is missing")
        sources.append((component, artifacts))
    destination.mkdir(parents=True)
    for component, artifacts in sources:
        for name, source in [
            ("standalone", artifacts.standalone),
            ("static", artifacts.static),
            ("public", artifacts.public),
        ]:
            shutil.copytree(source, destination / component / name, symlinks=True)
    (destination / "architecture").write_text(
        {"x86_64": "amd64", "aarch64": "arm64"}[architecture] + "\n"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--typescript-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--architecture", choices=["x86_64", "aarch64"], required=True)
    args = parser.parse_args()
    prepare(args.typescript_root, args.output, args.architecture)


if __name__ == "__main__":
    main()
