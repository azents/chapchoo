"""Full-CI ownership and single-build artifact contract checks."""

from __future__ import annotations

import json
import re
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class CiStructureTests(unittest.TestCase):
    def test_rust_and_frontend_checks_have_one_ci_owner(self) -> None:
        workflow = (ROOT / ".github/workflows/ci.yaml").read_text()
        self.assertEqual(workflow.count("run: bash scripts/check-rust.sh"), 1)
        self.assertIn("SKIP: rust-quality,ts-format,ts-lint,ts-typecheck", workflow)
        self.assertIn(
            "pnpm run format:check && pnpm run lint && pnpm run typecheck", workflow
        )
        self.assertIn("run: uvx pre-commit run --all-files", workflow)
        hooks = (ROOT / ".pre-commit-config.yaml").read_text()
        self.assertIn("entry: bash scripts/check-rust.sh", hooks)
        self.assertIn("entry: bash scripts/check-rpc.sh", hooks)
        rust_job = workflow.split("  validate:", 1)[0]
        self.assertIn('CARGO_PROFILE_DEV_DEBUG: "0"', rust_job)
        self.assertIn('CARGO_PROFILE_TEST_DEBUG: "0"', rust_job)

    def test_native_acceptance_retains_all_assembly_boundaries(self) -> None:
        workflow = (ROOT / ".github/workflows/linux-package.yaml").read_text()
        for required in [
            "architecture: x86_64",
            "architecture: aarch64",
            "scripts/test-installed-linux.py",
            "scripts/test-container-images.py",
            "pnpm --filter @chapchoo/e2e test:e2e",
            "sha256sum --check",
            "scripts/package-linux.py",
            'test "$(uname -m)" = "$EXPECTED_ARCH"',
        ]:
            self.assertIn(required, workflow)
        self.assertNotIn("docker build ", workflow)
        self.assertNotIn("turbo run build", workflow)
        self.assertIn('web-artifacts/general/standalone"', workflow)
        self.assertIn('web-artifacts/admin/standalone"', workflow)
        self.assertIn("needs: [web-build, binary-build]", workflow)
        self.assertEqual(workflow.count("uses: docker/bake-action@"), 1)
        self.assertNotIn("push: true", workflow)
        for filename in ["linux-package.yaml", "snapshot.yaml"]:
            text = (ROOT / ".github/workflows" / filename).read_text()
            self.assertIn(
                "source: .\n          files: images/chapchoo/ci-bake.hcl", text
            )
            self.assertIn("fs.read=${{ runner.temp }}/web-artifacts", text)
            self.assertIn("fs.read=${{ runner.temp }}/binary-artifacts", text)

    def test_bake_targets_share_builder_outputs_without_publishing(self) -> None:
        result = subprocess.run(
            [
                "docker",
                "buildx",
                "bake",
                "-f",
                "images/chapchoo/ci-bake.hcl",
                "--print",
            ],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=True,
        )
        targets = json.loads(result.stdout)["target"]
        self.assertEqual(set(targets), {"backend", "general-web", "admin-web"})
        for name in ["backend", "general-web", "admin-web"]:
            self.assertEqual(targets[name]["target"], name)
            self.assertEqual(targets[name]["output"][0]["type"], "docker")
        dockerfile = (ROOT / "images/chapchoo/Dockerfile").read_text()
        self.assertIn("COPY --from=binary-artifacts", dockerfile)
        self.assertIn("COPY --from=web-artifacts", dockerfile)
        self.assertNotIn("cargo build", dockerfile)
        self.assertNotIn("pnpm", dockerfile)
        self.assertNotIn("COPY typescript", dockerfile)
        self.assertEqual(
            set(targets["general-web"]["contexts"]),
            {"binary-artifacts", "web-artifacts"},
        )

    def test_new_actions_are_immutable_and_cache_has_architecture_scope(self) -> None:
        for filename in [
            "ci.yaml",
            "linux-package.yaml",
            "build-web.yaml",
            "build-binary.yaml",
            "snapshot.yaml",
        ]:
            content = (ROOT / ".github/workflows" / filename).read_text()
            for action in re.findall(r"uses: ([^\s]+)", content):
                if action.startswith("./.github/workflows/"):
                    continue
                self.assertRegex(action, r"^[^@]+@[0-9a-f]{40}$")
        native = (ROOT / ".github/workflows/linux-package.yaml").read_text()
        self.assertEqual(native.count("cache-to=type=gha,scope="), 3)
        for line in native.splitlines():
            if "cache-to=" in line or "cache-from=" in line:
                self.assertIn("${{ matrix.architecture }}", line)

    def test_local_mac_wrapper_builds_linux_before_packaging(self) -> None:
        script = (ROOT / "scripts/build-images.sh").read_text()
        self.assertLess(
            script.index("scripts/build-web.sh"), script.index("docker buildx bake")
        )
        self.assertLess(
            script.index("scripts/build-binary.sh"), script.index("docker buildx bake")
        )
        for filename in ["build-web.sh", "build-binary.sh"]:
            text = (ROOT / "scripts" / filename).read_text()
            self.assertIn("aarch64|arm64) architecture=arm64", text)
            self.assertIn('--platform "linux/$architecture"', text)
        web = (ROOT / ".github/workflows/build-web.yaml").read_text()
        self.assertIn("runner.arch", web)
        self.assertIn("hashFiles('typescript/pnpm-lock.yaml')", web)
        self.assertIn("pnpm exec turbo run build", web)


if __name__ == "__main__":
    unittest.main()
