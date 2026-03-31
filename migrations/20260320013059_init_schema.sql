-- rm template.db ; sqlx database setup --database-url sqlite:template.db
CREATE TABLE IF NOT EXISTS file (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    hash INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    parent_id BIGINT NOT NULL,
    depth INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);
INSERT INTO file (id, name, hash, modified_at, parent_id,depth,created_at) VALUES (
    -4513623453135682776, '.', 0, 0, 0, 1, 0
);

CREATE TABLE IF NOT EXISTS queued_update (
    file_id INTEGER PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS queued_delete (
    file_id INTEGER PRIMARY KEY,
    deleted_at INTEGER NOT NULL,
    path TEXT NOT NULL
);
