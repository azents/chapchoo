-- A new preparation can keep the existing callable selection on the same owner.
ALTER TABLE conversation_startups ADD COLUMN external_selection_reused BOOLEAN NOT NULL DEFAULT FALSE
    CHECK(external_selection_reused IN (FALSE, TRUE));
