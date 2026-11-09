DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM conversation_records)
       OR EXISTS (SELECT 1 FROM conversation_record_fields)
       OR EXISTS (SELECT 1 FROM conversation_input_delivery)
       OR EXISTS (SELECT 1 FROM conversation_requests)
       OR EXISTS (SELECT 1 FROM conversation_answers)
       OR EXISTS (SELECT 1 FROM conversation_input_preparation)
       OR EXISTS (SELECT 1 FROM conversation_native_state)
       OR EXISTS (SELECT 1 FROM conversation_stops) THEN
        RAISE EXCEPTION 'Conversation projection downgrade requires empty new state';
    END IF;
END $$;
DROP TABLE conversation_native_state;
DROP TABLE conversation_stops;
DROP TABLE conversation_input_delivery;
DROP TABLE conversation_input_preparation;
DROP TABLE conversation_answers;
DROP TABLE conversation_requests;
DROP TABLE conversation_record_fields;
DROP TABLE conversation_records;
