-- Create the files table
CREATE TABLE IF NOT EXISTS files (
    id BIGINT PRIMARY KEY,
    name TEXT NOT NULL,
    hash INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    parent_id BIGINT NOT NULL
);
