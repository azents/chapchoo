CREATE TEMP TABLE conversation_work_downgrade_guard(n BIGINT CHECK (n=0));
INSERT INTO conversation_work_downgrade_guard SELECT count(*) FROM conversation_work_items;
DROP TABLE conversation_work_downgrade_guard;
DROP TABLE conversation_work_items;
