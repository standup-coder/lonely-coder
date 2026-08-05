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

-- =====================================================================
-- W2: lobbies + room + match queue
-- =====================================================================

-- A lobby is the "tinder match → mutual-yes → room" container. Status
-- transitions: waiting (1..3 seats filled) → negotiating (4 seats, voting)
-- → matched (all votes accept) → closed (room opened, or expired/abandoned).
CREATE TABLE IF NOT EXISTS lobbies (
  id            TEXT PRIMARY KEY,                      -- short, shareable id
  topic         TEXT,                                   -- what the host wants to brainstorm
  status        TEXT    NOT NULL DEFAULT 'waiting',     -- waiting | negotiating | matched | closed
  created_by    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  matched_at    TEXT,                                   -- when the last accept vote landed
  expires_at    TEXT                                    -- negotiated deadline; nullable until matched
);
CREATE INDEX IF NOT EXISTS idx_lobbies_status ON lobbies(status);

-- One row per (lobby, user). `seat_role` is 'host' for the creator and
-- 'guest' otherwise. We use it for the seat visual + to enforce the
-- "host can never leave while others are still here" rule.
CREATE TABLE IF NOT EXISTS lobby_seats (
  lobby_id      TEXT    NOT NULL REFERENCES lobbies(id) ON DELETE CASCADE,
  user_id       INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  seat_role     TEXT    NOT NULL DEFAULT 'guest',        -- 'host' | 'guest'
  joined_at     TEXT    NOT NULL DEFAULT (datetime('now')),
  vote          TEXT,                                    -- 'accept' | 'skip' | NULL
  voted_at      TEXT,
  PRIMARY KEY (lobby_id, user_id)
);
CREATE INDEX IF NOT EXISTS idx_lobby_seats_user ON lobby_seats(user_id);

-- A room is created from a matched lobby. The brainstorming session
-- lives here: canvas + chat + AI.
CREATE TABLE IF NOT EXISTS rooms (
  id            TEXT PRIMARY KEY,
  lobby_id      TEXT    NOT NULL UNIQUE REFERENCES lobbies(id) ON DELETE CASCADE,
  started_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  ended_at      TEXT
);

-- Append-only event log for the room. We don't store the canvas as a
-- single blob; the room replays events on join. Kinds: canvas.put, chat,
-- ai.thinking, ai.delta, ai.done, member.joined, member.left, etc.
CREATE TABLE IF NOT EXISTS room_events (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  room_id       TEXT    NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
  user_id       INTEGER REFERENCES users(id) ON DELETE SET NULL,  -- nullable: AI events have no user
  kind          TEXT    NOT NULL,
  payload       TEXT    NOT NULL,                                  -- JSON
  created_at    TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_room_events_room ON room_events(room_id, id);

-- The match queue is the "I'm here, find me a squad" waitlist. Rows are
-- short-lived: they're inserted when the user enters the queue and
-- removed when the matching engine creates a lobby for them.
CREATE TABLE IF NOT EXISTS match_queue (
  user_id       INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  enqueued_at   TEXT    NOT NULL DEFAULT (datetime('now')),
  -- JSON-encoded MatchPreferences (languages, skill_level, mode)
  preferences   TEXT    NOT NULL DEFAULT '{}'
);
CREATE INDEX IF NOT EXISTS idx_match_queue_enqueued ON match_queue(enqueued_at);
