"""Snapshot publication guards and workflow boundaries without registry writes."""

from __future__ import annotations

import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "publish_snapshot", ROOT / "scripts/publish-snapshot.py"
)
assert SPEC is not None and SPEC.loader is not None
publisher = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(publisher)


class SnapshotTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.metadata = self.root / "metadata"
        self.metadata.mkdir()
        for component in publisher.COMPONENTS:
            for arch in publisher.ARCHITECTURES:
                (self.metadata / f"{component}-{arch}.json").write_text(
                    json.dumps(
                        {
                            "component": component,
                            "architecture": arch,
                            "image": f"ghcr.io/azents/chapchoo-{component}-snapshot",
                            "digest": "sha256:"
                            + ("a" if arch == "amd64" else "b") * 64,
                        }
                    )
                )
        self.env = {
            "GITHUB_REPOSITORY": "azents/chapchoo",
            "GITHUB_REF": "refs/heads/main",
            "GITHUB_SHA": "c" * 40,
            "GITHUB_RUN_ID": "123",
            "GITHUB_RUN_ATTEMPT": "2",
        }

    def test_complete_metadata_has_exact_expected_sources(self) -> None:
        result = publisher.read_metadata(self.metadata)
        self.assertEqual(set(result), set(publisher.COMPONENTS))
        self.assertTrue(all(len(refs) == 2 for refs in result.values()))

    def test_missing_extra_identity_and_digest_fail(self) -> None:
        path = self.metadata / "backend-amd64.json"
        original = path.read_text()
        path.unlink()
        with self.assertRaises(ValueError):
            publisher.read_metadata(self.metadata)
        path.write_text(original)
        extra = self.metadata / "extra.json"
        extra.write_text("{}")
        with self.assertRaises(ValueError):
            publisher.read_metadata(self.metadata)
        extra.unlink()
        for field, value in [
            ("component", "foreign"),
            ("architecture", "arm64"),
            ("image", "ghcr.io/foreign/image"),
            ("digest", "sha256:bad"),
        ]:
            data = json.loads(original)
            data[field] = value
            path.write_text(json.dumps(data))
            with self.assertRaises(ValueError):
                publisher.read_metadata(self.metadata)
        path.write_text(original)

    def test_publish_requires_main_and_canonical_repository(self) -> None:
        for field, value in [
            ("GITHUB_REF", "refs/heads/feature"),
            ("GITHUB_REPOSITORY", "fork/chapchoo"),
            ("GITHUB_SHA", "bad"),
        ]:
            with (
                patch.dict(os.environ, {**self.env, field: value}, clear=True),
                self.assertRaises(ValueError),
            ):
                publisher.publish(self.metadata, self.root / "output.json")

    def test_publish_checks_registry_index_and_writes_one_complete_payload(
        self,
    ) -> None:
        def docker(*arguments: str) -> str:
            if arguments[0] == "create":
                return ""
            if arguments[-1] == "--raw":
                return json.dumps(
                    {
                        "manifests": [
                            {"platform": {"os": "linux", "architecture": arch}}
                            for arch in publisher.ARCHITECTURES
                        ]
                    }
                )
            return json.dumps({"digest": "sha256:" + "d" * 64})

        output = self.root / "output.json"
        with (
            patch.dict(os.environ, self.env, clear=True),
            patch.object(publisher, "docker", side_effect=docker),
        ):
            publisher.publish(self.metadata, output)
        payload = json.loads(output.read_text())
        self.assertEqual(payload["sha"], self.env["GITHUB_SHA"])
        self.assertEqual(payload["runAttempt"], "2")
        self.assertEqual(len(payload["images"]), 3)
        self.assertEqual(len({item["uniqueTag"] for item in payload["images"]}), 1)

    def test_wrong_registry_platform_never_emits_payload(self) -> None:
        def docker(*arguments: str) -> str:
            if arguments[0] == "create":
                return ""
            return json.dumps({"digest": "sha256:" + "d" * 64, "manifests": []})

        output = self.root / "output.json"
        with (
            patch.dict(os.environ, self.env, clear=True),
            patch.object(publisher, "docker", side_effect=docker),
            self.assertRaises(ValueError),
        ):
            publisher.publish(self.metadata, output)
        self.assertFalse(output.exists())

    def test_workflow_exercises_prefix_before_push_and_uses_native_runners(
        self,
    ) -> None:
        text = (ROOT / ".github/workflows/snapshot.yaml").read_text()
        self.assertIn("ubuntu-24.04-arm", text)
        self.assertIn('test "$(uname -m)" = "$MACHINE"', text)
        self.assertIn("--general chapchoo-general-web:acceptance", text)
        self.assertIn("--admin chapchoo-admin-web:acceptance", text)
        self.assertLess(
            text.index("scripts/test-container-images.py"), text.index("docker push")
        )
        self.assertIn("needs: build", text)
        self.assertIn("name: snapshot-payload", text)
        self.assertNotIn("pull_request_target", text)
        self.assertNotIn("gh pr merge", text)


if __name__ == "__main__":
    unittest.main()
