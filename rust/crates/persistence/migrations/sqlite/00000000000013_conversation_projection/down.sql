CREATE TEMP TABLE projection_down_guard (empty INTEGER NOT NULL CHECK(empty=0));
INSERT INTO projection_down_guard SELECT
    (SELECT count(*) FROM conversation_records) +
    (SELECT count(*) FROM conversation_record_fields) +
    (SELECT count(*) FROM conversation_input_delivery) +
    (SELECT count(*) FROM conversation_requests) +
    (SELECT count(*) FROM conversation_answers) +
    (SELECT count(*) FROM conversation_input_preparation) +
    (SELECT count(*) FROM conversation_native_state) +
    (SELECT count(*) FROM conversation_stops);
DROP TABLE projection_down_guard;
DROP TABLE conversation_native_state;
DROP TABLE conversation_stops;
DROP TABLE conversation_input_delivery;
DROP TABLE conversation_input_preparation;
DROP TABLE conversation_answers;
DROP TABLE conversation_requests;
DROP TABLE conversation_record_fields;
DROP TABLE conversation_records;
