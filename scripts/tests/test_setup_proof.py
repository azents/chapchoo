"""Exercise create-once setup proof lifecycle without a live cluster or secrets."""

from __future__ import annotations

import base64
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "infra/charts/chapchoo/files/bootstrap-setup-proof.sh"
FAKE = r"""#!/usr/bin/env python3
import base64,json,os,sys
from pathlib import Path
root=Path(os.environ['FIXTURE'])
args=sys.argv[1:]
with (root/'calls').open('a') as log: log.write(json.dumps(args)+'\n')
state=root/'secret'
if 'get' in args:
    if os.environ.get('DENIED')=='1': sys.exit(1)
    if state.exists():
        if args[-1]=='name': print('secret/fixture-setup-proof')
        else: sys.stdout.write(base64.b64encode(state.read_bytes()).decode())
    sys.exit(0)
if 'create' in args:
    value=Path(next(a.split('=',2)[2] for a in args if a.startswith('--from-file='))).read_bytes()
    if os.environ.get('RACE'):
        state.write_bytes(b'A'*43 if os.environ['RACE']=='valid' else b'invalid')
        sys.exit(1)
    if state.exists(): sys.exit(1)
    state.write_bytes(value)
    sys.exit(0)
sys.exit(9)
"""


class SetupProofTest(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        kubectl = self.root / "kubectl"
        kubectl.write_text(FAKE)
        kubectl.chmod(0o700)
        self.env = {
            **os.environ,
            "PATH": str(self.root) + os.pathsep + os.environ["PATH"],
            "FIXTURE": str(self.root),
            "POD_NAMESPACE": "fixture",
            "SETUP_SECRET_NAME": "fixture-setup-proof",
            "SETUP_SECRET_KEY": "setup-proof",
        }

    def run_script(self, **extra: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["/bin/sh", str(SCRIPT)],
            env={**self.env, **extra},
            text=True,
            capture_output=True,
            check=False,
            timeout=10,
        )

    def test_create_once_then_reuse_without_value_output(self) -> None:
        first = self.run_script()
        self.assertEqual(first.returncode, 0, first.stderr)
        proof = (self.root / "secret").read_text()
        self.assertEqual(len(proof), 43)
        self.assertEqual(len(base64.urlsafe_b64decode(proof + "=")), 32)
        second = self.run_script()
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertEqual((self.root / "secret").read_text(), proof)
        self.assertNotIn(
            proof, first.stdout + first.stderr + second.stdout + second.stderr
        )
        calls = [
            json.loads(line) for line in (self.root / "calls").read_text().splitlines()
        ]
        self.assertEqual(sum("create" in call for call in calls), 1)
        self.assertTrue(
            all(
                "delete" not in call and "patch" not in call and "apply" not in call
                for call in calls
            )
        )
        self.assertFalse(any(proof in arg for call in calls for arg in call))

    def test_existing_invalid_never_replaced(self) -> None:
        for invalid in [b"", b"bad", b"A" * 42 + b"B", b"A" * 42 + b"\n"]:
            (self.root / "secret").write_bytes(invalid)
            result = self.run_script()
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual((self.root / "secret").read_bytes(), invalid)
        calls = (self.root / "calls").read_text()
        self.assertNotIn('"create"', calls)

    def test_concurrent_create_is_verified_not_overwritten(self) -> None:
        valid = self.run_script(RACE="valid")
        self.assertEqual(valid.returncode, 0, valid.stderr)
        self.assertEqual((self.root / "secret").read_bytes(), b"A" * 43)
        (self.root / "secret").unlink()
        invalid = self.run_script(RACE="invalid")
        self.assertNotEqual(invalid.returncode, 0)
        self.assertEqual((self.root / "secret").read_bytes(), b"invalid")

    def test_api_denial_fails_before_create(self) -> None:
        result = self.run_script(DENIED="1")
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.root / "secret").exists())


if __name__ == "__main__":
    unittest.main()
