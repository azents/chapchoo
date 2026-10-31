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
        self.assertIn("  browser-e2e:\n", workflow)
        self.assertIn("run: pnpm --filter @chapchoo/e2e test:e2e", workflow)
        for command in [
            "pnpm --filter @chapchoo/testenv test",
            "pnpm --filter @chapchoo/ui test:components",
            "pnpm --filter @chapchoo/web-core test:browser",
            "pnpm --filter @chapchoo/web test:components",
            "pnpm --filter @chapchoo/admin-web test:components",
        ]:
            self.assertIn(command, workflow)
        rust_job = workflow.split("  validate:", 1)[0]
        self.assertIn('CARGO_PROFILE_DEV_DEBUG: "0"', rust_job)
        self.assertIn('CARGO_PROFILE_TEST_DEBUG: "0"', rust_job)

    def test_native_build_exports_outputs_to_one_downstream_browser_gate(self) -> None:
        workflow = (ROOT / ".github/workflows/linux-package.yaml").read_text()
        for required in [
            "architecture: x86_64",
            "architecture: aarch64",
            "scripts/package-linux.py",
            'test "$(uname -m)" = "$EXPECTED_ARCH"',
            "workflow_call:",
        ]:
            self.assertIn(required, workflow)
        self.assertNotIn("test:e2e", workflow)
        self.assertNotIn("test-installed-linux.py", workflow)
        self.assertIn(
            "python3 scripts/test-container-images.py \\\n"
            "            --backend chapchoo-package-backend:ci \\\n"
            "            --general chapchoo-package-general:ci \\\n"
            "            --admin chapchoo-package-admin:ci",
            workflow,
        )
        self.assertLess(
            workflow.index("uses: docker/bake-action@"),
            workflow.index("python3 scripts/test-container-images.py"),
        )
        self.assertLess(
            workflow.index("python3 scripts/test-container-images.py"),
            workflow.index("python3 scripts/package-linux.py"),
        )
        ci = (ROOT / ".github/workflows/ci.yaml").read_text()
        self.assertIn("uses: ./.github/workflows/linux-package.yaml", ci)
        browser = ci.split("  browser-e2e:", 1)[1]
        self.assertIn("needs: native-build", browser)
        self.assertIn("name: chapchoo-linux-x86_64", browser)
        self.assertIn("sha256sum --check chapchoo-linux-x86_64.tar.gz.sha256", browser)
        self.assertIn("CHAPCHOO_E2E_PACKAGE_ROOT=", browser)
        self.assertEqual(ci.count("run: pnpm --filter @chapchoo/e2e test:e2e"), 1)
        self.assertNotIn("          pnpm run build\n", ci)
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

    def test_root_workspace_owns_shared_testenv_and_builder_inputs(self) -> None:
        workspace = (ROOT / "pnpm-workspace.yaml").read_text()
        for path in ["typescript/apps/*", "typescript/packages/*", "testenv/chapchoo"]:
            self.assertIn(path, workspace)
        for filename in ["ci.yaml", "build-web.yaml"]:
            workflow = (ROOT / ".github/workflows" / filename).read_text()
            self.assertIn("package_json_file: package.json", workflow)
            self.assertIn("cache-dependency-path: pnpm-lock.yaml", workflow)
        builder = (ROOT / "images/chapchoo/web-build.Dockerfile").read_text()
        self.assertIn("COPY testenv/chapchoo ./testenv/chapchoo", builder)
        self.assertIn(
            "COPY package.json pnpm-workspace.yaml pnpm-lock.yaml turbo.json ./",
            builder,
        )
        hooks = (ROOT / ".pre-commit-config.yaml").read_text()
        self.assertIn("testenv/chapchoo/", hooks)

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
        self.assertIn("hashFiles('pnpm-lock.yaml')", web)
        self.assertIn("pnpm exec turbo run build", web)


if __name__ == "__main__":
    unittest.main()
