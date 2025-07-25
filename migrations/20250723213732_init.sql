-- Add migration script here
CREATE TABLE tasks(
    id TEXT PRIMARY KEY, -- UUID
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('Todo', 'In Progress', 'Done')),
    priority TEXT NOT NULL CHECK (priority IN ('Low', 'Medium', 'High')),
    due_date TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE task_tags( -- many to many relation
    task_id TEXT NOT NULL,
    tag TEXT NOT NULL,
    PRIMARY KEY (task_id, tag),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

-- Indexes for common queries
CREATE INDEX idx_title ON tasks(title);
CREATE INDEX idx_status ON tasks(status);
CREATE INDEX idx_priority ON tasks(priority);
CREATE INDEX idx_due_date ON tasks(due_date);
CREATE INDEX idx_created_at ON tasks(created_at);
CREATE INDEX idx_updated_at ON tasks(updated_at);

CREATE INDEX idx_tag ON task_tags(tag);
