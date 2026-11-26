LOCK TABLE agent_integrations, agent_mcp_settings, agent_integration_receipts IN ACCESS EXCLUSIVE MODE;
CREATE TEMP TABLE agent_integration_downgrade_guard(n BIGINT CHECK(n=0));
INSERT INTO agent_integration_downgrade_guard SELECT count(*) FROM agent_integrations;
INSERT INTO agent_integration_downgrade_guard SELECT count(*) FROM agent_integration_receipts;
DROP TABLE agent_integration_downgrade_guard;
DROP TABLE agent_integration_receipts;
DROP TABLE agent_mcp_settings;
DROP TABLE agent_integrations;
