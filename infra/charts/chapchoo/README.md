# Chapchoo Helm chart

This chart packages account and Agent/Computer services, two independent web
surfaces, Control and an optional Computer Provider. It provisions no database,
namespace, fixed server key, certificates, secret provider or ingress controller.
A dedicated hook generates only the first-admin setup proof when no existing
Secret is selected. Rendering is not live application or security evidence.

## Prerequisites and explicit values

Use Kubernetes 1.27 or later for native gRPC probes and a compatible Helm release.
The offline suite uses Helm 4.2.4. Before deployment, provide:

1. External PostgreSQL and an already existing URL Secret selected by
   `database.existingSecret` / `database.urlKey`. All backend groups and the
   preparation Job share that product authority.
2. The original fixed installation key through `secretKey.existingSecret` /
   `secretKey.key`. Preserve the same bytes across upgrades and rollback. The
   chart never generates, rotates or rebinds that key.
3. Immutable architecture-compatible backend/general-web/admin-web images,
   each with `repository` and `tag` or `digest`. Digest takes precedence.
4. Explicit HTTPS public URLs. Their paths are runtime configuration, not build
   variants. Initial setup proof is separate from schema preparation.
5. Trusted internal routing, suitable CNI enforcement and operator-owned database,
   component/public traffic and encryption policy. The chart's peer NetworkPolicy
   is useful only when actually enforced. It installs no mesh or native backend TLS.
6. Referenced pull, TLS, owner-token and optional mail Secrets before workloads
   need them. Secret values do not belong in values files or chat.

`prerequisitesConfirmed: true` is an operator acknowledgement, not a cluster lookup.
Defaults fail rather than inventing credentials, image tags or public origins.
For Argo CD, same-application Sync Secrets/ExternalSecrets cannot satisfy PreSync
prerequisites; provision them independently before full application sync.

## Units and topology

| Selection                    | Result                                                                                                                               |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `backend.topology: combined` | One `serve --groups client,admin,control` Deployment                                                                                 |
| `backend.topology: separate` | Independent Client, Admin and Control Deployments                                                                                    |
| `general.enabled`            | Independent `serve --groups --web general` Deployment                                                                                |
| `admin.enabled`              | Independent `serve --groups --web admin` Deployment                                                                                  |
| `computerProvider.enabled`   | Independent `computer-provider --in-cluster` Deployment                                                                              |
| `backend.enabled: false`     | No local backend/preparation Job; selected webs need explicit `backendUrl`, and a selected Provider needs explicit `controlEndpoint` |

Both backend topologies use PostgreSQL and the original fixed key; no SQLite
replication/conversion is supplied. `backend.replicas` scales combined mode;
`backend.groupReplicas.client/admin/control` independently scale separate groups.
General and Admin ClusterIP Services expose 50051 and 50052 respectively, both
targeting the selected pod's named `grpc` port 50051. A Control ClusterIP Service
exposes 50051. Combined mode selects the same service union; separate mode selects
the appropriate group. No backend hostPort, NodePort or LoadBalancer is created.

Web Services use 3003/3004, explicitly binding `0.0.0.0` rather than loopback CLI
defaults. Web pods receive only their selected public URL/listener/backend URL;
no product DB, root/setup/owner token or mail credential. Each web target is
independent even when operators deliberately choose equal URLs. Backend images
require no Node or web assets. Enabled backends still need `general.publicUrl`
for issued links even when the general web is not local.

Optional SMTP uses backend-only `mail.sender/host/port/tls`; TLS is `starttls` or
`implicit`, always certificate-verified. `mail.existingCredentialsSecret` and
username/password keys optionally select credentials. Empty credentials permit an
unauthenticated TLS relay, not plaintext SMTP. No mail provider/Secret is created.

## Computer components and peer isolation

Every Control pod binds peer 50053 separately from component/public gRPC. The
peer Service is internal-only, with no Ingress. The default advertise address is
`http://$(POD_IP):50053` from the Downward API, not a load-balanced ownership
address or Kubernetes discovery. IPv6 operators must provide a compatible bind
configuration and bracketed advertised address. The peer NetworkPolicy permits
only this release's backend server workloads on that port; public/component 50051
remains a separate authenticated boundary. An allowed compromised server remains
inside the explicit trusted-network boundary. Rendering alone does not prove CNI
enforcement, denied untrusted traffic or end-to-end TLS.

`systemOwner.id`, `providerIds`, `existingSecret` and `key` select deployment-
provisioned owner authority. Generate 32 random bytes as unpadded URL-safe Base64,
for example `python3 -c 'import secrets; print(secrets.token_urlsafe(32))'`.
Privately enter that result in the deployment's secret manager and materialize
the referenced Secret before startup. Never put the value in arguments, values,
logs or chat. It grants metadata/key management for exactly the listed Provider
IDs. Preserve it until explicit operator rotation/revocation. Only Control and
the designated Computer Provider receive it, not separate Client/Admin pods,
webs, execution Pods or preparation.

Enable `computerProvider` with explicit logical `id`/`name`, process `image`,
`computerImage` and scoped owner configuration. The existing backend image can
run either component command. `managedNamespace` must explicitly select an existing
namespace. Isolate it from backend/owner credential Secrets: namespaced Secret get
permission includes every Secret in that namespace, so sharing a credential
namespace grants that additional visibility. The chart does not infer or create
a safe namespace. The Provider gets only namespaced Pod
get/list/watch/create/delete and PVC/Secret get/create/delete. It receives no
product DB/root key. Its default one replica and Recreate strategy are operational
policy, not authority or quiescence proof.

Execution Pods use a distinct ServiceAccount with API token automount disabled.
Image/pull policy, CPU/memory Requests/Limits, storage class/capacity and saved
mount remain runtime policy. Omitted StorageClass uses the operator's existing
default; the chart provisions no class or storage. Stop/restart preserve the
original dedicated PVC; missing/unverified original storage never authorizes
replacement. Actual Provider bootstrap issues Connector credentials after
resource UID acknowledgements, rather than rendering secrets in Helm.

For external Control, set `computerProvider.controlEndpoint` and configure the
same system-owner scope/credential at that Control independently. Component
TLS/exposure remains operator-owned. A Running Provider pod does not prove an
authenticated connection, execution readiness, PVC preservation or cleanup.

## Public origins, prefixes and TLS edges

Kubernetes public web URLs require HTTPS. Local HTTP belongs only to the
loopback-bound CLI policy. The public origin is explicit, not inferred from
forwarded headers. `general.publicUrl` / `admin.publicUrl` determine Ingress and
`<prefix>/health` paths. Identical image bytes can serve root or canonical prefixes
without rebuilding, rewrites or generated-output patching.

`webIngress.enabled` targets an existing ingress-nginx controller with explicit
class and existing TLS Secret. `grpcIngress.enabled` independently configures
General/Admin TLS hosts, which must differ. Leave Admin routing disabled to keep
that edge unexposed. A gRPC catch-all cannot share the same ingress class/host/root
web path. Distinct web prefixes or controllers remain valid, including General
`/` and Admin `/admin` on one web host.

The adapter uses plaintext internal HTTP/2 (`backend-protocol: GRPC`) with TLS at
the external edge. Verify certificates/hostnames, framing, auth metadata,
trailers/status, cancellation and deadlines in actual acceptance. Another proxy
requires operator-managed routing; arbitrary ingress classes are not assumed
compatible with ingress-nginx annotations. Peer methods are never attached to
these public routes.

## Preparation, setup and controller lifecycles

A local backend gets one fixed-name `<fullname>-prepare` Job using its selected
image's `prepare` command. Helm pre-install/pre-upgrade weight -10 and Argo PreSync
wave -10 share that Job. It references only preexisting DB/key Secrets, not later
ConfigMaps/ServiceAccounts/Sync Secrets. It has a finite deadline, zero retries,
no pod restart or API token, and before-creation/success cleanup. Failed evidence
is retained; a deliberate full deployment retry replaces the old hook Job.

Default setup-proof bootstrap is separate: SA/Role/Binding at -30 and a finite
proof Job at -20 precede preparation at -10. A projected API token authorizes get
for the named Secret and namespace Secret create, not list/update/patch/delete
or cluster authority. Kubernetes cannot restrict create by resourceName. The Job
uses pinned official kubectl 1.35; use a compatible server minor. It generates
only a missing valid 43-character URL-safe proof, preserves a valid existing or
concurrently created proof, and never overwrites invalid material. Values are
not rendered, looked up by Helm or printed. The proof Secret has no hook/release
owner reference and survives deletion. Explicit `setupProof.existingSecret` skips
this bootstrap. Operators retrieve it privately and enter it in first Admin
setup; consumed DB state prevents reopening initialization.

`secretKey.existingSecret/key` configure `CHAPCHOO_SECRET_KEY`; preserve the
original 32-byte Base64 value, including padding. This is neither key replacement
nor a new migration facility. Backend/web/preparation pods mount no API token;
Computer Provider and setup bootstrap have their own narrow authority.

Use **full Argo CD application sync**: selective sync skips hooks. The reference
has no automatic sync/prune, namespace creation, retry loop or
`ApplyOutOfSyncOnly`. Replace every example destination/revision/image before use.
Kustomize patches replace `helm.valuesObject` wholesale; retain complete explicit
prerequisites. A hook does not stop old writers. Coordinate contract/schema
cutover; mixed-old/new compatibility and zero downtime are not promised. A
committed migration survives later failure, and image rollback is not DB rollback.

## Health and shutdown

Liveness uses `chapchoo.liveness`. Readiness selects Client/Admin/Control business
health and checks schema/key/DB without preparation. Combined admission belongs
to the whole selected group union. Web health checks process/config, not backend
auth or a whole product journey. Provider connectivity is independently projected
by Control, not inferred from Kubernetes Deployment readiness.

Startup probes allow 90 seconds for the CLI's 60-second startup budget. Grace is
at least 30 seconds for default 20-second graceful plus 5-second forced/reap
budgets. Increasing application budgets requires corresponding orchestration
changes. Containers use UID/GID 10001, drop capabilities and disable privilege
escalation. Writable temporary/cache behavior needs actual image acceptance.

## Offline validation and live gates

```sh
helm lint --strict infra/charts/chapchoo -f testenv/chapchoo/chart-values.yaml
helm template contract infra/charts/chapchoo -f testenv/chapchoo/chart-values.yaml --namespace contract
uv run --locked --project .codex python scripts/check-chart.py
```

The locked `.codex` environment supplies PyYAML. Offline values contain no actual
credentials and are not deployable. Tests cover hooks, failure retention,
prerequisites, group topology, explicit web targets, health/grace, TLS routing,
peer policy, scoped Provider RBAC and unsafe configuration refusal. They execute
no cluster reads/writes/exec or Helm install/upgrade.

Live Helm/Argo full-sync, Kubernetes retirement/storage/CNI and external TLS gates
remain unexecuted without approved disposable cluster/namespace, images,
exposure/TLS and existing DB/key/data scope. Synthetic SDK/process/browser tests
must not be reported as those live gates.
