DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM agent_harness_supplies)
       OR EXISTS (SELECT 1 FROM agent_harness_supply_requests)
       OR EXISTS (SELECT 1 FROM conversations)
       OR EXISTS (SELECT 1 FROM agent_conversations)
       OR EXISTS (SELECT 1 FROM conversation_inputs)
       OR EXISTS (SELECT 1 FROM conversation_event_pairs) THEN
        RAISE EXCEPTION 'Conversation downgrade requires empty new state';
    END IF;
END $$;
DROP TABLE conversation_event_pairs;
DROP TABLE conversation_inputs;
DROP TABLE agent_conversations;
DROP TABLE conversations;
DROP TABLE agent_harness_supply_requests;
DROP TABLE agent_harness_supplies;
