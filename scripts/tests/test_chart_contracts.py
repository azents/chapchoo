"""Verify offline chart rendering, role boundaries and preparation prerequisites."""

from __future__ import annotations

import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parents[2]
CHART = ROOT / "infra/charts/chapchoo"
FIXTURE = ROOT / "testenv/chapchoo/chart-values.yaml"
Manifest = dict[str, Any]


class ChartContractsTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        helm = shutil.which("helm")
        if helm is None:
            raise RuntimeError("Helm is required; chart contracts must not be skipped")
        cls.helm = helm

    def template(
        self,
        overrides: dict[str, Any] | None = None,
        *,
        fixture: bool = True,
        upgrade: bool = False,
    ) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory(prefix="chapchoo-chart-") as temporary:
            command = [
                self.helm,
                "template",
                "contract",
                str(CHART),
                "--namespace",
                "contract",
                "--kube-version",
                "1.30.0",
            ]
            if fixture:
                command.extend(["-f", str(FIXTURE)])
            if upgrade:
                command.append("--is-upgrade")
            if overrides is not None:
                path = Path(temporary) / "values.json"
                path.write_text(json.dumps(overrides), encoding="utf-8")
                command.extend(["-f", str(path)])
            return subprocess.run(
                command, capture_output=True, text=True, check=False, timeout=30
            )

    def render(
        self,
        overrides: dict[str, Any] | None = None,
        *,
        upgrade: bool = False,
    ) -> list[Manifest]:
        result = self.template(overrides, upgrade=upgrade)
        self.assertEqual(result.returncode, 0, result.stderr)
        documents = list(yaml.safe_load_all(result.stdout))
        self.assertTrue(all(isinstance(doc, dict) for doc in documents))
        return documents

    @staticmethod
    def kind(documents: list[Manifest], kind: str) -> list[Manifest]:
        return [document for document in documents if document["kind"] == kind]

    @staticmethod
    def component(documents: list[Manifest], component: str) -> Manifest:
        return next(
            document
            for document in documents
            if document["kind"] == "Deployment"
            and document["metadata"]["labels"]["app.kubernetes.io/component"]
            == component
        )

    @staticmethod
    def environment(container: Manifest) -> dict[str, Manifest]:
        return {entry["name"]: entry for entry in container["env"]}

    def test_one_fixed_job_has_both_controller_hooks_on_install_and_upgrade(
        self,
    ) -> None:
        installs = self.render()
        upgrades = self.render(upgrade=True)
        self.assertEqual(len(self.kind(installs, "Job")), 1)
        self.assertEqual(len(self.kind(upgrades, "Job")), 1)
        job = self.kind(installs, "Job")[0]
        self.assertEqual(
            job["metadata"]["name"], self.kind(upgrades, "Job")[0]["metadata"]["name"]
        )
        self.assertNotIn("generateName", job["metadata"])
        self.assertEqual(
            job["metadata"]["annotations"],
            {
                "helm.sh/hook": "pre-install,pre-upgrade",
                "helm.sh/hook-weight": "-10",
                "helm.sh/hook-delete-policy": "before-hook-creation,hook-succeeded",
                "argocd.argoproj.io/hook": "PreSync",
                "argocd.argoproj.io/sync-wave": "-10",
                "argocd.argoproj.io/hook-delete-policy": "BeforeHookCreation,HookSucceeded",
            },
        )
        spec = job["spec"]
        self.assertEqual(spec["backoffLimit"], 0)
        self.assertGreaterEqual(spec["activeDeadlineSeconds"], 30)
        self.assertLessEqual(spec["activeDeadlineSeconds"], 3600)
        self.assertNotIn("ttlSecondsAfterFinished", spec)
        pod = spec["template"]["spec"]
        self.assertEqual(pod["restartPolicy"], "Never")
        self.assertFalse(pod["automountServiceAccountToken"])
        self.assertNotIn("serviceAccountName", pod)
        self.assertNotIn("volumes", pod)
        container = pod["containers"][0]
        self.assertEqual(container["args"], ["prepare"])
        self.assertNotIn("envFrom", container)
        env = self.environment(container)
        self.assertEqual(set(env), {"CHAPCHOO_POSTGRES_URL", "CHAPCHOO_SECRET_KEY"})
        self.assertEqual(
            env["CHAPCHOO_POSTGRES_URL"]["valueFrom"]["secretKeyRef"],
            {"name": "fixture-postgres", "key": "postgres-url", "optional": False},
        )
        self.assertEqual(
            env["CHAPCHOO_SECRET_KEY"]["valueFrom"]["secretKeyRef"],
            {
                "name": "fixture-secret-key",
                "key": "secret-key",
                "optional": False,
            },
        )

    def test_chart_never_provisions_credentials_or_later_hook_dependencies(
        self,
    ) -> None:
        documents = self.render()
        self.assertEqual(
            {document["kind"] for document in documents},
            {"Job", "Deployment", "Service", "Ingress"},
        )
        for path in (CHART / "templates").iterdir():
            self.assertNotIn("lookup ", path.read_text(encoding="utf-8"))
        for document in self.kind(documents, "Deployment") + self.kind(
            documents, "Job"
        ):
            pod = document["spec"]["template"]["spec"]
            self.assertFalse(pod["automountServiceAccountToken"])
            self.assertGreaterEqual(pod["terminationGracePeriodSeconds"], 30)
            self.assertNotIn("hostNetwork", pod)
            for container in pod["containers"]:
                self.assertNotIn("hostPort", json.dumps(container))
                self.assertTrue(container["securityContext"]["runAsNonRoot"])
                self.assertEqual(container["securityContext"]["runAsUser"], 10001)
                self.assertFalse(
                    container["securityContext"]["allowPrivilegeEscalation"]
                )

    def test_combined_and_separate_backends_share_data_and_distinct_private_services(
        self,
    ) -> None:
        for topology, roles in (
            ("combined", ["both"]),
            ("separate", ["general", "admin"]),
        ):
            with self.subTest(topology=topology):
                documents = self.render({"backend": {"topology": topology}})
                for role in roles:
                    deployment = self.component(documents, f"backend-{role}")
                    container = deployment["spec"]["template"]["spec"]["containers"][0]
                    self.assertEqual(container["args"], ["serve", "--role", role])
                    env = self.environment(container)
                    self.assertEqual(
                        env["CHAPCHOO_POSTGRES_URL"]["valueFrom"]["secretKeyRef"][
                            "name"
                        ],
                        "fixture-postgres",
                    )
                    self.assertEqual(
                        env["CHAPCHOO_SECRET_KEY"]["valueFrom"]["secretKeyRef"]["name"],
                        "fixture-secret-key",
                    )
                    self.assertNotIn("CHAPCHOO_SQLITE_PATH", env)
                    self.assertEqual(
                        container["livenessProbe"]["grpc"]["service"],
                        "chapchoo.liveness",
                    )
                    self.assertEqual(
                        container["readinessProbe"]["grpc"]["service"],
                        "chapchoo.accounts.Admin"
                        if role == "admin"
                        else "chapchoo.accounts.General",
                    )
                    self.assertGreater(container["readinessProbe"]["timeoutSeconds"], 5)
                    self.assertGreaterEqual(
                        container["startupProbe"]["periodSeconds"]
                        * container["startupProbe"]["failureThreshold"],
                        60,
                    )
                services = self.kind(documents, "Service")
                for role, port in (("general", 50051), ("admin", 50052)):
                    service = next(
                        entry
                        for entry in services
                        if entry["metadata"]["name"].endswith(f"-{role}-grpc")
                    )
                    self.assertEqual(service["spec"]["type"], "ClusterIP")
                    self.assertEqual(service["spec"]["ports"][0]["port"], port)
                    self.assertEqual(
                        service["spec"]["selector"]["app.kubernetes.io/component"],
                        "backend-both" if topology == "combined" else f"backend-{role}",
                    )
                self.assertTrue(
                    all(service["spec"]["type"] == "ClusterIP" for service in services)
                )

    def test_web_units_have_only_selected_listener_origin_and_rpc_configuration(
        self,
    ) -> None:
        documents = self.render()
        for role, port, prefix in (("general", 3003, ""), ("admin", 3004, "/admin")):
            deployment = self.component(documents, f"{role}-web")
            container = deployment["spec"]["template"]["spec"]["containers"][0]
            self.assertEqual(
                container["args"], ["serve", "--role", "none", "--web", role]
            )
            self.assertEqual(
                set(self.environment(container)),
                {
                    f"CHAPCHOO_{role.upper()}_WEB_URL",
                    f"CHAPCHOO_{role.upper()}_WEB_LISTEN",
                    f"CHAPCHOO_{role.upper()}_BACKEND_URL",
                },
            )
            for entry in container["env"]:
                self.assertNotIn("valueFrom", entry)
            self.assertEqual(
                self.environment(container)[f"CHAPCHOO_{role.upper()}_WEB_LISTEN"][
                    "value"
                ],
                f"0.0.0.0:{port}",
            )
            for probe in ("startupProbe", "livenessProbe", "readinessProbe"):
                self.assertEqual(
                    container[probe]["httpGet"],
                    {"path": f"{prefix}/health", "port": port, "scheme": "HTTP"},
                )
            self.assertNotIn("CHAPCHOO_COMPILED_BASE_PATH", self.environment(container))

    def test_each_web_can_run_independently_without_backend_credentials_or_job(
        self,
    ) -> None:
        for role in ("general", "admin"):
            other = "admin" if role == "general" else "general"
            documents = self.render(
                {
                    "backend": {"enabled": False},
                    "database": {"existingSecret": ""},
                    "secretKey": {"existingSecret": ""},
                    role: {"backendUrl": f"https://{role}-rpc.example.invalid"},
                    other: {"enabled": False},
                    "grpcIngress": {"enabled": False},
                }
            )
            self.assertEqual(len(self.kind(documents, "Deployment")), 1)
            self.assertEqual(self.kind(documents, "Job"), [])
            self.assertEqual(len(self.kind(documents, "Service")), 1)
            self.assertNotIn("secretKeyRef", json.dumps(documents))
            self.assertEqual(
                self.environment(
                    self.component(documents, f"{role}-web")["spec"]["template"][
                        "spec"
                    ]["containers"][0]
                )[f"CHAPCHOO_{role.upper()}_BACKEND_URL"]["value"],
                f"https://{role}-rpc.example.invalid",
            )

    def test_tls_edges_preserve_distinct_routes_without_plaintext_public_services(
        self,
    ) -> None:
        documents = self.render()
        grpc_hosts = set()
        for ingress in self.kind(documents, "Ingress"):
            annotations = ingress["metadata"]["annotations"]
            self.assertEqual(
                annotations["nginx.ingress.kubernetes.io/ssl-redirect"], "true"
            )
            self.assertEqual(
                annotations["nginx.ingress.kubernetes.io/force-ssl-redirect"], "true"
            )
            self.assertNotIn("nginx.ingress.kubernetes.io/rewrite-target", annotations)
            spec = ingress["spec"]
            self.assertTrue(spec["ingressClassName"])
            self.assertTrue(spec["tls"][0]["secretName"])
            self.assertEqual(spec["tls"][0]["hosts"][0], spec["rules"][0]["host"])
            if ingress["metadata"]["name"].endswith("-grpc"):
                self.assertEqual(
                    annotations["nginx.ingress.kubernetes.io/backend-protocol"], "GRPC"
                )
                grpc_hosts.add(spec["rules"][0]["host"])
                self.assertEqual(spec["rules"][0]["http"]["paths"][0]["path"], "/")
        self.assertEqual(
            grpc_hosts, {"rpc.example.invalid", "admin-rpc.example.invalid"}
        )
        private_admin = self.render({"grpcIngress": {"admin": {"enabled": False}}})
        self.assertFalse(
            any(
                ingress["metadata"]["name"].endswith("-admin-grpc")
                for ingress in self.kind(private_admin, "Ingress")
            )
        )

    def test_generated_setup_proof_is_separate_ordered_and_narrowly_authorized(
        self,
    ) -> None:
        overrides = {"setupProof": {"existingSecret": ""}}
        documents = self.render(overrides)
        repeated = self.render(overrides)
        self.assertEqual(documents, repeated)
        self.assertEqual(self.kind(documents, "Secret"), [])
        jobs = self.kind(documents, "Job")
        self.assertEqual(len(jobs), 2)
        bootstrap = next(
            j for j in jobs if j["metadata"]["name"].endswith("-setup-proof")
        )
        prepare = next(j for j in jobs if j["metadata"]["name"].endswith("-prepare"))
        self.assertEqual(
            bootstrap["metadata"]["annotations"]["helm.sh/hook-weight"], "-20"
        )
        self.assertEqual(
            bootstrap["metadata"]["annotations"]["argocd.argoproj.io/sync-wave"], "-20"
        )
        self.assertEqual(
            prepare["metadata"]["annotations"]["helm.sh/hook-weight"], "-10"
        )
        pod = bootstrap["spec"]["template"]["spec"]
        self.assertTrue(pod["automountServiceAccountToken"])
        self.assertFalse(
            prepare["spec"]["template"]["spec"]["automountServiceAccountToken"]
        )
        self.assertEqual(bootstrap["spec"]["backoffLimit"], 0)
        env = self.environment(pod["containers"][0])
        self.assertEqual(
            set(env), {"POD_NAMESPACE", "SETUP_SECRET_NAME", "SETUP_SECRET_KEY"}
        )
        self.assertNotIn("secretKeyRef", json.dumps(env))
        name = bootstrap["metadata"]["name"]
        self.assertEqual(pod["serviceAccountName"], name)
        (role,) = self.kind(documents, "Role")
        self.assertEqual(
            role["rules"],
            [
                {
                    "apiGroups": [""],
                    "resources": ["secrets"],
                    "resourceNames": [name],
                    "verbs": ["get"],
                },
                {"apiGroups": [""], "resources": ["secrets"], "verbs": ["create"]},
            ],
        )
        for kind in ["Role", "RoleBinding", "ServiceAccount"]:
            (item,) = self.kind(documents, kind)
            self.assertEqual(
                item["metadata"]["annotations"]["helm.sh/hook-weight"], "-30"
            )
        backend = self.component(documents, "backend-both")
        backend_env = self.environment(
            backend["spec"]["template"]["spec"]["containers"][0]
        )
        self.assertEqual(
            backend_env["CHAPCHOO_SETUP_PROOF"]["valueFrom"]["secretKeyRef"]["name"],
            name,
        )
        self.assertNotIn(
            "CHAPCHOO_SETUP_PROOF",
            self.environment(prepare["spec"]["template"]["spec"]["containers"][0]),
        )
        explicit = self.render()
        self.assertEqual(self.kind(explicit, "Role"), [])
        self.assertEqual(len(self.kind(explicit, "Job")), 1)

    def test_mail_credentials_are_backend_only_and_not_preparation_prerequisites(
        self,
    ) -> None:
        documents = self.render(
            {
                "mail": {
                    "enabled": True,
                    "sender": "mail@example.invalid",
                    "region": "us-east-1",
                    "existingCredentialsSecret": "fixture-mail",
                }
            }
        )
        backend = self.component(documents, "backend-both")
        env = self.environment(backend["spec"]["template"]["spec"]["containers"][0])
        self.assertEqual(env["CHAPCHOO_SES_REGION"]["value"], "us-east-1")
        self.assertEqual(
            env["AWS_SECRET_ACCESS_KEY"]["valueFrom"]["secretKeyRef"]["name"],
            "fixture-mail",
        )
        for deployment in self.kind(documents, "Deployment"):
            if deployment != backend:
                self.assertNotIn("fixture-mail", json.dumps(deployment))
        self.assertNotIn("fixture-mail", json.dumps(self.kind(documents, "Job")))

    def test_grpc_and_root_web_collisions_fail_without_rejecting_distinct_routes(
        self,
    ) -> None:
        for role in ("general", "admin"):
            result = self.template(
                {"grpcIngress": {role: {"host": "chapchoo.example.invalid"}}}
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("gRPC catch-all and a root web Ingress", result.stderr)
        # The fixture's root General and /admin web routes share a host safely.
        self.render()
        # A non-root web prefix remains more specific than the gRPC catch-all.
        self.render(
            {
                "general": {
                    "publicUrl": "https://chapchoo.example.invalid/app",
                },
                "grpcIngress": {"general": {"host": "chapchoo.example.invalid"}},
            }
        )
        # Distinct controllers do not produce duplicate ingress-nginx locations.
        self.render(
            {
                "grpcIngress": {
                    "className": "other-nginx",
                    "general": {"host": "chapchoo.example.invalid"},
                }
            }
        )
        self.render(
            {
                "general": {"enabled": False},
                "grpcIngress": {"general": {"host": "chapchoo.example.invalid"}},
            }
        )

    def test_backend_only_keeps_edges_off_and_uses_the_same_pinned_prepare_image(
        self,
    ) -> None:
        digest = "sha256:" + "a" * 64
        documents = self.render(
            {
                "general": {"enabled": False},
                "admin": {"enabled": False},
                "backend": {"image": {"digest": digest}},
                "webIngress": {"enabled": False},
                "grpcIngress": {"enabled": False},
                "imagePullSecrets": [{"name": "fixture-pull-secret"}],
            }
        )
        self.assertEqual(len(self.kind(documents, "Job")), 1)
        self.assertEqual(len(self.kind(documents, "Deployment")), 1)
        self.assertEqual(len(self.kind(documents, "Service")), 2)
        self.assertEqual(self.kind(documents, "Ingress"), [])
        for document in self.kind(documents, "Job") + self.kind(
            documents, "Deployment"
        ):
            pod = document["spec"]["template"]["spec"]
            self.assertEqual(pod["imagePullSecrets"], [{"name": "fixture-pull-secret"}])
            self.assertEqual(
                pod["containers"][0]["image"],
                f"registry.example.invalid/chapchoo/backend@{digest}",
            )

    def test_missing_or_unsafe_configuration_fails_rendering(self) -> None:
        default = self.template(fixture=False)
        self.assertNotEqual(default.returncode, 0)
        invalid = [
            {"setupProof": {"existingSecret": "", "key": "different.field"}},
            {"prerequisitesConfirmed": False},
            {"database": {"existingSecret": ""}},
            {"secretKey": {"existingSecret": ""}},
            {"backend": {"image": {"repository": ""}}},
            {"backend": {"topology": "unknown"}},
            {"backend": {"service": {"type": "LoadBalancer"}}},
            {"terminationGracePeriodSeconds": 25},
            {"prepare": {"activeDeadlineSeconds": 0}},
            {"prepare": {"activeDeadlineSeconds": 4000}},
            {"general": {"publicUrl": "http://localhost:3003"}},
            {"admin": {"publicUrl": "http://admin.example.invalid"}},
            {"general": {"publicUrl": "https://u:p@example.invalid"}},
            {"general": {"publicUrl": "https://example.invalid/?untrusted=1"}},
            {"general": {"publicUrl": "https://example.invalid/#untrusted"}},
            {"admin": {"publicUrl": "https://example.invalid//admin"}},
            {"admin": {"publicUrl": "https://example.invalid/%61dmin"}},
            {
                "admin": {
                    "publicUrl": "https://example.invalid/admin/..",
                }
            },
            {"admin": {"backendUrl": "https://rpc.example.invalid/rewritten"}},
            {"admin": {"backendUrl": "https://u:p@rpc.example.invalid"}},
            {"admin": {"backendUrl": "https://rpc.example.invalid/%2f"}},
            {"backend": {"enabled": False}, "grpcIngress": {"enabled": False}},
            {"grpcIngress": {"general": {"existingTlsSecret": ""}}},
            {"grpcIngress": {"admin": {"host": "rpc.example.invalid"}}},
            {"grpcIngress": {"className": ""}},
            {"grpcIngress": {"general": {"host": "127.0.0.1"}}},
            {"grpcIngress": {"general": {"host": "bad..example.invalid"}}},
            {"grpcIngress": {"controller": "unknown-controller"}},
            {"webIngress": {"existingTlsSecret": ""}},
            {"general": {"publicUrl": "https://chapchoo.example.invalid:444"}},
            {
                "admin": {
                    "publicUrl": "https://chapchoo.example.invalid:443",
                }
            },
            {
                "backend": {"enabled": False},
                "general": {"enabled": False},
                "admin": {"enabled": False},
                "webIngress": {"enabled": False},
                "grpcIngress": {"enabled": False},
            },
            {"mail": {"enabled": True}},
        ]
        for overrides in invalid:
            with self.subTest(overrides=overrides):
                result = self.template(overrides)
                self.assertNotEqual(
                    result.returncode, 0, "Unsafe settings unexpectedly rendered"
                )

    def test_argocd_example_is_a_full_sync_reference_not_a_secret_provider(
        self,
    ) -> None:
        example = yaml.safe_load(
            (ROOT / "testenv/chapchoo/argocd-example.yaml").read_text(encoding="utf-8")
        )
        self.assertEqual(example["kind"], "Application")
        self.assertEqual(example["spec"]["source"]["path"], "infra/charts/chapchoo")
        self.assertNotIn("syncPolicy", example["spec"])
        self.assertEqual(
            example["spec"]["source"]["helm"]["valuesObject"]["database"][
                "existingSecret"
            ],
            "prereq-postgres",
        )
        self.assertNotIn(
            "password", example["spec"]["source"]["helm"]["valuesObject"]["database"]
        )


if __name__ == "__main__":
    unittest.main()
