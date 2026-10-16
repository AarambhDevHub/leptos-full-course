-- Minimal migration for Module 17: File Uploads.
CREATE TABLE IF NOT EXISTS module_17_items (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
