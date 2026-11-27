-- Retain the input this preparation serves after pickup deletes that input.
ALTER TABLE conversation_startups ADD COLUMN external_input_id BLOB
    CHECK(external_input_id IS NULL OR (typeof(external_input_id) = 'blob' AND length(external_input_id) = 16));
-- Only a possible refresh can keep its exact applied predecessor in service.
ALTER TABLE conversation_startups ADD COLUMN external_predecessor_id BLOB
    CHECK(external_predecessor_id IS NULL OR (typeof(external_predecessor_id) = 'blob' AND length(external_predecessor_id) = 16));
ALTER TABLE conversation_input_preparation ADD COLUMN ready_connection_id BLOB
    CHECK(ready_connection_id IS NULL OR (typeof(ready_connection_id) = 'blob' AND length(ready_connection_id) = 16));
ALTER TABLE conversation_startups ADD COLUMN external_ready_connection_id BLOB
    CHECK(external_ready_connection_id IS NULL OR (typeof(external_ready_connection_id) = 'blob' AND length(external_ready_connection_id) = 16));
