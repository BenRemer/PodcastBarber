CREATE TABLE transcripts (
    id TEXT PRIMARY KEY NOT NULL,
    episode_id TEXT NOT NULL UNIQUE,
    data TEXT NOT NULL,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (episode_id)
        REFERENCES episodes(id)
        ON DELETE CASCADE
);
