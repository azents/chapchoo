CREATE TEMP TABLE reply_work_downgrade_guard(n BIGINT CHECK (n=0));
INSERT INTO reply_work_downgrade_guard SELECT count(*) FROM conversation_work_items
WHERE title='' OR reply_ready=FALSE;
DROP TABLE reply_work_downgrade_guard;
ALTER TABLE conversation_work_items DROP COLUMN reply_ready;
