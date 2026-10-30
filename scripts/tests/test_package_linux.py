"""Synthetic artifact safety tests, not native release acceptance evidence."""

from __future__ import annotations

import importlib.util
import io
import os
import shutil
import struct
import subprocess
import sys
import tarfile
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "package_linux", ROOT / "scripts/package-linux.py"
)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("Package assembler module is unavailable")
package = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = package
SPEC.loader.exec_module(package)


def elf(architecture: str) -> bytes:
    header = bytearray(64)
    header[:7] = b"\x7fELF\x02\x01\x01"
    struct.pack_into("<HH", header, 16, 3, {"x86_64": 62, "aarch64": 183}[architecture])
    return bytes(header)


def member(name: str, data: bytes) -> tuple[tarfile.TarInfo, io.BytesIO]:
    info = tarfile.TarInfo(name)
    info.size = len(data)
    info.mode = 0o755 if name.endswith("/node") else 0o644
    return info, io.BytesIO(data)


class PackageLinuxTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="chapchoo-package-unit-")
        self.root = Path(self.temporary.name)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def fixtures(self, architecture: str = "x86_64"):
        binary = self.root / "inputs" / "chapchoo"
        binary.parent.mkdir(parents=True, exist_ok=True)
        binary.write_bytes(elf(architecture))
        binary.chmod(0o755)
        node_name = (
            f"node-v{package.NODE_VERSION}-linux-{package.NODE_ARCH[architecture]}"
        )
        archive = self.root / f"{node_name}.tar.xz"
        with tarfile.open(archive, "w:xz") as output:
            for name, data in [
                (f"{node_name}/bin/node", elf(architecture)),
                (f"{node_name}/LICENSE", b"Synthetic upstream license"),
            ]:
                info, content = member(name, data)
                output.addfile(info, content)
        digest = package.sha256(archive)
        shasums = self.root / "SHASUMS256.txt"
        shasums.write_text(f"{digest}  {archive.name}\n", encoding="ascii")
        web = []
        for name, app in [("general", "chapchoo-web"), ("admin", "chapchoo-admin-web")]:
            source = self.root / "inputs" / name
            standalone = source / "standalone"
            app_root = standalone / "apps" / app
            (app_root / "node_modules").mkdir(parents=True)
            (app_root / "server.js").write_text(
                "// Synthetic server; never executed\n", encoding="utf-8"
            )
            dependency = standalone / "node_modules" / "next"
            dependency.mkdir(parents=True)
            (dependency / "package.json").write_text("{}\n", encoding="utf-8")
            (dependency / "addon.node").write_bytes(elf(architecture))
            (app_root / "node_modules/next").symlink_to(
                "../../../node_modules/next", target_is_directory=True
            )
            static = source / "static"
            static.mkdir()
            (static / "font.woff2").write_bytes(b"wOF2:preserved-real-asset-bytes")
            public = source / "public"
            public.mkdir()
            (public / "icon.svg").write_text(
                '<svg xmlns="http://www.w3.org/2000/svg"/>', encoding="utf-8"
            )
            web.append(package.WebArtifacts(standalone, static, public, app))
        return package.PackageInputs(
            architecture,
            binary,
            archive,
            shasums,
            web[0],
            web[1],
            self.root / "output/package.tar.gz",
            0,
        ), digest

    def build(self, inputs, digest: str) -> Path:
        # Production has no checksum bypass. Only these synthetic tests replace
        # the pinned inventory in memory, and no fixture executable is run.
        with patch.dict(package.NODE_SHA256, {inputs.architecture: digest}):
            return package.assemble(inputs)

    def test_both_architectures_keep_exact_layout_assets_and_relative_links(
        self,
    ) -> None:
        fixture_root = self.root
        for architecture in ["x86_64", "aarch64"]:
            with self.subTest(architecture=architecture):
                self.root = fixture_root / architecture
                self.root.mkdir()
                inputs, digest = self.fixtures(architecture)
                inputs = replace(
                    inputs, output=self.root / f"output/{architecture}.tar.gz"
                )
                result = self.build(inputs, digest)
                prefix = f"chapchoo-linux-{architecture}"
                with tarfile.open(result, "r:gz") as archive:
                    names = set(archive.getnames())
                    for name in [
                        "chapchoo",
                        "runtime/bin/node",
                        "runtime/LICENSE",
                        "web/general/apps/chapchoo-web/server.js",
                        "web/admin/apps/chapchoo-admin-web/server.js",
                    ]:
                        self.assertIn(f"{prefix}/{name}", names)
                    font = archive.extractfile(
                        f"{prefix}/web/general/apps/chapchoo-web/.next/static/font.woff2"
                    )
                    self.assertIsNotNone(font)
                    if font is not None:
                        self.assertEqual(
                            font.read(), b"wOF2:preserved-real-asset-bytes"
                        )
                    link = archive.getmember(
                        f"{prefix}/web/general/apps/chapchoo-web/node_modules/next"
                    )
                    self.assertTrue(link.issym())
                    self.assertEqual(link.linkname, "../../../node_modules/next")
                    self.assertFalse(
                        any("inputs" in name or ".env" in name for name in names)
                    )
                self.assertEqual(
                    result.with_name(result.name + ".sha256").read_text(),
                    f"{package.sha256(result)}  {result.name}\n",
                )

    def test_outputs_are_deterministic_and_existing_output_is_preserved(self) -> None:
        inputs, digest = self.fixtures()
        first = self.build(inputs, digest)
        second = self.build(
            replace(inputs, output=self.root / "output/second.tar.gz"), digest
        )
        self.assertEqual(first.read_bytes(), second.read_bytes())
        with self.assertRaisesRegex(package.PackageError, "already exists"):
            self.build(inputs, digest)
        self.assertEqual(first.read_bytes(), second.read_bytes())

    def test_missing_public_directory_is_an_explicit_optional_input(self) -> None:
        inputs, digest = self.fixtures()
        inputs = replace(
            inputs,
            general=replace(inputs.general, public=None),
            admin=replace(inputs.admin, public=None),
        )
        self.build(inputs, digest)

    def test_node_digest_must_match_both_pinned_and_supplied_values(self) -> None:
        inputs, digest = self.fixtures()
        with self.assertRaisesRegex(package.PackageError, "checksum"):
            package.assemble(inputs)
        inputs.node_shasums.write_text(
            f"{'0' * 64}  {inputs.node_archive.name}\n", encoding="ascii"
        )
        with (
            patch.dict(package.NODE_SHA256, {inputs.architecture: digest}),
            self.assertRaisesRegex(package.PackageError, "checksum"),
        ):
            package.assemble(inputs)
        self.assertFalse(inputs.output.exists())

    def test_duplicate_checksum_entry_is_rejected(self) -> None:
        inputs, digest = self.fixtures()
        line = inputs.node_shasums.read_text()
        inputs.node_shasums.write_text(line + line)
        with self.assertRaises(package.PackageError):
            self.build(inputs, digest)

    def test_wrong_backend_and_native_addon_architecture_are_rejected(self) -> None:
        inputs, digest = self.fixtures()
        inputs.binary.write_bytes(elf("aarch64"))
        with self.assertRaisesRegex(package.PackageError, "architecture"):
            self.build(inputs, digest)
        inputs.binary.write_bytes(elf("x86_64"))
        (inputs.general.standalone / "node_modules/next/addon.node").write_bytes(
            elf("aarch64")
        )
        with self.assertRaisesRegex(package.PackageError, "architecture"):
            self.build(inputs, digest)
        self.assertFalse(inputs.output.exists())

    def test_credential_database_and_test_output_paths_are_rejected(self) -> None:
        inputs, digest = self.fixtures()
        for relative in [
            ".env",
            ".env.production",
            "private.key",
            "account.sqlite-wal",
            "state/session",
            "test-results/result.txt",
        ]:
            with self.subTest(relative=relative):
                path = inputs.general.public / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("synthetic marker")
                with self.assertRaisesRegex(package.PackageError, "forbidden"):
                    self.build(inputs, digest)
                path.unlink()
                if path.parent != inputs.general.public:
                    path.parent.rmdir()
        self.assertFalse(inputs.output.exists())

    def test_escaping_absolute_broken_and_cyclic_symlinks_are_rejected(self) -> None:
        inputs, digest = self.fixtures()
        outside = self.root / "outside"
        outside.write_text("not an input")
        link = inputs.general.standalone / "escape"
        for target in [str(outside), "../../outside", "missing", "escape"]:
            with self.subTest(target=target):
                link.symlink_to(target)
                with self.assertRaises(package.PackageError):
                    self.build(inputs, digest)
                link.unlink()
        self.assertFalse(inputs.output.exists())

    def test_special_files_other_apps_empty_assets_and_recursive_output_are_rejected(
        self,
    ) -> None:
        inputs, digest = self.fixtures()
        fifo = inputs.general.standalone / "pipe"
        os.mkfifo(fifo)
        with self.assertRaisesRegex(package.PackageError, "Special files"):
            self.build(inputs, digest)
        fifo.unlink()
        other = inputs.general.standalone / "apps/chapchoo-admin-web"
        other.mkdir()
        with self.assertRaisesRegex(package.PackageError, "selected application"):
            self.build(inputs, digest)
        other.rmdir()
        with self.assertRaisesRegex(package.PackageError, "outside"):
            self.build(
                replace(inputs, output=inputs.general.static / "package.tar.gz"), digest
            )
        (inputs.general.static / "font.woff2").unlink()
        with self.assertRaisesRegex(package.PackageError, "empty"):
            self.build(inputs, digest)

    def test_node_tar_rejects_traversal_special_files_and_escaping_links(self) -> None:
        prefix = "node-v24.22.1-linux-x64"
        required = [
            member(f"{prefix}/bin/node", elf("x86_64"))[0],
            member(f"{prefix}/LICENSE", b"license")[0],
        ]
        for name in [
            "../escape",
            "/absolute",
            f"{prefix}/../escape",
            f"{prefix}\\escape",
        ]:
            with self.subTest(name=name), self.assertRaises(package.PackageError):
                package.validate_node_members(
                    [*required, tarfile.TarInfo(name)], prefix
                )
        link = tarfile.TarInfo(f"{prefix}/bin/link")
        link.type = tarfile.SYMTYPE
        link.linkname = "../../../escape"
        with self.assertRaises(package.PackageError):
            package.validate_node_members([*required, link], prefix)
        link.type = tarfile.LNKTYPE
        link.linkname = "outside"
        with self.assertRaises(package.PackageError):
            package.validate_node_members([*required, link], prefix)
        special = tarfile.TarInfo(f"{prefix}/device")
        special.type = tarfile.CHRTYPE
        with self.assertRaises(package.PackageError):
            package.validate_node_members([*required, special], prefix)
        alias = member(f"{prefix}/./bin/node", b"duplicate")[0]
        with self.assertRaisesRegex(package.PackageError, "duplicate"):
            package.validate_node_members([*required, alias], prefix)

    def test_help_is_offline_and_does_not_execute_inputs(self) -> None:
        result = subprocess.run(
            [sys.executable, str(ROOT / "scripts/package-linux.py"), "--help"],
            check=True,
            capture_output=True,
            text=True,
        )
        self.assertIn("--architecture", result.stdout)
        self.assertIn("--node-shasums", result.stdout)

    def test_image_stager_contains_only_the_selected_synthetic_app(self) -> None:
        inputs, _ = self.fixtures()
        typescript = self.root / "typescript"
        app_root = typescript / "apps/chapchoo-web"
        for source, relative in [
            (inputs.general.standalone, ".next/standalone"),
            (inputs.general.static, ".next/static"),
            (inputs.general.public, "public"),
        ]:
            shutil.copytree(source, app_root / relative, symlinks=True)
        output = self.root / "image"
        command = [
            sys.executable,
            str(ROOT / "images/chapchoo/stage-web.py"),
            "--app",
            "general",
            "--architecture",
            "amd64",
            "--typescript-root",
            str(typescript),
            "--binary",
            str(inputs.binary),
            "--node",
            str(inputs.binary),
            "--node-license",
            str(ROOT / "LICENSE"),
            "--output",
            str(output),
        ]
        subprocess.run(command, check=True, capture_output=True, text=True)
        self.assertTrue((output / "chapchoo").is_file())
        self.assertTrue((output / "runtime/bin/node").is_file())
        self.assertTrue((output / "web/general/apps/chapchoo-web/server.js").is_file())
        self.assertFalse((output / "web/admin").exists())
        recursive = [*command[:-1], str(typescript / "recursive-output")]
        result = subprocess.run(recursive, check=False, capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((typescript / "recursive-output").exists())

    def test_node_extraction_never_follows_archive_parent_symlinks(self) -> None:
        prefix = "node-v24.22.1-linux-x64"
        archive = self.root / "parent-symlink.tar.xz"
        with tarfile.open(archive, "w:xz") as output:
            parent = tarfile.TarInfo(f"{prefix}/bin")
            parent.type = tarfile.SYMTYPE
            parent.linkname = "different-internal-directory"
            output.addfile(parent)
            for name, data in [
                (f"{prefix}/bin/node", elf("x86_64")),
                (f"{prefix}/LICENSE", b"license"),
            ]:
                info, stream = member(name, data)
                output.addfile(info, stream)
        destination = self.root / "runtime"
        package.extract_node(archive, destination, prefix, "x86_64")
        self.assertFalse((destination / "bin").is_symlink())
        self.assertEqual((destination / "bin/node").read_bytes(), elf("x86_64"))
        self.assertFalse((destination / "different-internal-directory").exists())

    def test_independent_docker_target_contracts(self) -> None:
        dockerfile = (ROOT / "images/chapchoo/Dockerfile").read_text()
        backend = dockerfile.split("FROM runtime-base AS backend\n", 1)[1].split(
            "FROM runtime-base AS web-runtime-base", 1
        )[0]
        self.assertNotIn("--from=general-build", backend)
        self.assertNotIn("--from=admin-build", backend)
        self.assertNotIn("runtime/bin/node", backend)
        self.assertIn("AS general-web", dockerfile)
        self.assertIn("AS admin-web", dockerfile)
        self.assertIn('"--role", "none", "--web", "general"', dockerfile)
        self.assertIn('"--role", "none", "--web", "admin"', dockerfile)
        self.assertIn("USER 10001:10001", dockerfile)
        self.assertNotIn("CHAPCHOO_INSTALLATION_KEY=", dockerfile)


if __name__ == "__main__":
    unittest.main()
