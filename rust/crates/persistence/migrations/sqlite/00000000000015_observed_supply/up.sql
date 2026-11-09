-- Existing observations remain unknown; no saved selection is inferred as applied.
CREATE TABLE conversation_applied_supply (
    conversation_id BLOB PRIMARY KEY NOT NULL REFERENCES conversation_native_state(conversation_id) ON DELETE CASCADE CHECK(typeof(conversation_id)='blob' AND length(conversation_id)=16),
    supply_kind TEXT NOT NULL CHECK(supply_kind IN ('MANAGED','EXISTING')),
    executable_path TEXT,
    CHECK((supply_kind='MANAGED' AND executable_path IS NULL) OR
          (supply_kind='EXISTING' AND executable_path IS NOT NULL AND length(executable_path) BETWEEN 1 AND 4096))
);
