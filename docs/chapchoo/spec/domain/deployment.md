---
title: "Helm and Argo deployment boundaries"
---

# Helm and Argo deployment boundaries

The public chart composes workloads and explicit prerequisites; offline rendering does not prove a live rollout or substrate cleanup.

## Chart and controller lifecycle

`infra/charts/chapchoo` configures a combined Client/Admin/Control union or
independently scalable group deployments, independent webs and private ClusterIP
RPC Services. General/Admin service ports 50051/50052 target the selected named
gRPC listener; Control has its own Service. Network peer 50053 is a separate
internal-only listener/Service, never a public Ingress. One external PostgreSQL
authority and preexisting Secret references are required.
The chart does not create database/key Secrets or provision a database. Web
configuration contains selected listener/public/backend settings and optional
peer public web navigation metadata. Public URL
paths directly determine Ingress and health paths; images contain no selected
deployment path. Artifact execution, not a declaration, verifies portability.

A single fixed-name prepare Job uses both native Helm pre-install/pre-upgrade
weight-10 and Argo PreSync wave-10 annotations. It depends only on preexisting
DB/key Secrets, not later ConfigMaps, ServiceAccounts or same-app Sync Secrets.
It has a finite deadline, no retries/restarts or service-account token, and
before-creation/success cleanup; failure evidence is retained. Applications do
not run migrations. Termination grace is at least30s. Argo acceptance requires a
full application sync: selective resource sync does not execute hooks.

Optional ingress-nginx external RPC edges require explicit TLS/HTTP2 settings
and distinct General/Admin hosts. Internal plaintext is a trusted network hop,
not end-to-end TLS. Public web URLs require HTTPS. A rendered Ingress is not
proof of certificate, metadata/trailer or product behavior. Image rollback does
not roll back database schema or committed data; arbitrary zero downtime is not
promised. Examples under `testenv/chapchoo` do not authorize a cluster write.
The chart rejects a gRPC catch-all sharing the same ingress class, host and root
path with an enabled web route. Different web prefixes remain valid, including
General `/` and Admin `/admin` on one host.

### Computer component deployment

The peer ingress NetworkPolicy permits only this release's backend server
workloads on 50053; actual CNI denial/enforcement is a live release gate, not a
rendering result. Component/public gRPC remains a separate authenticated boundary.
A scoped existing system-owner Secret is provisioned only to Control and the
selected Computer Provider, not webs, separate Client/Admin or execution Pods.

The optional Computer Provider is independent of Control replicas, defaults to
one replica/Recreate, and uses the binary-only image with explicit runtime image,
resource/storage and namespace policy. Its managed namespace must already exist
and be explicitly selected. Namespaced Role/Binding grants Pod get/list/watch/
create/delete and PVC/Secret get/create/delete, with a distinct token-free
execution ServiceAccount. No product DB/root environment is provisioned to this
process. Namespace Secret get still grants visibility to all Secrets in that
namespace; operators must isolate managed resources from backend credentials.
Default replica count is not quiescence proof. Bootstrap credentials are created
through the actual Provider's acknowledged resource flow, not Helm templates.

See [Computer Control RPC](computer-control-rpc.md),
[Kubernetes boundaries](computer-kubernetes.md) and
[owner/operator web](computer-web.md). Live resource retirement, original PVC
contents, policy enforcement and component TLS require approved disposable
prerequisites and are not established by offline/synthetic tests.

## Deployment secret clarification

`CHAPCHOO_SECRET_KEY` and chart `secretKey` replace the prior configuration name
without a fallback alias. Preserve the existing key bytes; crypto derivation and
DB binding are unchanged. Home's PASSWORD is the sole external DB secret input;
its ExternalSecret template creates both password and escaped postgres-url.

The default chart also owns first-admin proof bootstrap (Design37/M44), distinct
from schema preparation. Hook SA/Role/Binding at-30 authorize a finite Job at-20;
prepare remains-10. Bootstrap uses only namespace/name/field and a projected API
token, never the DB/root key. It gets the one named Secret, creates if absent,
validates/reuses existing or concurrently created material and never overwrites.
RBAC grants named get and namespace Secret create, not list/update/patch/delete;
create cannot be restricted to one resourceName by Kubernetes RBAC. Backend/web
pods and schema prepare retain no token; the Computer Provider uses its own
namespaced authority described above. Secret values do not appear in rendered
manifests/logs and survive hook/release deletion. Explicit existing setup Secret
configuration skips bootstrap. The backend image includes a pinned official
kubectl1.35 client; compatible Kubernetes server version is required.

The setup proof is not an Infisical prerequisite in Home. Authorized operators
retrieve it privately after bootstrap for the first Admin setup; DB consumption
still prevents reinitialization. Live bootstrap/API authorization and deployment
remain separately unexecuted until publication and activation approval.

## Connections

- [Linux artifacts and image assembly](account-delivery.md) — Build owners produce deployment-neutral inputs; packaging validates and assembles those exact artifacts without rebuilding them.
- [Foreground process ownership and health](runtime-process.md) — The composition root owns admission, startup and shutdown of its selected backend and web processes. A successful cleanup submission is not completed shutdown.
- [Snapshot publication and downstream handoff](snapshot-publication.md) — A Snapshot publication binds architecture artifacts, image digests and source/run identity. Downstream deployment owns its own activation and data prerequisites.
- [Control admission, streams and calls](computer-control-rpc.md) — Control separates public component authority from trusted peer delivery. Streams and exact-call correlation do not form a durable transport log.
- [Kubernetes resource and retirement contracts](computer-kubernetes.md) — Supported Kubernetes SDK operations validate original resource identities and ordered terminal evidence, without equating a delete response with retirement.
- [Verification layers and release evidence](acceptance.md) — Each test layer establishes a specific owned contract. Synthetic SDK bytes, real Engine acceptance, browser journeys and live cluster rollout are different evidence.

## References

- [`infra/charts/chapchoo`](../../../../infra/charts/chapchoo)
- [`testenv/chapchoo/argocd-example.yaml`](../../../../testenv/chapchoo/argocd-example.yaml)
- [`scripts/check-chart.py`](../../../../scripts/check-chart.py)
