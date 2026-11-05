"""Full-CI ownership and single-build artifact contract checks."""

from __future__ import annotations

import json
import os
import re
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class CiStructureTests(unittest.TestCase):
    def test_rpc_typecheck_waits_for_its_source_generation(self) -> None:
        result = subprocess.run(
            ["pnpm", "exec", "turbo", "run", "typecheck", "--dry=json"],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=True,
        )
        tasks = {task["taskId"]: task for task in json.loads(result.stdout)["tasks"]}
        self.assertIn(
            "@chapchoo/rpc-client#generate",
            tasks["@chapchoo/rpc-client#typecheck"]["dependencies"],
        )

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
        rust_job = workflow.split("  rust:", 1)[1].split("  repository-checks:", 1)[0]
        self.assertIn('CARGO_PROFILE_DEV_DEBUG: "0"', rust_job)
        self.assertIn('CARGO_PROFILE_TEST_DEBUG: "0"', rust_job)

    def test_native_build_exports_outputs_to_one_downstream_browser_gate(self) -> None:
        workflow = (ROOT / ".github/workflows/linux-package.yaml").read_text()
        for required in [
            "architecture:\n        required: true",
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
        self.assertNotIn("needs:", workflow)
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
                self.assertIn("${{ inputs.architecture }}", line)

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

    def test_real_docker_integration_has_a_native_owner_and_same_run_binary(
        self,
    ) -> None:
        ci = (ROOT / ".github/workflows/ci.yaml").read_text()
        browser, native = ci.split("  browser-e2e:", 1)[1].split(
            "  docker-integration:", 1
        )
        for required in [
            "needs: binary-x86",
            "name: binary-build-x86_64",
            "binary-artifacts.tar",
            "--build-context",
            "CHAPCHOO_INTEGRATION_BINARY=",
            "CHAPCHOO_INTEGRATION_DOCKER_AUTHORIZED=disposable",
            "CHAPCHOO_INTEGRATION_DOCKER_SOCKET=/var/run/docker.sock",
            "CHAPCHOO_INTEGRATION_DOCKER_IMAGE=chapchoo-computer-acceptance:ci",
            "pnpm --filter @chapchoo/integration test:integration",
        ]:
            self.assertIn(required, native)
        self.assertIn("pnpm --filter @chapchoo/e2e test:e2e", browser)
        self.assertNotIn("CHAPCHOO_INTEGRATION_DOCKER", browser)
        self.assertNotIn("playwright", native)
        image = (ROOT / "images/chapchoo/Dockerfile").read_text()
        self.assertIn("mkdir -p /opt/chapchoo /workspace", image)
        self.assertIn("chown 10001:10001 /opt/chapchoo /workspace", image)
        self.assertTrue((ROOT / "typescript/apps/e2e/src/computers.spec.ts").is_file())
        self.assertTrue(
            (ROOT / "typescript/apps/integration/src/docker.test.ts").is_file()
        )

    def test_independent_owners_and_architectures_have_narrow_dependencies(
        self,
    ) -> None:
        ci = (ROOT / ".github/workflows/ci.yaml").read_text()
        jobs = dict(
            re.findall(
                r"^  ([\w-]+):\n(.*?)(?=^  [\w-]+:\n|\Z)", ci, re.MULTILINE | re.DOTALL
            )
        )
        for owner in ["repository-checks", "frontend-quality", "frontend-tests"]:
            self.assertNotIn("needs:", jobs[owner])
        for architecture, runner, machine, node in [
            ("x86", "ubuntu-24.04", "x86_64", "x64"),
            ("arm", "ubuntu-24.04-arm", "aarch64", "arm64"),
        ]:
            package = "native-build" if architecture == "x86" else "native-build-arm"
            self.assertIn(
                f"needs: [web-{architecture}, binary-{architecture}]", jobs[package]
            )
            for producer, filename in [
                ("web", "build-web"),
                ("binary", "build-binary"),
            ]:
                job = jobs[f"{producer}-{architecture}"]
                self.assertNotIn("needs:", job)
                self.assertIn(f"uses: ./.github/workflows/{filename}.yaml", job)
                self.assertIn(f"runner: {runner}\n", job)
                self.assertIn(f"architecture: {machine}\n", job)
            self.assertIn(f"architecture: {machine}\n", jobs[package])
            self.assertIn(f"node-architecture: {node}\n", jobs[package])
        self.assertIn("needs: binary-x86\n", jobs["docker-integration"])
        self.assertIn("needs: native-build\n", jobs["browser-e2e"])
        self.assertIn("if: always()", jobs["validate"])
        self.assertIn(
            "needs: [repository-checks, frontend-quality, frontend-tests]",
            jobs["validate"],
        )
        self.assertIn("toJSON(needs.*.result)", jobs["validate"])
        # Direct unit tests import the built RPC exports before Storybook runs.
        self.assertLess(
            jobs["frontend-tests"].index(
                "turbo run build --filter=@chapchoo/rpc-client"
            ),
            jobs["frontend-tests"].index("pnpm --filter @chapchoo/testenv test"),
        )

    def test_aggregate_gate_rejects_every_non_success_result(self) -> None:
        ci = (ROOT / ".github/workflows/ci.yaml").read_text()
        gate = ci.split("  validate:", 1)[1].split("  browser-e2e:", 1)[0]
        script = gate.split("python3 - <<'PY'\n", 1)[1].split("          PY", 1)[0]
        for results, expected in [
            (["success"] * 3, 0),
            (["failure", "success", "success"], 1),
            (["success", "cancelled", "success"], 1),
            (["success", "success", "skipped"], 1),
            ([], 1),
        ]:
            with self.subTest(results=results):
                result = subprocess.run(
                    [
                        "python3",
                        "-c",
                        "\n".join(line[10:] for line in script.splitlines()),
                    ],
                    env={**os.environ, "RESULTS": json.dumps(results)},
                    check=False,
                )
                self.assertEqual(result.returncode, expected)


if __name__ == "__main__":
    unittest.main()
