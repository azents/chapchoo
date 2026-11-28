LOCK TABLE agent_mcp_keys IN ACCESS EXCLUSIVE MODE;
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM agent_mcp_keys) THEN
        RAISE EXCEPTION 'Cannot discard reserved Agent MCP names';
    END IF;
END $$;
DROP TABLE agent_mcp_keys;
