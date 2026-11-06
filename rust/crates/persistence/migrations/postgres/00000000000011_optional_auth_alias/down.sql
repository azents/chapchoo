ALTER TABLE external_authentications DROP CONSTRAINT external_authentications_name_check;
ALTER TABLE external_authentications ADD CONSTRAINT external_authentications_name_check CHECK(length(name) BETWEEN 1 AND 256);
