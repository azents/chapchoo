-- Replace only the alias column; preserve IDs, ciphertext and incoming references.
ALTER TABLE external_authentications ADD COLUMN optional_alias TEXT NOT NULL DEFAULT '' CHECK(length(optional_alias) BETWEEN 0 AND 256);
UPDATE external_authentications SET optional_alias=name;
ALTER TABLE external_authentications DROP COLUMN name;
ALTER TABLE external_authentications RENAME COLUMN optional_alias TO name;
