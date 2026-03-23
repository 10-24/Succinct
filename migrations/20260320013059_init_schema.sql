-- rm template.db ; sqlx database setup --database-url sqlite:template.db
CREATE TABLE IF NOT EXISTS file (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    hash INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    parent_id BIGINT NOT NULL
);
INSERT INTO file (id, name, hash, modified_at, parent_id) VALUES (
    -4513623453135682776, '', 0, 0, 0
);
CREATE TABLE IF NOT EXISTS queued_update (
    file_id INTEGER PRIMARY KEY,
    updated_at INTEGER NOT NULL,
    depth INTEGER NOT NULL
);
CREATE INDEX queued_update ON files(depth);

CREATE TABLE IF NOT EXISTS queued_delete (
    file_id INTEGER PRIMARY KEY,
    deleted_at INTEGER NOT NULL,
    path TEXT NOT NULL
);
