# Chapchoo Helm chart

This chart packages the implemented account services and two independent web
surfaces. It does not provision a database, namespace, server key, certificates,
secret provider or ingress controller. A dedicated hook and minimal namespace
RBAC generate only the first-admin setup proof when no existing Secret is selected.
Rendering is
not evidence of a deployed application, valid TLS or successful product journeys.

## Prerequisites and explicit values

Use Kubernetes 1.27 or later for native gRPC probes and a compatible Helm
installation. The offline contract suite was exercised with Helm 4.2.4.

Before a deployment, an operator must provide:

1. A reachable external PostgreSQL database and an **already existing** Secret
   containing its connection URL. All backend roles and the prepare Job use
   `database.existingSecret` / `database.urlKey`.
2. The installation's original fixed key in an **already existing** Secret,
   selected by `secretKey.existingSecret` / `secretKey.key`. Preserve
   this key through upgrades, restarts and image rollback. The chart does not
   generate, replace, rebind or rotate it.
3. Immutable, architecture-compatible backend/general-web/admin-web images.
   Each selected image requires `repository` plus `tag` or a SHA-256 `digest`.
   A digest takes precedence when both are supplied.
4. Explicit HTTPS public URLs and matching image path declarations. The first-admin
   setup proof is generated once by a separate chart hook unless
   `setupProof.existingSecret` explicitly selects an existing Secret; schema
   preparation does not initialize the first administrator.
5. Trusted internal routing and network access for the Job, database, web and
   backend pods. ClusterIP/private addresses alone do not prove a trusted path.
   Apply appropriate operator-owned workload/network controls. The chart does not
   install a NetworkPolicy, mesh or native backend TLS.
6. Any referenced image-pull Secrets, TLS Secrets and optional mail-credential
   Secret before the hook executes. Image-pull Secrets are only referenced.

Set `prerequisitesConfirmed: true` after these prerequisites are available.
This is an operator acknowledgement, **not a cluster lookup or existence check**.
Missing/incorrect Secrets, connectivity, schema or key bindings still fail at
runtime. Default values deliberately fail rendering rather than inventing
credentials, image tags or a public origin.

For Argo CD, an ordinary Secret/ExternalSecret in the same application's Sync
phase cannot satisfy this chart's PreSync dependencies, even with a smaller wave.
Provision prerequisites separately before a full Chapchoo application sync.
No Secret value needs to be placed in chart values or shared in chat.

## Units and topology

| Selection | Result |
| --- | --- |
| `backend.enabled: true`, `backend.topology: combined` | One `serve --role both` Deployment |
| `backend.topology: separate` | Separate `serve --role general` and `serve --role admin` Deployments |
| `general.enabled` | Independent `serve --role none --web general` Deployment |
| `admin.enabled` | Independent `serve --role none --web admin` Deployment |
| `backend.enabled: false` | No local backend or schema Job; every selected web requires its explicit `backendUrl` |

Both backend topologies share the same external PostgreSQL and fixed-key Secret.
No SQLite volume, conversion or replication mechanism is supplied. General and
Admin have distinct, fixed **ClusterIP-only** protocol Services on 50051 and
50052. Combined mode selects one pod with both ports; separate mode selects each
matching role. There are no backend host ports, NodePorts or LoadBalancers.

Web Services use 3003 and 3004. Their commands explicitly bind the corresponding
listener to `0.0.0.0`; loopback CLI defaults would not be reachable from Services.
Web pods receive only their selected public URL, listener and backend URL.
They receive no database, secret-key, setup-proof or mail credentials.
The backend image does not require Node or web assets.

An enabled backend always needs `general.publicUrl`, even if the chart's general
web is disabled: issued links still need the intended general web location.
Optional SES uses backend-only sender/region configuration and an existing
credential Secret. No mail provider resource is created. The separate setup-proof bootstrap has
its own narrowly authorized service account; ordinary backend/web and schema
preparation pods still do not mount API tokens.

## Public origins, prefixes and TLS edges

Every Kubernetes public web URL must use HTTPS. Local HTTP is supported only by
the separate loopback-bound CLI policy; it is not an insecure Kubernetes fallback.
The public origin is configured, not inferred from forwarding headers.

`general.compiledBasePath` / `admin.compiledBasePath` declare the path already
compiled into the corresponding image. Use `""` for root, or a prefix such as
`/admin`. The chart checks that declaration against `publicUrl` and probes
`<prefix>/health`; no prefix rewrite is introduced and no runtime compiled-path
override is injected. **The declaration does not inspect or prove an image's
contents.** Runtime `getConfig()` / health rejection and artifact acceptance must
independently confirm the actual compiled prefix.

`webIngress.enabled` is optional. The supplied edge adapter targets an existing
**ingress-nginx** controller: explicitly select `className` and an existing TLS
Secret covering the configured public web hosts. HTTPS redirects are configured;
paths retain their original prefix.

`grpcIngress.enabled` is separately optional, with the same explicit supported
controller/class requirement. Each enabled route requires its own host and
existing TLS Secret. General and Admin hosts must differ. Leave
`grpcIngress.admin.enabled: false` to keep Admin routing unexposed.
The chart rejects an enabled root web route sharing the gRPC catch-all's host
and ingress class. Web prefixes such as `/app` and `/admin`, or genuinely
different ingress classes, do not create that duplicate location. General `/`
and Admin `/admin` on one web host remain supported.

The gRPC adapter configures `backend-protocol: GRPC` for the **plaintext internal
HTTP/2 hop** and TLS/HTTPS at the edge. It does not pretend to encrypt the backend
hop. Operator acceptance must verify certificate/hostname validation, HTTP/2
framing, authorization metadata, trailers/status, cancellation and deadlines.
The Node client retains TLS validation without an insecure retry fallback.
If a different proxy/controller is required, supply that operator-managed edge
outside this chart; an arbitrary ingress class is not assumed compatible with
the ingress-nginx annotations.

## One preparation Job, two controller lifecycles

With a local backend, exactly one fixed-name `<release>-chapchoo-prepare` Job runs
the backend image's `prepare` command. A fullname override/truncation can change
the release prefix, but the name is stable across install and upgrade.

| Controller | Ordering | Cleanup |
| --- | --- | --- |
| Helm | `pre-install,pre-upgrade`, weight `-10` | `before-hook-creation,hook-succeeded` |
| Argo CD | `PreSync`, wave `-10` | `BeforeHookCreation,HookSucceeded` |

Argo CD uses its native annotations, not a second controller-specific schema
implementation. The Job references DB/key Secrets directly and has no dependency
on later ConfigMaps, ServiceAccounts or application-created Secrets. It has no
service-account token, no pod restart, zero Job retries and a finite deadline.
Only this explicit command prepares schema and checks/binds the original key.
Routine server startup/readiness is read-only.

Successful hook Jobs are deleted; failed Jobs have no failure-delete policy or
TTL. Preserve their logs/status before a deliberate retry: the next full
deployment's before-creation cleanup replaces the prior Job. No automatic
mutation replay or compensating rollback is added.

**Use a full Argo CD application sync. Selective sync skips hooks.** The reference
`testenv/chapchoo/argocd-example.yaml` deliberately has no automatic sync/prune,
namespace creation, retry loop or `ApplyOutOfSyncOnly` setting. Replace every
example destination, revision, image and prerequisite before authorized use.
Kustomize strategic-merge patches replace `helm.valuesObject` wholesale, so
preserve the entire prerequisite/image configuration in an overlay.

The hook gates new resource application; it does not stop already running pods.
This chart promises neither arbitrary zero-downtime schema upgrades nor whole-
release rollback. A successful earlier migration can remain committed after a
later failure. **Image rollback is not database rollback.** Migration compatibility
and any future maintenance window require their own explicit operational plan.

## Health and shutdown

Backend liveness uses `chapchoo.liveness`; readiness uses the selected General or
Admin business health name and checks schema/key/DB without preparation. The
combined deployment's admission flag belongs to its complete selected backend.
Web health checks process/config responsiveness only, not backend auth or an
assembled product journey.

Startup probes allow 90 seconds for the CLI's 60-second startup budget.
`terminationGracePeriodSeconds` defaults to 30 and cannot be less than 30, exceeding
the default 20-second graceful plus 5-second forced cleanup/reap budgets. Raising
application budgets later requires a corresponding orchestration adjustment.
Containers use UID/GID 10001, drop capabilities, disable privilege escalation and
do not automount Kubernetes service-account tokens. Writable Node cache/runtime
requirements still belong to actual image acceptance.

## Offline validation

From the repository root:

```sh
helm lint --strict infra/charts/chapchoo -f testenv/chapchoo/chart-values.yaml
helm template contract infra/charts/chapchoo \
  -f testenv/chapchoo/chart-values.yaml --namespace contract
uv run --locked --project .codex python scripts/check-chart.py
```

The existing `.codex` environment supplies PyYAML through its locked
python-frontmatter dependency; no new production dependency is added.
The fake fixture contains no actual credentials and is **not deployable input**.
Tests verify install/upgrade hooks, failure retention, prerequisites, role/service
separation, web-only credential isolation, health/grace, optional TLS routing and
unsafe/missing-value rejection. They execute no cluster reads, writes, exec or
Helm install/upgrade.

Real Helm install/upgrade and Argo full-sync acceptance remain separate gates:
initial preparation, already-current schema, wrong fixed key, migration failure
gating, application health, product journeys and TLS behavior. They require
approved cluster/namespace/exposure, existing data/key scope and actual images.

## One-time setup proof and server secret naming

Use `secretKey.existingSecret` / `secretKey.key` for `CHAPCHOO_SECRET_KEY`.
The former configuration name is removed; preserve the original32-byte value,
including its Base64 padding. This does not change key derivation, binding,
storage or introduce a replacement/migration facility.

By default the chart creates a persistent `<fullname>-setup-proof` Secret through
a separate bootstrap Job. It never puts random data in rendered YAML or performs
Helm lookup. Existing valid43-character URL-safe32-byte proof is reused; missing
or invalid fields are not overwritten. A concurrent create is accepted only
when rereading a valid existing proof. No proof is printed in logs or arguments.
The Secret has no hook/release owner reference and survives hook/release deletion.
An explicit existing Secret skips this bootstrap and retains its prior behavior.

Both controllers order bootstrap SA/Role/Binding at-30, proof Job at-20, schema
prepare at-10. Bootstrap alone mounts its API token. The Role permits get only
for the named Secret and Secret create within the namespace; Kubernetes RBAC
cannot constrain create with resourceNames. No list/update/patch/delete or
cluster-wide permission is granted. A pinned official kubectl binary lives only
in the backend image for this Job; web images do not carry that API client.
Use a Kubernetes server compatible with the shipped kubectl1.35 minor version
for automatic bootstrap; the inspected Home kubelet1.36 is within one minor.

An authorized operator can retrieve the proof privately from that named Secret
and enter it in initial Admin setup. Do not put it in logs/chat. DB one-time
consumption remains authoritative; a new proof cannot reopen completed setup.
The external DB/key/pull prerequisites remain operator-owned and preexisting.
