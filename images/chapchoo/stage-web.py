#!/usr/bin/env python3
"""Stage one architecture-checked, independently runnable web image tree."""

from __future__ import annotations

import argparse
import importlib.util
import shutil
import sys
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", choices=["general", "admin"], required=True)
    parser.add_argument("--architecture", choices=["amd64", "arm64"], required=True)
    parser.add_argument("--typescript-root", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--node", type=Path, required=True)
    parser.add_argument("--node-license", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location(
        "package_linux",
        Path(__file__).resolve().parents[2] / "scripts/package-linux.py",
    )
    if spec is None or spec.loader is None:
        raise RuntimeError("The artifact assembler is unavailable")
    package = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = package
    spec.loader.exec_module(package)
    architecture = {"amd64": "x86_64", "arm64": "aarch64"}[args.architecture]
    package.elf_architecture(args.binary, architecture)
    package.elf_architecture(args.node, architecture)
    if args.output.resolve().is_relative_to(args.typescript_root.resolve(strict=True)):
        raise package.PackageError(
            "The image staging output must be outside its source tree"
        )
    app = "web" if args.app == "general" else "admin-web"
    source = args.typescript_root / "apps" / app
    args.output.mkdir(parents=True, exist_ok=False)
    shutil.copyfile(args.binary, args.output / "chapchoo")
    (args.output / "chapchoo").chmod(0o755)
    runtime = args.output / "runtime"
    (runtime / "bin").mkdir(parents=True)
    shutil.copyfile(args.node, runtime / "bin/node")
    (runtime / "bin/node").chmod(0o755)
    shutil.copyfile(args.node_license, runtime / "LICENSE")
    package.copy_web(
        package.WebArtifacts(
            source / ".next/standalone", source / ".next/static", source / "public", app
        ),
        args.output / "web" / args.app,
        architecture,
    )


if __name__ == "__main__":
    main()
