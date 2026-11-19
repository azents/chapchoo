ALTER TABLE conversation_work_items ADD COLUMN reply_ready BOOLEAN NOT NULL DEFAULT TRUE
    CHECK (reply_ready IN (FALSE, TRUE));
