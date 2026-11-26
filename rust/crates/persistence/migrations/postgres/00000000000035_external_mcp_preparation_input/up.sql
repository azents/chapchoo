-- Retain the input this preparation serves after pickup deletes that input.
ALTER TABLE conversation_startups ADD COLUMN external_input_id UUID;
-- Only a possible refresh can keep its exact applied predecessor in service.
ALTER TABLE conversation_startups ADD COLUMN external_predecessor_id UUID;
ALTER TABLE conversation_input_preparation ADD COLUMN ready_connection_id UUID;
ALTER TABLE conversation_startups ADD COLUMN external_ready_connection_id UUID;
