CREATE TABLE IF NOT EXISTS file (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    hash INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    parent_id BIGINT NOT NULL
);
INSERT INTO file (id, name, hash, modified_at, parent_id) VALUES (
    1, '', 0, 0, 0
);
CREATE TABLE IF NOT EXISTS queued_update (
    file_id INTEGER PRIMARY KEY,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS queued_delete (
    file_id INTEGER PRIMARY KEY,
    deleted_at INTEGER NOT NULL
);
