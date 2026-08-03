-- CodeMatch schema. SQLite, single file. Safe to re-run.

CREATE TABLE IF NOT EXISTS users (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  github_id     INTEGER UNIQUE NOT NULL,
  username      TEXT    NOT NULL,
  display_name  TEXT,
  email         TEXT,
  avatar_url    TEXT,
  bio           TEXT,
  -- JSON arrays / strings stored as TEXT to keep the schema flat.
  -- Real schemas would json_extract() these in queries — we just round-trip.
  skills        TEXT    NOT NULL DEFAULT '[]',
  timezone      TEXT,
  primary_ai    TEXT,                                  -- 'claude' | 'gpt4' | 'gemini' | 'deepseek'
  -- Free-text single sentence describing what they want to brainstorm about.
  -- Surfaced on the profile card; also the seed for matching.
  topic         TEXT,
  is_dev_seed   INTEGER NOT NULL DEFAULT 0,            -- 1 if the row was created by the dev seed
  created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  last_active_at TEXT   NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_users_github_id ON users(github_id);
CREATE INDEX IF NOT EXISTS idx_users_last_active ON users(last_active_at);

CREATE TABLE IF NOT EXISTS sessions (
  -- Opaque random token. 32 bytes hex = 64 chars.
  token       TEXT PRIMARY KEY,
  user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  expires_at  TEXT    NOT NULL,
  created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
