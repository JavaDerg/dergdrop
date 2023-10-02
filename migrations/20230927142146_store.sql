CREATE TABLE files (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    created timestamp NOT NULL DEFAULT now(),
    uploaded timestamp DEFAULT NULL,
    meta bytea NOT NULL
);

CREATE INDEX files_created_idx
    ON files (created);

CREATE INDEX files_uploaded_idx
    ON files (uploaded);
