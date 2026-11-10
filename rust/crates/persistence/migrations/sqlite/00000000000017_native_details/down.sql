CREATE TEMP TABLE native_details_down_guard (empty INTEGER NOT NULL CHECK(empty=0));
INSERT INTO native_details_down_guard SELECT count(*) FROM conversation_native_details;
DROP TABLE native_details_down_guard;
DROP TABLE conversation_native_details;
