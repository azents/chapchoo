CREATE TABLE conversation_work_items (
    conversation_id UUID NOT NULL,
    goal_key TEXT NOT NULL,
    title TEXT NOT NULL,
    tasks_json TEXT NOT NULL,
    PRIMARY KEY (conversation_id, goal_key),
    FOREIGN KEY (conversation_id, goal_key)
        REFERENCES conversation_goals(conversation_id, goal_key) ON DELETE CASCADE
);
