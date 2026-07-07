CREATE TABLE episodes (
    id TEXT PRIMARY KEY NOT NULL,
    podcast_id TEXT NOT NULL,
    guid TEXT NOT NULL,
    title TEXT NOT NULL,
    audio_url TEXT NOT NULL,
    local_file_path TEXT,
    state TEXT NOT NULL DEFAULT 'pending',

    -- If you unsubscribe from a podcast, automatically delete its episode records
    FOREIGN KEY(podcast_id) REFERENCES podcasts(id) ON DELETE CASCADE,

    -- Ensure we never download the exact same episode twice for a podcast
    UNIQUE(podcast_id, guid)
);