CREATE TEMP TABLE supply_down_guard (empty INTEGER NOT NULL CHECK(empty=0));
INSERT INTO supply_down_guard SELECT count(*) FROM conversation_applied_supply;
DROP TABLE supply_down_guard;
DROP TABLE conversation_applied_supply;
