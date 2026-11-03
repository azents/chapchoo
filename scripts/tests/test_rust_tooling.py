"""Exercise the repository's inherited Rust lint policy, not Clippy's full matrix."""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

import tomllib

ROOT = Path(__file__).resolve().parents[2]
RUST = ROOT / "rust"


class RustPolicyTest(unittest.TestCase):
    def test_every_actual_member_inherits_workspace_lints(self) -> None:
        workspace = tomllib.loads((RUST / "Cargo.toml").read_text())["workspace"]
        for member in workspace["members"]:
            manifests = list(RUST.glob(f"{member}/Cargo.toml"))
            self.assertTrue(manifests, f"No manifest for member {member}")
            for manifest in manifests:
                with self.subTest(manifest=manifest.relative_to(ROOT)):
                    package = tomllib.loads(manifest.read_text())
                    self.assertIs(package.get("lints", {}).get("workspace"), True)

    def test_actual_policy_rejects_bad_code_and_accepts_valid_counterpart(self) -> None:
        manifest = (RUST / "Cargo.toml").read_text()
        parsed = tomllib.loads(manifest)["workspace"]
        lint_tables = "\n".join(
            re.findall(r"(?ms)^\[workspace\.lints\.[^\]]+\]\n.*?(?=^\[|\Z)", manifest)
        )
        self.assertEqual(
            tomllib.loads(lint_tables)["workspace"]["lints"], parsed["lints"]
        )
        tracing = parsed["dependencies"]["tracing"]
        version = tracing if isinstance(tracing, str) else tracing["version"]
        edition = parsed["package"]["edition"]
        with tempfile.TemporaryDirectory(prefix="chapchoo-rust-policy-") as directory:
            probe = Path(directory)
            source = probe / "probe/src/main.rs"
            source.parent.mkdir(parents=True)
            (probe / "Cargo.toml").write_text(
                '[workspace]\nmembers = ["probe"]\nresolver = "3"\n'
                f"[workspace.dependencies]\ntracing = {{ version = {json.dumps(version)}, "
                'default-features = false, features = ["std"] }\n' + lint_tables
            )
            (probe / "probe/Cargo.toml").write_text(
                '[package]\nname = "chapchoo-rust-policy-probe"\nversion = "0.0.0"\n'
                f"edition = {json.dumps(edition)}\npublish = false\n"
                "[dependencies]\ntracing.workspace = true\n"
                "[lints]\nworkspace = true\n"
            )
            shutil.copyfile(RUST / "rust-toolchain.toml", probe / "rust-toolchain.toml")
            shutil.copyfile(RUST / "clippy.toml", probe / "clippy.toml")
            env = {**os.environ, "CARGO_TARGET_DIR": str(RUST / "target")}

            def run(*args: str) -> subprocess.CompletedProcess[str]:
                return subprocess.run(
                    ["cargo", *args],
                    cwd=probe,
                    env=env,
                    capture_output=True,
                    text=True,
                    timeout=120,
                    check=False,
                )

            source.write_text(
                '#![allow(dead_code, reason = "Compile-only policy probes are never executed")]\n'
                "fn lossy(value: i64) -> u32 { value as u32 }\n"
                "fn forgotten_future() { let _ = std::future::ready(()); }\n"
                'fn leaked_drop() { std::mem::forget(String::from("fixture")); }\n'
                "fn hidden_exit() { std::process::exit(1); }\n"
                'fn ignored_result() { std::fs::metadata("never-executed"); }\n'
                "async fn held_span() {\n"
                '    let span = tracing::info_span!("fixture");\n'
                "    let _entered = span.enter();\n"
                "    std::future::pending::<()>().await;\n"
                "}\n"
                "async fn held_owned_span() {\n"
                '    let _entered = tracing::info_span!("owned-fixture").entered();\n'
                "    std::future::pending::<()>().await;\n"
                "}\nfn main() {}\n"
            )
            locked = run("generate-lockfile", "--offline")
            self.assertEqual(locked.returncode, 0, locked.stderr)
            invalid = run("clippy", "--locked", "--offline", "--message-format=json")
            self.assertNotEqual(invalid.returncode, 0)
            errors: set[str] = set()
            held_span_errors = 0
            for line in invalid.stdout.splitlines():
                item = json.loads(line)
                if item.get("reason") != "compiler-message":
                    continue
                message = item["message"]
                code = message.get("code")
                if message["level"] == "error" and code:
                    errors.add(code["code"])
                    held_span_errors += (
                        code["code"] == "clippy::await_holding_invalid_type"
                    )
            self.assertEqual(held_span_errors, 2, invalid.stdout + invalid.stderr)
            self.assertTrue(
                {
                    "clippy::cast_possible_truncation",
                    "clippy::cast_sign_loss",
                    "clippy::let_underscore_future",
                    "clippy::mem_forget",
                    "clippy::exit",
                    "clippy::await_holding_invalid_type",
                    "unused_must_use",
                }.issubset(errors),
                f"Missing inherited errors: {errors}\n{invalid.stderr}",
            )
            source.write_text(
                '#![allow(dead_code, reason = "Compile-only policy probes are never executed")]\n'
                "use tracing::Instrument;\n"
                "async fn scoped_context() {\n"
                '    async {}.instrument(tracing::info_span!("fixture")).await;\n'
                "}\n"
                "fn main() -> Result<(), std::num::TryFromIntError> {\n"
                "    let value = u32::try_from(7_i64)?;\n"
                "    assert_eq!(value, 7);\n"
                "    Ok(())\n}\n"
            )
            valid = run("clippy", "--locked", "--offline", "--message-format=json")
            self.assertEqual(valid.returncode, 0, valid.stdout + valid.stderr)


if __name__ == "__main__":
    unittest.main()
