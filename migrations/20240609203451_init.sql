-- Add migration script here
CREATE TABLE task (
    id INTEGER PRIMARY KEY NOT NULL,
    description TEXT NOT NULL,
    done BOOLEAN NOT NULL DEFAULT 0
);
