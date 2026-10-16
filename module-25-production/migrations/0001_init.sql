-- Module 25: production capstone schema.
CREATE TABLE IF NOT EXISTS production_users (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS production_notes (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES production_users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS activity_events (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID REFERENCES production_users(id) ON DELETE SET NULL,
    message TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'info',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS production_users_email_idx ON production_users (email);
CREATE INDEX IF NOT EXISTS production_notes_user_idx ON production_notes (user_id, updated_at DESC);
CREATE INDEX IF NOT EXISTS activity_events_created_idx ON activity_events (created_at DESC);
