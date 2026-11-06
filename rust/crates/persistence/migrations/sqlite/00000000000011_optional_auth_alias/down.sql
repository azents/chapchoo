-- Copying an empty alias fails atomically rather than inventing or discarding data.
ALTER TABLE external_authentications ADD COLUMN required_alias TEXT NOT NULL DEFAULT 'Migration' CHECK(length(required_alias) BETWEEN 1 AND 256);
UPDATE external_authentications SET required_alias=name;
ALTER TABLE external_authentications DROP COLUMN name;
ALTER TABLE external_authentications RENAME COLUMN required_alias TO name;
