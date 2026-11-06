-- Refuse destructive downgrade even when only accepted result identity remains.
CREATE TEMP TABLE external_auth_down_guard (empty INTEGER NOT NULL CHECK(empty=0));
INSERT INTO external_auth_down_guard SELECT
    (SELECT count(*) FROM external_authentications) +
    (SELECT count(*) FROM external_auth_attempts) +
    (SELECT count(*) FROM external_auth_requests) +
    (SELECT count(*) FROM agent_authentication_selections);
DROP TABLE external_auth_down_guard;
DROP TABLE agent_authentication_selections;
DROP TABLE external_auth_requests;
DROP TABLE external_authentications;
DROP TABLE external_auth_attempts;
