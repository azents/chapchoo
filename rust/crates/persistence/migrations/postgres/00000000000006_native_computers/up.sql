-- Explicit prepare runs this forward change transactionally. Existing
-- Kubernetes addresses, UIDs, credential rows and connection routes are retained.
ALTER TABLE computer_providers
    DROP CONSTRAINT computer_providers_provider_kind_check,
    ADD COLUMN docker_daemon_id TEXT,
    ADD CONSTRAINT native_provider_kind
        CHECK (provider_kind IN ('KUBERNETES','DOCKER')),
    ADD CONSTRAINT docker_system_provider
        CHECK (provider_kind<>'DOCKER' OR owner_kind='SYSTEM'),
    ADD CONSTRAINT docker_provider_binding
        CHECK (docker_daemon_id IS NULL OR
            (provider_kind='DOCKER' AND octet_length(docker_daemon_id) BETWEEN 1 AND 1024));

ALTER TABLE computer_resources RENAME COLUMN namespace TO native_scope;
ALTER TABLE computer_resources
    DROP CONSTRAINT computer_resources_resource_kind_check,
    DROP CONSTRAINT computer_resources_check,
    ADD COLUMN native_scope_kind TEXT NOT NULL DEFAULT 'KUBERNETES_NAMESPACE',
    ADD COLUMN docker_container_id TEXT,
    ADD COLUMN docker_volume_metadata TEXT,
    ADD CONSTRAINT native_resource_kind
        CHECK (resource_kind IN ('POD','PVC','BOOTSTRAP_SECRET','DOCKER_CONTAINER','DOCKER_VOLUME')),
    ADD CONSTRAINT native_resource_scope
        CHECK (
            (resource_kind IN ('POD','PVC','BOOTSTRAP_SECRET')
                AND native_scope_kind='KUBERNETES_NAMESPACE')
            OR (resource_kind IN ('DOCKER_CONTAINER','DOCKER_VOLUME')
                AND native_scope_kind='DOCKER_DAEMON'
                AND octet_length(native_scope) BETWEEN 1 AND 1024)
        ),
    ADD CONSTRAINT native_saved_resource_role
        CHECK ((resource_kind IN ('PVC','DOCKER_VOLUME'))=(execution_id IS NULL)),
    ADD CONSTRAINT native_kubernetes_uid
        CHECK (resource_kind IN ('POD','PVC','BOOTSTRAP_SECRET') OR resource_uid IS NULL),
    ADD CONSTRAINT native_container_identity
        CHECK (docker_container_id IS NULL OR
            (resource_kind='DOCKER_CONTAINER' AND docker_container_id ~ '^[0-9a-f]{64}$')),
    ADD CONSTRAINT native_volume_identity
        CHECK (docker_volume_metadata IS NULL OR
            (resource_kind='DOCKER_VOLUME'
                AND octet_length(docker_volume_metadata) <= 32768
                AND jsonb_typeof(docker_volume_metadata::jsonb)='object')),
    ADD CONSTRAINT native_docker_present_identity
        CHECK (presence<>'PRESENT'
            OR resource_kind IN ('POD','PVC','BOOTSTRAP_SECRET')
            OR (resource_kind='DOCKER_CONTAINER' AND docker_container_id IS NOT NULL)
            OR (resource_kind='DOCKER_VOLUME' AND docker_volume_metadata IS NOT NULL));

-- Scope kind is determined by resource kind. The existing primary/Provider
-- uniqueness keys now address native_scope without changing historical names.
-- Saved storage is unique across both native saved-resource variants.
DROP INDEX one_saved_pvc_per_computer;
CREATE UNIQUE INDEX one_saved_environment_per_computer ON computer_resources(computer_id)
    WHERE resource_kind IN ('PVC','DOCKER_VOLUME');
