-- Requires explicit preparation-local assignments and profile initialization.

CREATE TABLE profile_images (
    id UUID PRIMARY KEY NOT NULL,
    item_type TEXT NOT NULL CHECK(item_type='AGENT'),
    internal_name TEXT NOT NULL UNIQUE CHECK(length(internal_name)>0),
    source_kind TEXT NOT NULL CHECK(source_kind='BUILTIN'),
    asset_key TEXT NOT NULL UNIQUE CHECK(length(asset_key)>0)
);
INSERT INTO profile_images(id,item_type,internal_name,source_kind,asset_key)
SELECT id,'AGENT',internal_name,'BUILTIN',asset_key FROM _profile_images;

CREATE TEMP TABLE _profile_fks AS SELECT conrelid::regclass::text AS relation,conname AS name,pg_get_constraintdef(oid) AS definition FROM pg_constraint WHERE contype='f' AND connamespace=current_schema()::regnamespace;
DO $$ DECLARE fk record; BEGIN FOR fk IN SELECT * FROM _profile_fks LOOP EXECUTE format('ALTER TABLE %s DROP CONSTRAINT %I',fk.relation,fk.name); END LOOP; END $$;

ALTER TABLE agents DROP COLUMN cloud_computer_id;

CREATE FUNCTION pg_temp.profile_identity(entity_kind text, original text) RETURNS uuid LANGUAGE sql STRICT AS 'SELECT new_id FROM _profile_id_map WHERE kind=entity_kind AND old_id=original';

ALTER TABLE computer_providers ADD COLUMN registration_key TEXT UNIQUE;
UPDATE computer_providers SET registration_key=id;
ALTER TABLE computer_providers ALTER COLUMN registration_key SET NOT NULL;

ALTER TABLE computer_resources ADD COLUMN allocation_computer_id TEXT, ADD COLUMN allocation_provider_id TEXT, ADD COLUMN allocation_execution_id TEXT;
UPDATE computer_resources SET allocation_computer_id=computer_id,allocation_provider_id=provider_id,allocation_execution_id=execution_id;
ALTER TABLE computer_resources ALTER COLUMN allocation_computer_id SET NOT NULL, ALTER COLUMN allocation_provider_id SET NOT NULL, ADD CHECK(length(allocation_computer_id)>0), ADD CHECK(length(allocation_provider_id)>0), ADD CHECK((execution_id IS NULL)=(allocation_execution_id IS NULL));

ALTER TABLE users ALTER COLUMN id TYPE UUID USING pg_temp.profile_identity('U',id);

ALTER TABLE teams ALTER COLUMN id TYPE UUID USING pg_temp.profile_identity('T',id);

ALTER TABLE teams ALTER COLUMN owner_user_id TYPE UUID USING pg_temp.profile_identity('U',owner_user_id);

ALTER TABLE memberships ALTER COLUMN team_id TYPE UUID USING pg_temp.profile_identity('T',team_id);

ALTER TABLE memberships ALTER COLUMN user_id TYPE UUID USING pg_temp.profile_identity('U',user_id);

ALTER TABLE personal_contexts ALTER COLUMN user_id TYPE UUID USING pg_temp.profile_identity('U',user_id);

ALTER TABLE personal_contexts ALTER COLUMN team_id TYPE UUID USING pg_temp.profile_identity('T',team_id);

ALTER TABLE auth_sessions ALTER COLUMN id TYPE UUID USING pg_temp.profile_identity('S',id);

ALTER TABLE auth_sessions ALTER COLUMN user_id TYPE UUID USING pg_temp.profile_identity('U',user_id);

ALTER TABLE grants ALTER COLUMN id TYPE UUID USING pg_temp.profile_identity('G',id);

ALTER TABLE grants ALTER COLUMN user_id TYPE UUID USING pg_temp.profile_identity('U',user_id);

ALTER TABLE grants ALTER COLUMN issued_by TYPE UUID USING pg_temp.profile_identity('U',issued_by);

ALTER TABLE invitations ALTER COLUMN id TYPE UUID USING pg_temp.profile_identity('I',id);

ALTER TABLE invitations ALTER COLUMN team_id TYPE UUID USING pg_temp.profile_identity('T',team_id);

ALTER TABLE invitations ALTER COLUMN invited_by TYPE UUID USING pg_temp.profile_identity('U',invited_by);

ALTER TABLE login_challenges ALTER COLUMN id TYPE UUID USING pg_temp.profile_identity('L',id);

ALTER TABLE computer_providers ALTER COLUMN id TYPE UUID USING pg_temp.profile_identity('P',id);

ALTER TABLE computer_providers ALTER COLUMN user_owner_id TYPE UUID USING pg_temp.profile_identity('U',user_owner_id);

ALTER TABLE computer_providers ALTER COLUMN owner_team_id TYPE UUID USING pg_temp.profile_identity('T',owner_team_id);

ALTER TABLE provider_keys ALTER COLUMN provider_id TYPE UUID USING pg_temp.profile_identity('P',provider_id);

ALTER TABLE provider_key_requests ALTER COLUMN provider_id TYPE UUID USING pg_temp.profile_identity('P',provider_id);

ALTER TABLE provider_registration_credentials ALTER COLUMN provider_id TYPE UUID USING pg_temp.profile_identity('P',provider_id);

ALTER TABLE computers ALTER COLUMN id TYPE UUID USING pg_temp.profile_identity('C',id);

ALTER TABLE computers ALTER COLUMN owner_user_id TYPE UUID USING pg_temp.profile_identity('U',owner_user_id);

ALTER TABLE computers ALTER COLUMN provider_id TYPE UUID USING pg_temp.profile_identity('P',provider_id);

ALTER TABLE computers ALTER COLUMN current_execution_id TYPE UUID USING pg_temp.profile_identity('E',current_execution_id);

ALTER TABLE computers ALTER COLUMN expected_old_execution_id TYPE UUID USING pg_temp.profile_identity('E',expected_old_execution_id);

ALTER TABLE computers ALTER COLUMN removed_agent_id TYPE UUID USING pg_temp.profile_identity('A',removed_agent_id);

ALTER TABLE computers ALTER COLUMN power_source_connection_id TYPE UUID USING pg_temp.profile_identity('CN',power_source_connection_id);

ALTER TABLE computers ALTER COLUMN power_source_execution_id TYPE UUID USING pg_temp.profile_identity('E',power_source_execution_id);

ALTER TABLE computers ALTER COLUMN environment_source_connection_id TYPE UUID USING pg_temp.profile_identity('CN',environment_source_connection_id);

ALTER TABLE agents ALTER COLUMN id TYPE UUID USING pg_temp.profile_identity('A',id);

ALTER TABLE agents ALTER COLUMN owner_user_id TYPE UUID USING pg_temp.profile_identity('U',owner_user_id);

ALTER TABLE agents ALTER COLUMN team_id TYPE UUID USING pg_temp.profile_identity('T',team_id);

ALTER TABLE agents ALTER COLUMN computer_id TYPE UUID USING pg_temp.profile_identity('C',computer_id);

ALTER TABLE agent_creation_receipts ALTER COLUMN owner_user_id TYPE UUID USING pg_temp.profile_identity('U',owner_user_id);

ALTER TABLE agent_creation_receipts ALTER COLUMN team_id TYPE UUID USING pg_temp.profile_identity('T',team_id);

ALTER TABLE agent_creation_receipts ALTER COLUMN assigned_agent_id TYPE UUID USING pg_temp.profile_identity('A',assigned_agent_id);

ALTER TABLE computer_executions ALTER COLUMN id TYPE UUID USING pg_temp.profile_identity('E',id);

ALTER TABLE computer_executions ALTER COLUMN computer_id TYPE UUID USING pg_temp.profile_identity('C',computer_id);

ALTER TABLE computer_resources ALTER COLUMN computer_id TYPE UUID USING pg_temp.profile_identity('C',computer_id);

ALTER TABLE computer_resources ALTER COLUMN provider_id TYPE UUID USING pg_temp.profile_identity('P',provider_id);

ALTER TABLE computer_resources ALTER COLUMN execution_id TYPE UUID USING pg_temp.profile_identity('E',execution_id);

ALTER TABLE computer_resources ALTER COLUMN source_connection_id TYPE UUID USING pg_temp.profile_identity('CN',source_connection_id);

ALTER TABLE connector_sessions ALTER COLUMN id TYPE UUID USING pg_temp.profile_identity('CS',id);

ALTER TABLE connector_sessions ALTER COLUMN computer_id TYPE UUID USING pg_temp.profile_identity('C',computer_id);

ALTER TABLE connector_sessions ALTER COLUMN execution_id TYPE UUID USING pg_temp.profile_identity('E',execution_id);

ALTER TABLE connector_key_requests ALTER COLUMN session_id TYPE UUID USING pg_temp.profile_identity('CS',session_id);

ALTER TABLE control_instances ALTER COLUMN id TYPE UUID USING pg_temp.profile_identity('CT',id);

ALTER TABLE provider_connections ALTER COLUMN provider_id TYPE UUID USING pg_temp.profile_identity('P',provider_id);

ALTER TABLE provider_connections ALTER COLUMN control_id TYPE UUID USING pg_temp.profile_identity('CT',control_id);

ALTER TABLE provider_connections ALTER COLUMN connection_id TYPE UUID USING pg_temp.profile_identity('CN',connection_id);

ALTER TABLE connector_connections ALTER COLUMN computer_id TYPE UUID USING pg_temp.profile_identity('C',computer_id);

ALTER TABLE connector_connections ALTER COLUMN session_id TYPE UUID USING pg_temp.profile_identity('CS',session_id);

ALTER TABLE connector_connections ALTER COLUMN control_id TYPE UUID USING pg_temp.profile_identity('CT',control_id);

ALTER TABLE connector_connections ALTER COLUMN connection_id TYPE UUID USING pg_temp.profile_identity('CN',connection_id);

ALTER TABLE agents ADD COLUMN cloud_computer_id UUID GENERATED ALWAYS AS (CASE WHEN computer_kind='CLOUD' THEN computer_id ELSE NULL END) STORED UNIQUE;

ALTER TABLE teams ALTER COLUMN handle DROP NOT NULL;
UPDATE teams SET handle=(SELECT handle FROM _profile_teams WHERE old_id=(SELECT old_id FROM _profile_id_map WHERE kind='T' AND new_id=teams.id));

ALTER TABLE teams ADD CHECK((kind='PERSONAL' AND handle IS NULL) OR (kind='COLLABORATIVE' AND handle IS NOT NULL AND (handle ~ '^[a-z0-9][a-z0-9-]{1,14}[a-z0-9]$' AND octet_length(handle) BETWEEN 3 AND 16)));

ALTER TABLE agents ADD COLUMN handle TEXT, ADD COLUMN profile_image_id UUID, ADD COLUMN palette_key TEXT;
UPDATE agents SET handle=profile.handle,profile_image_id=profile.profile_image_id,palette_key=profile.palette_key FROM _profile_agents profile JOIN _profile_id_map ids ON ids.kind='A' AND ids.old_id=profile.old_id WHERE agents.id=ids.new_id;

ALTER TABLE agents ALTER COLUMN handle SET NOT NULL, ALTER COLUMN profile_image_id SET NOT NULL, ALTER COLUMN palette_key SET NOT NULL, ADD CHECK(handle ~ '^[a-z0-9][a-z0-9-]{1,14}[a-z0-9]$' AND octet_length(handle) BETWEEN 3 AND 16), ADD CHECK(palette_key IN ('butter','sage','apricot','teal','lilac','rose')), ADD FOREIGN KEY(profile_image_id) REFERENCES profile_images(id);

CREATE UNIQUE INDEX agents_handle_key ON agents(handle);

DO $$ DECLARE fk record; BEGIN FOR fk IN SELECT * FROM _profile_fks LOOP EXECUTE format('ALTER TABLE %s ADD CONSTRAINT %I %s',fk.relation,fk.name,fk.definition); END LOOP; END $$;
DROP TABLE _profile_fks;
DROP FUNCTION pg_temp.profile_identity(text,text);
