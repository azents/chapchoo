CREATE TEMP TABLE refresh_down_guard (empty INTEGER NOT NULL CHECK(empty=0));
INSERT INTO refresh_down_guard SELECT count(*) FROM authentication_refreshes;
INSERT INTO refresh_down_guard SELECT count(*) FROM execution_authentication_applications;
DROP TABLE refresh_down_guard;
DROP TABLE execution_authentication_applications;
DROP TABLE authentication_refreshes;
