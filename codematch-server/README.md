# codematch-server

The backend for [CodeMatch](../codematch-prototype/) — 4-person
brainstorm matching with BYOK LLM agents.

This server is **W1 + W2 + W2d** of the product:

- **W1**: real auth, real users, real profile CRUD, real deck API
- **W2a**: real matching engine + 4-person lobby + mutual-yes voting
- **W2b**: real-time room via WebSocket (canvas + chat fan-out)
- **W2d**: AI proxy — every user's AI is in the room, responding to context

**Deferred to W3** (not in this build):

- Per-user BYOK API key storage (currently one server-wide key)
- Voice channel (WebRTC)
- AI-to-AI conversation (AIs can only nudge, not chat with each other)

---

## Endpoints

### Health / auth (W1)

| Endpoint | Method | Purpose | Auth |
|---|---|---|---|
| `/health` | GET | Liveness probe | none |
| `/auth/status` | GET | Who am I + is the API in dev mode? | none |
| `/auth/github` | GET | Start GitHub OAuth (redirect) | none |
| `/auth/github/callback` | GET | Finish GitHub OAuth (set session cookie) | none |
| `/auth/dev-login?as=HANDLE` | GET | **Dev only** — log in as a seed user | dev mode only |
| `/auth/logout` | POST | Clear session | optional |
| `/api/me` | GET | Current user's profile | session |
| `/api/me` | PATCH | Update display name, skills, topic, AI choice, timezone | session |
| `/api/deck` | GET | Other users (currently ordered by last-active) | session |

### Matching + lobby (W2a)

| Endpoint | Method | Purpose |
|---|---|---|
| `POST /api/match/queue` | Enters the match queue with the given preferences. |
| `DELETE /api/match/queue` | Leaves the queue. |
| `GET /api/match/status` | Returns `{in_queue, queue_size, waited_seconds, pending_lobby_id}`. |
| `GET /api/lobbies/:id` | Lobby view (id, topic, status, seats[], room_id). |
| `POST /api/lobbies/:id/join` | Adds the caller as a guest seat. |
| `POST /api/lobbies/:id/leave` | Removes the caller's seat. |
| `POST /api/lobbies/:id/vote` | Body `{vote: "accept" \| "skip"}`. May finalise the lobby. |

Lobby status transitions: `negotiating` (4 seats, voting) → `matched`
(all 4 accepted → room created) or `closed` (anyone skipped, survivors
re-queued).

### Room (W2b)

| Endpoint | Method | Purpose |
|---|---|---|
| `GET /api/rooms/:id/events` | Last ~500 events (replay on join). |
| `GET /api/rooms/:id/ws` | WebSocket upgrade. Receives backlog, then live fan-out. |
| `POST /api/rooms/:id/ai` | Body `{}`. Asks the user's AI for an observation. |

WebSocket message types: `chat`, `canvas.put`, `ai.thinking`, `ai.done`,
`system.peer_joined`, `system.peer_left`. Payload is JSON; the client
sends `{kind, payload}` and the server broadcasts the same shape.

### Test-only

| Endpoint | Method | Purpose |
|---|---|---|
| `POST /api/_test/sweep` | Runs the matching engine once. Gated on `DEV_MODE=1`. |

Sessions are 32-byte random tokens in an HTTP-only `SameSite=Lax` cookie.
Dev-mode login creates a user in the DB on first use; subsequent calls
hit the same row.

---

## Quick start (dev mode, no GitHub required)

```bash
# from repo root
cd codematch-server

# Build
cargo build

# Run — dev mode auto-seeds 6 fake users, refuses to bind on non-loopback
DEV_MODE=1 \
  HOST=127.0.0.1 \
  PORT=18081 \
  DATABASE_URL="sqlite://codematch.db?mode=rwc" \
  cargo run
```

You should see:

```
INFO codematch_server: starting codematch-server host=127.0.0.1 port=18081 dev_mode=true github=false
INFO codematch_server::seed: dev seed: inserted 6 users
INFO codematch_server: listening addr=127.0.0.1:18081
```

The default DB path is `codematch.db` in the working directory. Delete
the file (or `mavis-trash codematch.db`) to start over.

### Smoke test

```bash
# In another shell
COOKIE=$(curl -s -D - 'http://127.0.0.1:18081/auth/dev-login?as=you' \
  | grep -i 'set-cookie' | sed -E 's/.*cm_session=([^;]+).*/\1/' \
  | tr -d '\r\n')

curl -s -b "cm_session=$COOKIE" http://127.0.0.1:18081/api/me
curl -s -b "cm_session=$COOKIE" http://127.0.0.1:18081/api/deck
```

You should get back the user row and a list of 6 seeded candidates
(Maya, Raj, 林夏, Sam, Ana, Keita).

### Open the prototype

The prototype at `../codematch-prototype/` reads the API base from
`<meta name="api-base" content="…">` and defaults to
`http://127.0.0.1:18081`. So:

```bash
# Terminal A
cd ../codematch-prototype
python3 -m http.server 18881

# Terminal B — open in browser
open http://127.0.0.1:18881/
```

The landing page detects dev mode and shows a "Sign in (dev) as @you"
button. Click it → profile screen (auto-pre-filled from /api/me) → swipe
deck (loaded from /api/deck). Yes-swipe 3 cards → match screen → room.

---

## Quick start (real GitHub OAuth)

1. Register an OAuth App at <https://github.com/settings/developers>
   - **Homepage URL**: `http://127.0.0.1:18081` (or your public host)
   - **Authorization callback URL**: `http://127.0.0.1:18081/auth/github/callback`
2. Copy the client ID and secret.
3. Run with the credentials in env:

```bash
GITHUB_CLIENT_ID=xxx \
GITHUB_CLIENT_SECRET=yyy \
PUBLIC_URL="http://127.0.0.1:18081" \
HOST=127.0.0.1 \
PORT=18081 \
DATABASE_URL="sqlite://codematch.db?mode=rwc" \
cargo run
```

The landing page will show a single "Continue with GitHub" button. After
authorising, GitHub redirects back to `/auth/github/callback` which
exchanges the code for an access token, upserts the user, and sets the
session cookie.

> For production: bind to TLS, set `PUBLIC_URL=https://...`, and the
> session cookie will set `Secure` automatically.

---

## Architecture

```
src/
├── main.rs        — entry, router wiring, server boot
├── config.rs      — env-driven config + safety checks (dev mode refuses non-loopback)
├── error.rs       — AppError → IntoResponse, single source of HTTP error shape
├── models.rs      — User (DB), UserPublic (API), GitHubUser, request DTOs
├── db.rs          — sqlx queries, schema apply, upsert_github_user, deck_for
├── auth.rs        — session token, cookie helpers, GitHub OAuth flow, AppState
├── handlers.rs    — route handlers
└── seed.rs        — dev-only seed: 6 users that mirror the prototype's mock deck
```

### Key design choices

- **One `User` struct, two consumers.** Same row reads as `User` from the
  DB and as `UserPublic` on the wire. `UserPublic` hides the internal
  `is_dev_seed` flag and parses the JSON-encoded `skills` field into a
  `Vec<String>`. Keeping the row shape shared means the only "drift"
  surface is the conversion, not the field set.
- **OAuth state is in a cookie, not a server-side store.** We use the
  random state param to defeat CSRF, then compare it to the cookie the
  request brought. No session DB lookup needed for the callback.
- **`/auth/dev-login` refuses to load unless `DEV_MODE=1`, and `Config::from_env`
  refuses to start in `DEV_MODE=1` on a non-loopback host.** Belt and
  suspenders — there's no path that lets dev-login be reachable on the
  public internet.
- **Schema applied inline.** The migrations file is a single SQL string
  `include_str!`'d at compile time and executed on startup. It uses
  `CREATE TABLE IF NOT EXISTS`, so re-runs are safe. When the schema
  grows past one file we'll switch to `sqlx::migrate!()`.

---

## What's *not* in this build

These are deliberate cuts so W1+W2 stays shippable in a focused session:

- **Voice / WebRTC** — W3
- **Per-user BYOK AI key storage** — currently one server-wide
  `OPENAI_API_KEY`. The room-endpoint signature is already per-user so
  the swap-in is small
- **AI-to-AI conversation** — AIs in the room can nudge the human
  thread, but they don't talk to each other
- **Recordings** — the asciinema path is in the prototype but not wired
  to a real backend
- **Streaming AI responses** — we wait for the full model reply before
  broadcasting `ai.done`. Switching to SSE on the upstream is a
  one-evening change
- **Rate limiting / abuse** — out of scope for the prototype

The endpoints above are stable — they won't be renamed or moved in W3
without a deprecation period.

---

## Environment variables

| Variable | Default | Purpose |
|---|---|---|
| `HOST` | `127.0.0.1` | Bind host. `DEV_MODE=1` refuses non-loopback. |
| `PORT` | `8080` | Bind port. |
| `DATABASE_URL` | `sqlite://codematch.db` | SQLite path. Use `?mode=rwc` if the file doesn't exist yet. |
| `DEV_MODE` | `0` | When `1`, enables `/auth/dev-login` + the `/api/_test/sweep` test hook. |
| `GITHUB_CLIENT_ID` | — | OAuth App client id. Required unless `DEV_MODE=1`. |
| `GITHUB_CLIENT_SECRET` | — | OAuth App client secret. Required unless `DEV_MODE=1`. |
| `PUBLIC_URL` | `http://$HOST:$PORT` | Public URL of the running app, used to build the OAuth callback. |
| `SESSION_TTL_HOURS` | `720` (30d) | Session cookie lifetime. |
| `SESSION_COOKIE_NAME` | `cm_session` | Cookie name. |
| `OPENAI_API_KEY` | — | W2d: enables the room AI proxy. OpenAI-compatible. |
| `OPENAI_BASE_URL` | `https://api.openai.com/v1` | Override for compatible APIs (DeepSeek, local llama, etc.). |
| `OPENAI_MODEL` | `gpt-4o-mini` | Model name to send. |

---

## Tests

There are no unit tests yet (the smoke-test curl recipe above is the
fixture). The plan is to add an `in-process axum test` for the happy
path of every handler, using a temp SQLite file and a mocked
`reqwest::Client` for the GitHub path. Easy follow-up; not blocking W1.

---

## License

MIT.
