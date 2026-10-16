-- Minimal migration for Module 20: Caching and Optimization.
CREATE TABLE IF NOT EXISTS module_20_items (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
