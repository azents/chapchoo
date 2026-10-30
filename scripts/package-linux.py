#!/usr/bin/env python3
"""Assemble a source-free Linux distribution from explicit, built artifacts.

This tool never builds, downloads, executes an input, or falls back to host Node.
Official Node SHA-256 values are pinned from its first-party release metadata.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import os
import re
import shutil
import stat
import struct
import tarfile
import tempfile
from dataclasses import dataclass
from pathlib import Path, PurePosixPath

NODE_VERSION = "24.22.1"
PROJECT_LICENSE = Path(__file__).resolve().parent.parent / "LICENSE"
# https://nodejs.org/dist/v24.22.1/SHASUMS256.txt
NODE_SHA256 = {
    "x86_64": "6c172a9ab6718f7b801fb5a7bd486af2ee5e32758086a04b84fcce2aa77095e6",
    "aarch64": "dbabf7a1cbf947e6059ffad2c27b80041db97dea1e553b62afdee7f13fca336e",
}
NODE_ARCH = {"x86_64": "x64", "aarch64": "arm64"}
ELF_MACHINE = {"x86_64": 62, "aarch64": 183}
FORBIDDEN_NAMES = {
    ".git",
    ".azents",
    ".aws",
    ".ssh",
    ".npmrc",
    ".netrc",
    ".pypirc",
    ".turbo",
    "storybook-static",
    "test-results",
    "playwright-report",
    "coverage",
    "secrets",
    "credentials",
    "keys",
    "runtime-data",
    "__pycache__",
}
FORBIDDEN_SUFFIXES = (".key", ".pem", ".p12", ".pfx", ".sqlite", ".sqlite3", ".db")


class PackageError(ValueError):
    """A supplied artifact violates the installation contract."""


@dataclass(frozen=True)
class WebArtifacts:
    standalone: Path
    static: Path
    public: Path | None
    app: str


@dataclass(frozen=True)
class PackageInputs:
    architecture: str
    binary: Path
    node_archive: Path
    node_shasums: Path
    general: WebArtifacts
    admin: WebArtifacts
    output: Path
    epoch: int


def sha256(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def forbidden(path: PurePosixPath) -> bool:
    for part in path.parts:
        name = part.lower()
        if (
            name in FORBIDDEN_NAMES
            or name == ".env"
            or name.startswith(".env.")
            or name.endswith(FORBIDDEN_SUFFIXES)
            or re.search(r"\.(?:db|sqlite|sqlite3)-(?:wal|shm|journal)$", name)
        ):
            return True
    # Node's runtime libraries can legitimately have state implementations. The
    # standalone root itself, however, must not contain installation state.
    return bool(
        path.parts and path.parts[0].lower() in {"state", "data", "logs"}
    ) or any(
        parent == ".next" and child == "cache"
        for parent, child in zip(path.parts, path.parts[1:], strict=False)
    )


def elf_architecture(path: Path, architecture: str) -> None:
    with path.open("rb") as source:
        header = source.read(20)
    if (
        len(header) != 20
        or header[:4] != b"\x7fELF"
        or header[4:7] != b"\x02\x01\x01"
        or struct.unpack_from("<H", header, 16)[0] not in {2, 3}
        or struct.unpack_from("<H", header, 18)[0] != ELF_MACHINE[architecture]
    ):
        raise PackageError(
            "Executable or native library does not match the selected Linux architecture"
        )


def validate_tree(root: Path, architecture: str) -> None:
    root = root.resolve(strict=True)
    if not root.is_dir():
        raise PackageError("A web artifact root is not a directory")
    for path in sorted(root.rglob("*")):
        relative = PurePosixPath(path.relative_to(root).as_posix())
        if forbidden(relative):
            raise PackageError(
                "A web artifact contains a forbidden credential, state or test-output path"
            )
        mode = path.lstat().st_mode
        if stat.S_ISLNK(mode):
            target = os.readlink(path)
            if os.path.isabs(target) or "\\" in target:
                raise PackageError(
                    "Standalone symlinks must be relative and relocatable"
                )
            try:
                resolved = path.resolve(strict=True)
            except (OSError, RuntimeError) as error:
                raise PackageError(
                    "A standalone symlink is broken or cyclic"
                ) from error
            if not resolved.is_relative_to(root) or forbidden(
                PurePosixPath(resolved.relative_to(root).as_posix())
            ):
                raise PackageError(
                    "A standalone symlink escapes its supplied artifact root"
                )
        elif stat.S_ISREG(mode):
            with path.open("rb") as source:
                magic = source.read(4)
            if (
                magic == b"\x7fELF"
                or path.name.endswith(".node")
                or ".so" in path.suffixes
            ):
                elf_architecture(path, architecture)
        elif not stat.S_ISDIR(mode):
            raise PackageError("Special files are not accepted in web artifacts")


def verify_node_archive(archive: Path, shasums: Path, architecture: str) -> str:
    filename = f"node-v{NODE_VERSION}-linux-{NODE_ARCH[architecture]}.tar.xz"
    if archive.name != filename or not archive.is_file() or archive.is_symlink():
        raise PackageError(
            "The Node input must be the selected official release archive"
        )
    matches = []
    for line in shasums.read_text(encoding="ascii").splitlines():
        match = re.fullmatch(r"([0-9a-f]{64})  (\S+)", line)
        if match is None:
            raise PackageError("Malformed official Node SHA-256 inventory")
        if match[2] == filename:
            matches.append(match[1])
    expected = NODE_SHA256[architecture]
    if matches != [expected] or sha256(archive) != expected:
        raise PackageError(
            "The Node archive, supplied inventory and pinned first-party checksum do not agree"
        )
    return f"node-v{NODE_VERSION}-linux-{NODE_ARCH[architecture]}"


def clean_member_path(value: str) -> PurePosixPath:
    path = PurePosixPath(value)
    if (
        not value
        or "\\" in value
        or path.is_absolute()
        or ".." in path.parts
        or not path.parts
    ):
        raise PackageError("Unsafe path in Node archive")
    return path


def validate_node_members(
    members: list[tarfile.TarInfo], prefix: str
) -> dict[str, tarfile.TarInfo]:
    inventory: dict[str, tarfile.TarInfo] = {}
    for member in members:
        path = clean_member_path(member.name)
        if path.parts[0] != prefix or str(path) in inventory:
            raise PackageError("Unexpected or duplicate Node archive member")
        if not (member.isfile() or member.isdir() or member.issym() or member.islnk()):
            raise PackageError("Special files are forbidden in the Node archive")
        if member.issym():
            target = PurePosixPath(member.linkname)
            if target.is_absolute() or "\\" in member.linkname:
                raise PackageError("Unsafe Node archive symlink")
            normalized = os.path.normpath(str(path.parent / target))
            if not PurePosixPath(normalized).is_relative_to(PurePosixPath(prefix)):
                raise PackageError("Node archive symlink escapes its release root")
        elif member.islnk():
            target = clean_member_path(member.linkname)
            if target.parts[0] != prefix:
                raise PackageError("Node archive hard link escapes its release root")
        inventory[str(path)] = member
    # We extract only explicitly named regular files, never archive links or
    # directories. Even a safe-looking symlink parent cannot redirect writes.
    for name in [f"{prefix}/bin/node", f"{prefix}/LICENSE"]:
        if name not in inventory or not inventory[name].isfile():
            raise PackageError(
                "The official Node archive is missing a required regular file"
            )
    return inventory


def extract_node(
    archive: Path, destination: Path, prefix: str, architecture: str
) -> None:
    with tarfile.open(archive, "r:xz") as source:
        inventory = validate_node_members(source.getmembers(), prefix)
        for name, relative in [
            (f"{prefix}/bin/node", "bin/node"),
            (f"{prefix}/LICENSE", "LICENSE"),
        ]:
            member = inventory[name]
            stream = source.extractfile(member)
            if stream is None:
                raise PackageError("The Node archive file could not be read")
            target = destination / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            with stream, target.open("xb") as output:
                shutil.copyfileobj(stream, output)
            target.chmod(0o755 if relative == "bin/node" else 0o644)
    elf_architecture(destination / "bin/node", architecture)


def copy_web(artifacts: WebArtifacts, destination: Path, architecture: str) -> None:
    for root in [artifacts.standalone, artifacts.static, artifacts.public]:
        if root is not None:
            validate_tree(root, architecture)
    roots = {path.name for path in artifacts.standalone.iterdir()}
    if not roots.issubset({"apps", "node_modules", "packages", "package.json"}):
        raise PackageError("The standalone input is not a traced Next workspace tree")
    entry = artifacts.standalone / "apps" / artifacts.app / "server.mjs"
    if not entry.is_file() or entry.is_symlink():
        raise PackageError("The traced Next entrypoint is missing")
    if {path.name for path in (artifacts.standalone / "apps").iterdir()} != {
        artifacts.app
    }:
        raise PackageError(
            "A standalone tree must contain only its selected application"
        )
    if not any(path.is_file() for path in artifacts.static.rglob("*")):
        raise PackageError("The built static asset tree is empty")
    shutil.copytree(artifacts.standalone, destination, symlinks=True)
    app_root = destination / "apps" / artifacts.app
    shutil.copytree(
        artifacts.static, app_root / ".next/static", dirs_exist_ok=True, symlinks=True
    )
    if artifacts.public is not None:
        shutil.copytree(
            artifacts.public, app_root / "public", dirs_exist_ok=True, symlinks=True
        )
    validate_tree(destination, architecture)


def write_archive(root: Path, archive: Path, epoch: int) -> None:
    def normalize(member: tarfile.TarInfo) -> tarfile.TarInfo:
        member.uid = member.gid = 0
        member.uname = member.gname = ""
        member.mtime = epoch
        member.pax_headers = {}
        member.mode = 0o755 if member.isdir() or member.mode & 0o111 else 0o644
        if member.issym():
            member.mode = 0o777
        return member

    with (
        archive.open("xb") as output,
        gzip.GzipFile(
            filename="", mode="wb", fileobj=output, mtime=epoch
        ) as compressed,
        tarfile.open(
            fileobj=compressed, mode="w", format=tarfile.PAX_FORMAT, dereference=False
        ) as bundle,
    ):
        for path in [root, *sorted(root.rglob("*"))]:
            bundle.add(
                path,
                arcname=path.relative_to(root.parent).as_posix(),
                recursive=False,
                filter=normalize,
            )


def assemble(inputs: PackageInputs) -> Path:
    if inputs.architecture not in NODE_ARCH or inputs.epoch < 0:
        raise PackageError(
            "Unsupported architecture or invalid deterministic timestamp"
        )
    if inputs.output.suffixes[-2:] != [".tar", ".gz"]:
        raise PackageError("The package output must end in .tar.gz")
    checksum = inputs.output.with_name(inputs.output.name + ".sha256")
    if inputs.output.exists() or checksum.exists():
        raise PackageError("Package output already exists")
    if (
        inputs.binary.is_symlink()
        or not inputs.binary.is_file()
        or not os.access(inputs.binary, os.X_OK)
    ):
        raise PackageError(
            "The supplied backend executable is not an executable regular file"
        )
    elf_architecture(inputs.binary, inputs.architecture)
    prefix = verify_node_archive(
        inputs.node_archive, inputs.node_shasums, inputs.architecture
    )
    for web in [inputs.general, inputs.admin]:
        for source in [web.standalone, web.static, web.public]:
            if source is not None and inputs.output.parent.resolve().is_relative_to(
                source.resolve(strict=True)
            ):
                raise PackageError(
                    "Package output must be outside every supplied web artifact tree"
                )
    inputs.output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(
        prefix=".chapchoo-package-", dir=inputs.output.parent
    ) as temporary:
        staging = Path(temporary)
        root = staging / f"chapchoo-linux-{inputs.architecture}"
        root.mkdir()
        shutil.copyfile(inputs.binary, root / "chapchoo")
        (root / "chapchoo").chmod(0o755)
        shutil.copyfile(PROJECT_LICENSE, root / "LICENSE")
        extract_node(inputs.node_archive, root / "runtime", prefix, inputs.architecture)
        copy_web(inputs.general, root / "web/general", inputs.architecture)
        copy_web(inputs.admin, root / "web/admin", inputs.architecture)
        archive = staging / "package.tar.gz"
        write_archive(root, archive, inputs.epoch)
        digest = sha256(archive)
        # Same-filesystem exclusive links publish complete files without
        # replacing an existing output or exposing a partially written archive.
        os.link(archive, inputs.output)
        with checksum.open("x", encoding="ascii") as output:
            output.write(f"{digest}  {inputs.output.name}\n")
    return inputs.output


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--architecture", choices=NODE_ARCH, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--node-archive", type=Path, required=True)
    parser.add_argument("--node-shasums", type=Path, required=True)
    for app in ["general", "admin"]:
        parser.add_argument(f"--{app}-standalone", type=Path, required=True)
        parser.add_argument(f"--{app}-static", type=Path, required=True)
        parser.add_argument(f"--{app}-public", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--source-date-epoch", type=int, default=0)
    options = parser.parse_args()
    assemble(
        PackageInputs(
            architecture=options.architecture,
            binary=options.binary,
            node_archive=options.node_archive,
            node_shasums=options.node_shasums,
            general=WebArtifacts(
                options.general_standalone,
                options.general_static,
                options.general_public,
                "chapchoo-web",
            ),
            admin=WebArtifacts(
                options.admin_standalone,
                options.admin_static,
                options.admin_public,
                "chapchoo-admin-web",
            ),
            output=options.output,
            epoch=options.source_date_epoch,
        )
    )
    print("Linux distribution and SHA-256 sidecar assembled successfully")


if __name__ == "__main__":
    main()
