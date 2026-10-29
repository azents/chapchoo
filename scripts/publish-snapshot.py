#!/usr/bin/env python3
"""Publish one complete, native-verified Chapchoo image snapshot."""

from __future__ import annotations

import argparse
import datetime
import json
import os
import re
import subprocess
from pathlib import Path

COMPONENTS = ("backend", "general-web", "admin-web")
ARCHITECTURES = ("amd64", "arm64")
DIGEST = re.compile(r"sha256:[0-9a-f]{64}")


def read_metadata(directory: Path) -> dict[str, list[str]]:
    expected = {
        f"{component}-{arch}.json" for component in COMPONENTS for arch in ARCHITECTURES
    }
    if {p.name for p in directory.iterdir()} != expected:
        raise ValueError("Snapshot requires exactly six native image metadata files")
    result: dict[str, list[str]] = {}
    for component in COMPONENTS:
        image = f"ghcr.io/azents/chapchoo-{component}-snapshot"
        result[component] = []
        for architecture in ARCHITECTURES:
            item = json.loads(
                (directory / f"{component}-{architecture}.json").read_text()
            )
            if (
                not isinstance(item, dict)
                or item.get("component") != component
                or item.get("architecture") != architecture
                or item.get("image") != image
                or not isinstance(item.get("digest"), str)
                or not DIGEST.fullmatch(item["digest"])
            ):
                raise ValueError(
                    "Native snapshot metadata has an invalid identity or digest"
                )
            result[component].append(f"{image}@{item['digest']}")
    return result


def docker(*arguments: str) -> str:
    return subprocess.check_output(
        ["docker", "buildx", "imagetools", *arguments], text=True
    )


def publish(directory: Path, output: Path) -> None:
    source = os.environ["GITHUB_SHA"]
    run_id = os.environ["GITHUB_RUN_ID"]
    attempt = os.environ["GITHUB_RUN_ATTEMPT"]
    if (
        os.environ["GITHUB_REPOSITORY"] != "azents/chapchoo"
        or os.environ["GITHUB_REF"] != "refs/heads/main"
        or re.fullmatch(r"[0-9a-f]{40}", source) is None
        or not run_id.isdecimal()
        or not attempt.isdecimal()
    ):
        raise ValueError(
            "Deployable snapshots require the canonical repository main source"
        )
    inputs = read_metadata(directory)
    date = datetime.datetime.now(datetime.UTC).strftime("%Y%m%d")
    tag = f"dev-main-{date}-{source[:12]}-{run_id}-{attempt}"
    images = []
    for component, references in inputs.items():
        image = f"ghcr.io/azents/chapchoo-{component}-snapshot"
        docker("create", "--tag", f"{image}:{tag}", *references)
        # Inspect the registry's actual content identity, not a guessed hash of
        # CLI-formatted JSON. Reject incomplete indexes before downstream use.
        manifest = json.loads(
            docker("inspect", f"{image}:{tag}", "--format", "{{json .Manifest}}")
        )
        digest = manifest.get("digest")
        raw = json.loads(docker("inspect", f"{image}:{tag}", "--raw"))
        platforms = {
            (m.get("platform", {}).get("os"), m.get("platform", {}).get("architecture"))
            for m in raw.get("manifests", [])
        }
        if (
            not isinstance(digest, str)
            or not DIGEST.fullmatch(digest)
            or platforms != {("linux", "amd64"), ("linux", "arm64")}
        ):
            raise ValueError(
                "Published index must contain exactly both native Linux architectures"
            )
        images.append(
            {"component": component, "image": image, "uniqueTag": tag, "digest": digest}
        )
    payload = {
        "repository": "azents/chapchoo",
        "ref": "refs/heads/main",
        "sha": source,
        "runId": run_id,
        "runAttempt": attempt,
        "images": images,
    }
    output.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    publish(args.metadata, args.output)


if __name__ == "__main__":
    main()
