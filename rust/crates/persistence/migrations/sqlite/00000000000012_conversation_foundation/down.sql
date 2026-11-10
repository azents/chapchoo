CREATE TEMP TABLE conversation_down_guard (empty INTEGER NOT NULL CHECK(empty=0));
INSERT INTO conversation_down_guard SELECT
    (SELECT count(*) FROM agent_harness_supplies) +
    (SELECT count(*) FROM agent_harness_supply_requests) +
    (SELECT count(*) FROM conversations) +
    (SELECT count(*) FROM agent_conversations) +
    (SELECT count(*) FROM conversation_inputs) +
    (SELECT count(*) FROM conversation_event_pairs);
DROP TABLE conversation_down_guard;
DROP TABLE conversation_event_pairs;
DROP TABLE conversation_inputs;
DROP TABLE agent_conversations;
DROP TABLE conversations;
DROP TABLE agent_harness_supply_requests;
DROP TABLE agent_harness_supplies;
