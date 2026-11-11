CREATE TEMP TABLE conversation_startups_down_guard (empty INTEGER NOT NULL CHECK(empty=0));
INSERT INTO conversation_startups_down_guard SELECT count(*) FROM conversation_startups;
DROP TABLE conversation_startups_down_guard;
DROP TABLE conversation_startups;
