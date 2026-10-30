"""Validate target artifacts and Mac/Linux orchestration without real credentials."""

from __future__ import annotations

import importlib.util
import json
import os
import shutil
import struct
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "prepare_web_artifacts", ROOT / "scripts/prepare-web-artifacts.py"
)
assert SPEC is not None and SPEC.loader is not None
prepare = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(prepare)


class BuildArtifactTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.source = self.root / "typescript"
        for app in ["web", "admin-web"]:
            base = self.source / "apps" / app
            entry = base / ".next/standalone/typescript/apps" / app
            entry.mkdir(parents=True)
            (entry / "server.mjs").write_text("// static fixture\n")
            dependencies = base / ".next/standalone/node_modules"
            dependencies.mkdir()
            header = bytearray(64)
            header[:7] = b"\x7fELF\x02\x01\x01"
            struct.pack_into("<HH", header, 16, 3, 62)
            (dependencies / "fixture.node").write_bytes(header)
            static = base / ".next/static"
            static.mkdir()
            (static / "app.js").write_text("// built JS\n")
            (base / "public").mkdir()
            (base / "public/font.woff2").write_bytes(b"font fixture")

    def test_export_preserves_only_built_inputs(self) -> None:
        (self.source / "source-only.ts").write_text("not exported")
        destination = self.root / "output"
        prepare.prepare(self.source, destination, "x86_64")
        self.assertEqual((destination / "architecture").read_text(), "amd64\n")
        self.assertFalse((destination / "source-only.ts").exists())
        self.assertTrue(
            (
                destination / "general/standalone/typescript/apps/web/server.mjs"
            ).is_file()
        )
        self.assertEqual(
            (destination / "admin/public/font.woff2").read_bytes(), b"font fixture"
        )

    def test_mismatched_native_module_cannot_be_exported(self) -> None:
        destination = self.root / "output"
        with self.assertRaises(prepare.package.PackageError):
            prepare.prepare(self.source, destination, "aarch64")
        self.assertFalse(destination.exists())

    def test_mac_native_module_cannot_be_exported_as_linux(self) -> None:
        addon = self.source / "apps/web/.next/standalone/node_modules/fixture.node"
        addon.write_bytes(b"\xcf\xfa\xed\xfe" + bytes(60))
        with self.assertRaises(prepare.package.PackageError):
            prepare.prepare(self.source, self.root / "output", "x86_64")

    def test_foreign_symlink_and_secret_are_rejected_before_output(self) -> None:
        public = self.source / "apps/web/public"
        (public / "escape").symlink_to(self.root)
        with self.assertRaises(prepare.package.PackageError):
            prepare.prepare(self.source, self.root / "output", "x86_64")
        (public / "escape").unlink()
        (public / ".env").write_text("synthetic fixture")
        with self.assertRaises(prepare.package.PackageError):
            prepare.prepare(self.source, self.root / "output", "x86_64")
        self.assertFalse((self.root / "output").exists())

    def test_export_refuses_overwrite_or_source_tree_output(self) -> None:
        for destination in [self.root, self.source / "output"]:
            with self.assertRaises(prepare.package.PackageError):
                prepare.prepare(self.source, destination, "x86_64")

    def test_mac_wrapper_uses_linux_targets_and_independent_contexts(self) -> None:
        bin_path = self.root / "bin"
        bin_path.mkdir()
        uname = bin_path / "uname"
        uname.write_text("#!/bin/sh\nprintf 'arm64\\n'\n")
        uname.chmod(0o755)
        docker = bin_path / "docker"
        python = shutil.which("python3")
        assert python is not None
        docker.write_text(
            f"#!{python}\n"
            "import os,sys,json,tarfile\n"
            "for arg in sys.argv[1:]:\n"
            " if arg.startswith('type=tar,dest='):\n"
            "  tarfile.open(arg.split('dest=',1)[1], 'w').close()\n"
            "with open(os.environ['CALL_LOG'],'a') as f:\n"
            " f.write(json.dumps({'args':sys.argv[1:],"
            "'web':os.environ.get('WEB_ARTIFACTS'),"
            "'binary':os.environ.get('BINARY_ARTIFACTS')})+'\\n')\n"
        )
        docker.chmod(0o755)
        log = self.root / "calls.jsonl"
        subprocess.run(
            ["bash", str(ROOT / "scripts/build-images.sh")],
            env={
                **os.environ,
                "PATH": f"{bin_path}:{os.environ['PATH']}",
                "CALL_LOG": str(log),
            },
            check=True,
        )
        calls = [json.loads(line) for line in log.read_text().splitlines()]
        self.assertEqual(len(calls), 3)
        self.assertIn("linux/arm64", calls[0]["args"])
        self.assertIn("linux/arm64", calls[1]["args"])
        self.assertIn("web-build.Dockerfile", " ".join(calls[0]["args"]))
        self.assertIn("binary-build.Dockerfile", " ".join(calls[1]["args"]))
        self.assertEqual(calls[2]["args"][:2], ["buildx", "bake"])
        self.assertNotEqual(calls[2]["web"], calls[2]["binary"])


if __name__ == "__main__":
    unittest.main()
