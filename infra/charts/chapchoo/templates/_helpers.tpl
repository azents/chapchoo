{{- define "chapchoo.fullname" -}}
{{- default (printf "%s-chapchoo" .Release.Name | trunc 43 | trimSuffix "-") .Values.fullnameOverride -}}
{{- end -}}

{{- define "chapchoo.selector" -}}
{{- dict "app.kubernetes.io/name" "chapchoo" "app.kubernetes.io/instance" .root.Release.Name "app.kubernetes.io/component" .component | toJson -}}
{{- end -}}

{{- define "chapchoo.labels" -}}
{{- $labels := include "chapchoo.selector" . | fromJson -}}
{{- $_ := set $labels "app.kubernetes.io/managed-by" .root.Release.Service -}}
{{- $_ := set $labels "helm.sh/chart" (printf "chapchoo-%s" .root.Chart.Version) -}}
{{- toJson $labels -}}
{{- end -}}

{{- define "chapchoo.image" -}}
{{- $repo := required "An enabled component requires image.repository" .repository -}}
{{- if .digest -}}
{{- printf "%s@%s" $repo .digest -}}
{{- else -}}
{{- printf "%s:%s" $repo (required "An enabled component requires image.tag or image.digest" .tag) -}}
{{- end -}}
{{- end -}}

{{- define "chapchoo.secretEnv" -}}
{{- dict "name" .name "valueFrom" (dict "secretKeyRef" (dict "name" .secret "key" .key "optional" false)) | toJson -}}
{{- end -}}

{{- define "chapchoo.databaseEnv" -}}
{{- list
  (include "chapchoo.secretEnv" (dict "name" "CHAPCHOO_POSTGRES_URL" "secret" .Values.database.existingSecret "key" .Values.database.urlKey) | fromJson)
  (include "chapchoo.secretEnv" (dict "name" "CHAPCHOO_SECRET_KEY" "secret" .Values.secretKey.existingSecret "key" .Values.secretKey.key) | fromJson)
  | toJson -}}
{{- end -}}

{{- define "chapchoo.backendEnv" -}}
{{- $env := include "chapchoo.databaseEnv" . | fromJsonArray -}}
{{- $env = append $env (dict "name" "CHAPCHOO_GENERAL_LISTEN" "value" "0.0.0.0:50051") -}}
{{- $env = append $env (dict "name" "CHAPCHOO_ADMIN_LISTEN" "value" "0.0.0.0:50052") -}}
{{- $env = append $env (dict "name" "CHAPCHOO_GENERAL_WEB_URL" "value" .Values.general.publicUrl) -}}
{{- $proofSecret := default (printf "%s-setup-proof" (include "chapchoo.fullname" .)) .Values.setupProof.existingSecret -}}
{{- $env = append $env (include "chapchoo.secretEnv" (dict "name" "CHAPCHOO_SETUP_PROOF" "secret" $proofSecret "key" .Values.setupProof.key) | fromJson) -}}
{{- if .Values.mail.enabled -}}
{{- $env = append $env (dict "name" "CHAPCHOO_SES_SENDER" "value" .Values.mail.sender) -}}
{{- $env = append $env (dict "name" "CHAPCHOO_SES_REGION" "value" .Values.mail.region) -}}
{{- $env = append $env (include "chapchoo.secretEnv" (dict "name" "AWS_ACCESS_KEY_ID" "secret" .Values.mail.existingCredentialsSecret "key" .Values.mail.accessKeyIdKey) | fromJson) -}}
{{- $env = append $env (include "chapchoo.secretEnv" (dict "name" "AWS_SECRET_ACCESS_KEY" "secret" .Values.mail.existingCredentialsSecret "key" .Values.mail.secretAccessKeyKey) | fromJson) -}}
{{- end -}}
{{- toJson $env -}}
{{- end -}}

{{- define "chapchoo.securityContext" -}}
{{- dict "runAsNonRoot" true "runAsUser" 10001 "runAsGroup" 10001 "allowPrivilegeEscalation" false "capabilities" (dict "drop" (list "ALL")) | toJson -}}
{{- end -}}

{{- define "chapchoo.validatePublicUrl" -}}
{{- $url := required "A backend/general/Admin deployment requires the explicit general publicUrl; every enabled web also requires publicUrl" .url -}}
{{- $parsed := urlParse $url -}}
{{- if or (ne $parsed.scheme "https") (empty $parsed.host) (not (empty $parsed.userinfo)) (not (empty $parsed.query)) (not (empty $parsed.fragment)) -}}
{{- fail "Kubernetes publicUrl must be explicit HTTPS without credentials, query or fragment" -}}
{{- end -}}
{{- $path := trimSuffix "/" $parsed.path -}}
{{- if or (contains "//" $path) (contains "%" $url) (regexMatch "[;%\\\\]" $path) (regexMatch "(^|/)\\.{1,2}(/|$)" $path) -}}
{{- fail "publicUrl path must be canonical, without rewrites, escapes or duplicate slashes" -}}
{{- end -}}
{{- end -}}

{{- define "chapchoo.validateIngressHost" -}}
{{- if or (gt (len .) 253) (not (regexMatch "^([a-z0-9]([-a-z0-9]*[a-z0-9])?\\.)*[a-z0-9]([-a-z0-9]*[a-z0-9])?$" .)) (regexMatch "^[0-9]+\\.[0-9]+\\.[0-9]+\\.[0-9]+$" .) -}}
{{- fail "Ingress hosts must be explicit lowercase DNS names, not IP addresses, wildcards or ports" -}}
{{- end -}}
{{- end -}}

{{- define "chapchoo.validate" -}}
{{- if not .Values.prerequisitesConfirmed -}}
{{- fail "Set prerequisitesConfirmed only after external database, fixed-key/TLS/pull Secrets and trusted-network prerequisites already exist" -}}
{{- end -}}
{{- if not (or .Values.backend.enabled .Values.general.enabled .Values.admin.enabled) -}}
{{- fail "Select at least one backend or web component" -}}
{{- end -}}
{{- if .Values.backend.enabled -}}
{{- if and (empty .Values.setupProof.existingSecret) (ne .Values.setupProof.key "setup-proof") -}}
{{- fail "Generated setup proof uses the fixed setup-proof Secret field" -}}
{{- end -}}
{{- $_ := required "database.existingSecret is required for external PostgreSQL" .Values.database.existingSecret -}}
{{- $_ := required "secretKey.existingSecret is required; the fixed key is never generated or replaced" .Values.secretKey.existingSecret -}}
{{- include "chapchoo.validatePublicUrl" (dict "url" .Values.general.publicUrl) -}}
{{- end -}}
{{- if and .Values.mail.enabled (not .Values.backend.enabled) -}}
{{- fail "mail configuration belongs to an enabled backend" -}}
{{- end -}}
{{- if .Values.mail.enabled -}}
{{- $_ := required "mail.sender is required" .Values.mail.sender -}}
{{- $_ := required "mail.region is required" .Values.mail.region -}}
{{- $_ := required "mail.existingCredentialsSecret is required; no AWS provider is provisioned" .Values.mail.existingCredentialsSecret -}}
{{- end -}}
{{- range $role := list "general" "admin" -}}
{{- $web := index $.Values $role -}}
{{- if $web.enabled -}}
{{- include "chapchoo.validatePublicUrl" (dict "url" $web.publicUrl) -}}
{{- if and (not $.Values.backend.enabled) (empty $web.backendUrl) -}}
{{- fail "A web-only deployment requires an explicit matching backendUrl" -}}
{{- end -}}
{{- if $web.backendUrl -}}
{{- $parsed := urlParse $web.backendUrl -}}
{{- if or (not (has $parsed.scheme (list "http" "https"))) (empty $parsed.host) (contains "%" $web.backendUrl) (not (empty $parsed.userinfo)) (not (empty $parsed.query)) (not (empty $parsed.fragment)) (not (has $parsed.path (list "" "/"))) -}}
{{- fail "backendUrl must be an explicit http/https gRPC endpoint without credentials, query, fragment or path prefix" -}}
{{- end -}}
{{- end -}}
{{- end -}}
{{- end -}}
{{- if and .Values.general.enabled .Values.admin.enabled (eq (trimSuffix "/" .Values.general.publicUrl) (trimSuffix "/" .Values.admin.publicUrl)) -}}
{{- fail "General and Admin must not share the same public route" -}}
{{- end -}}
{{- if .Values.webIngress.enabled -}}
{{- if not (or .Values.general.enabled .Values.admin.enabled) -}}{{- fail "webIngress requires an enabled web deployment" -}}{{- end -}}
{{- $_ := required "webIngress.className must identify an existing ingress-nginx controller" .Values.webIngress.className -}}
{{- $_ := required "webIngress.existingTlsSecret is required" .Values.webIngress.existingTlsSecret -}}
{{- range $role := list "general" "admin" -}}
{{- $web := index $.Values $role -}}
{{- if $web.enabled -}}
{{- $parsed := urlParse $web.publicUrl -}}
{{- include "chapchoo.validateIngressHost" $parsed.hostname -}}
{{- if not (or (eq $parsed.host $parsed.hostname) (eq $parsed.host (printf "%s:443" $parsed.hostname))) -}}
{{- fail "The chart web Ingress serves HTTPS on 443; use an external edge for other public ports" -}}
{{- end -}}
{{- end -}}
{{- end -}}
{{- if and .Values.general.enabled .Values.admin.enabled (eq (urlParse .Values.general.publicUrl).hostname (urlParse .Values.admin.publicUrl).hostname) (eq (trimSuffix "/" (urlParse .Values.general.publicUrl).path) (trimSuffix "/" (urlParse .Values.admin.publicUrl).path)) -}}
{{- fail "General and Admin web Ingress routes must be distinct after HTTPS port normalization" -}}
{{- end -}}
{{- end -}}
{{- if .Values.grpcIngress.enabled -}}
{{- if not .Values.backend.enabled -}}{{- fail "grpcIngress requires this chart's backend" -}}{{- end -}}
{{- $_ := required "grpcIngress.className must identify an existing ingress-nginx controller" .Values.grpcIngress.className -}}
{{- if not (or .Values.grpcIngress.general.enabled .Values.grpcIngress.admin.enabled) -}}{{- fail "grpcIngress requires at least one explicit route" -}}{{- end -}}
{{- range $role := list "general" "admin" -}}
{{- $route := index $.Values.grpcIngress $role -}}
{{- if $route.enabled -}}
{{- $_ := required "Each enabled gRPC edge requires an explicit host" $route.host -}}
{{- include "chapchoo.validateIngressHost" $route.host -}}
{{- $_ := required "Each enabled gRPC edge requires an existing TLS Secret" $route.existingTlsSecret -}}
{{- if and $.Values.webIngress.enabled (eq $.Values.webIngress.className $.Values.grpcIngress.className) -}}
{{- range $webRole := list "general" "admin" -}}
{{- $web := index $.Values $webRole -}}
{{- if and $web.enabled (empty (trimSuffix "/" (urlParse $web.publicUrl).path)) (eq (urlParse $web.publicUrl).hostname $route.host) -}}
{{- fail "A gRPC catch-all and a root web Ingress cannot share the same ingress class and host" -}}
{{- end -}}
{{- end -}}
{{- end -}}
{{- end -}}
{{- end -}}
{{- if and .Values.grpcIngress.general.enabled .Values.grpcIngress.admin.enabled (eq .Values.grpcIngress.general.host .Values.grpcIngress.admin.host) -}}
{{- fail "General and Admin gRPC edge hosts must be distinct; Admin can remain unexposed" -}}
{{- end -}}
{{- end -}}
{{- end -}}
