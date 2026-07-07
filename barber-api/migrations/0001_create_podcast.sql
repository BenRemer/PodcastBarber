CREATE TABLE IF NOT EXISTS podcasts (
    id TEXT PRIMARY KEY,
    feed_url TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    image_url TEXT,
    description TEXT,
    author TEXT
);