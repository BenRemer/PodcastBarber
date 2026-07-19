CREATE TABLE detection_segments (
    id TEXT PRIMARY KEY NOT NULL,
    episode_id TEXT NOT NULL,
    start_time REAL NOT NULL,
    end_time REAL NOT NULL,
    text TEXT NOT NULL,
    ad_score INTEGER NOT NULL,
    is_ad BOOLEAN NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(episode_id, start_time, end_time),

    FOREIGN KEY (episode_id)
        REFERENCES episodes(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_detection_segments_episode_id
    ON detection_segments(episode_id);