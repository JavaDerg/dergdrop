CREATE TABLE files (
    id uuid PRIMARY KEY,
    completed timestamp DEFAULT NULL,
    meta bytea NOT NULL
);

CREATE INDEX files_uploaded_idx
    ON files (completed);
