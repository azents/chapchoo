# Chapchoo Helm chart

This chart packages account and Agent/Computer services, two independent web
surfaces, Control and an explicitly enabled Kubernetes Computer Provider. It
provisions no database, fixed server key, certificates, secret provider or ingress
controller. An enabled Provider owns a separate, retained execution namespace by
default. Dedicated hooks prepare first-admin proof and, only when selected,
protected Provider authentication credentials. Rendering is not live application
or security evidence.

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
6. Referenced pull, TLS and optional mail Secrets before workloads
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
| `kubernetesProvider.enabled`   | Independent `computer-provider --in-cluster` Deployment                                                                              |
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

### Explicit Kubernetes enablement and defaults

Kubernetes Provider support is **off by default**. After the ordinary database,
root-key, backend image and public-URL prerequisites are configured, opt in with:

```yaml
kubernetesProvider:
  enabled: true
```

No owner token or additional mandatory Provider fields are needed. The chart
supplies a stable release-derived Provider ID/name, Provider SA, authentication
audience/permit mapping, execution namespace, token-free Computer SA, runtime
resource defaults and inherited backend image for both Provider and Computer
Connector. A compatible backend image must contain the Provider and credential
commands. The image digest is not independently duplicated or selected by tag.
Override `id` when retaining a Provider identity from an existing deployment;
changing the ID does not transfer its existing Computers or native bindings.

The generated `<fullname>-computers` namespace is separate from product
credentials. It has Helm `keep` and Argo `Prune=false,Delete=false` protection.
Deleting/disabling a Provider, uninstalling a release or changing its configured
namespace must not be interpreted as permission to delete saved Computer data.
An advanced `managedNamespace` with `createManagedNamespace: false` selects an
already existing isolated namespace; the product namespace is rejected. Operators
own eventual authorized cleanup of retained namespaces and their data.

### Default ServiceAccount authentication

The Provider receives a projected token with audience `chapchoo-provider-auth`.
Control receives the exact bootstrap mapping from its trusted local cluster,
namespace/SA name and expected audience to the Provider ID. It validates the
presented token through TokenReview before admission and operational-token
exchange. Recreating the same SA name permits fresh valid tokens; it does not
make the deleted object's token valid. Permitted namespace administrators who
can recreate/use that SA are inside this deployment trust boundary.

Only Control-containing workloads get the `tokenreviews.create` ClusterRole and
an API-audience credential. Separate Client/Admin/web workloads do not. A combined
backend shares Control's process authority. The Provider's native Kubernetes API
credential remains separate from its dedicated authentication-audience token.
Provider runtime gets neither product DB configuration nor the installation root.

Provider operational credentials are valid for 15 minutes with proactive
re-authentication after five minutes. TokenReview failure or unavailability grants
no new credential and never selects a weaker method or extends existing expiry.
Already-issued operational authority can survive until its existing expiry.

The Provider gets namespaced Pod get/list/watch/create/delete and PVC/Secret
get/create/delete only in its managed execution namespace. Namespaced Secret get
can read every Secret there, so never put database/root/authentication/pull
credentials in that namespace. Execution Pods use a distinct SA with API token
automount disabled. Namespace separation is not proof of network/VM isolation.

### Advanced protected authentication

Select `kubernetesProvider.authentication.method: secret-key` only deliberately.
Control then receives an allowed Provider-ID bootstrap rule instead of an SA
binding. A finite pre-install/pre-upgrade and Argo PreSync Job invokes the trusted
server-side `provider-credential` command from `backend.image` using the existing
root Secret. It
creates or validates one dedicated Secret field `credential` and never prints or
overwrites credential material. The Provider mounts only that output Secret,
**not the root Secret**. Default SA authentication renders no issuer Job.

Issuer SA/Role/Binding precede the Job at hook wave/weight -30; the Job runs at
-20, before preparation (-10) and normal Provider startup. Its authority is named
Secret get and namespace Secret create. Kubernetes cannot name-limit create;
that finite trusted server-side capability is an explicit operational boundary.
Successful hooks are cleaned up; failures remain for diagnosis and have no
automatic Job restart/retry. The generated credential itself is retained outside
disposable hook ownership and reused on upgrade. A mismatching/corrupt existing
credential fails explicitly instead of rotating it. Reissuing a credential is
not revocation of older valid credentials for that ID. Remove its bootstrap
allowed ID to deny further exchange.

An advanced `authentication.existingCredentialSecret` skips issuer automation and
mounts a privately provisioned `credential` field. Do not put the value into Helm
values. This override is not valid for projected SA authentication.

### Native resources and external Control

Computer execution Pods do not receive product image-pull credentials. Their
image must be publicly pullable or already available under the operator's existing
cluster policy; this chart does not copy product pull Secrets into execution.
StorageClass omission uses an existing cluster default; the chart provisions no
StorageClass or storage backend. Requested capacity is not proof of a hard quota.
Stop/restart preserve the original dedicated PVC; missing/unverified original
storage never authorizes replacement. Native resource UID acknowledgements,
not Helm, authorize runtime Connector bootstrap issuance. A Running Provider Pod
is not proof of authenticated connectivity or storage correctness.

For a Provider-only installation, set `backend.enabled: false`, an explicit
`kubernetesProvider.controlEndpoint`, and a compatible Provider/Computer image
(or the backend image used as their default). Install matching trust on the
external Control independently: this chart cannot configure an unrelated server.
An external Kubernetes execution substrate may use another permitted authentication
method. There is no network Trust choice or Docker Provider Deployment in this
chart. Component TLS/exposure remains operator-owned.

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
nor a new migration facility. Web and preparation pods mount no API token. Control has explicit review
credentials only for enabled SA authentication; Provider, credential issuance and
setup bootstrap have their own narrow authority.

Use **full Argo CD application sync**: selective sync skips hooks. The reference
has no automatic sync/prune, namespace creation, retry loop or
`ApplyOutOfSyncOnly`. Replace every example destination/revision/image before use.
Kustomize patches replace `helm.valuesObject` wholesale; retain complete explicit
prerequisites. A hook does not stop old writers. Coordinate contract/schema
cutover. Upgrading from the previous owner-bearer authentication requires stopping
all old Control/Provider owners before schema preparation, invalidating old
Provider operational authority and reconnecting the same identities with the new
configuration. Preserve Connector credentials and original data; transport
reconnect is not credential re-enrollment. A rolling mixed-old/new rollout and
zero downtime are not supported for this transition. A
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
