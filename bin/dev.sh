#!/usr/bin/env bash
# dev.sh — boot the CodeMatch MVP in dev mode with mock data.
#
# What this does:
#   1. (re)build the server binary if it's missing or stale
#   2. trash any old SQLite DB so the dev seed runs fresh
#   3. start the server in the background with PROTOTYPE_DIR + DEV_MODE
#   4. pre-enqueue 3 of the dev-seed users so the matching screen has
#      visible queue state when you open the prototype
#   5. tail the log so you can see what's happening
#
# Usage:
#   ./bin/dev.sh           # boot + pre-enqueue + tail logs
#   ./bin/dev.sh --reset   # also kill any existing server first
#   ./bin/dev.sh --no-tail # boot + pre-enqueue, then exit (server keeps running)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SERVER_DIR="$ROOT/codematch-server"
PROTOTYPE_DIR="$ROOT/codematch-prototype"
DB="$SERVER_DIR/codematch.db"
BIN="$SERVER_DIR/target/debug/codematch-server"
LOG="${CODEMATCH_LOG:-/tmp/codematch-server.log}"
HOST="${CODEMATCH_HOST:-127.0.0.1}"
PORT="${CODEMATCH_PORT:-18081}"

case "${1:-}" in
  --reset)
    echo "→ killing any existing codematch-server"
    pkill -f "target/debug/codematch-server" || true
    sleep 1
    ;;
esac

# Refuse to double-start. If something is already listening on the port,
# tell the user how to reset instead of leaving them with a confusing
# "address in use" error.
if lsof -iTCP:"$PORT" -sTCP:LISTEN -n -P >/dev/null 2>&1; then
  echo "port $PORT is already in use — something is already serving."
  echo "  → tail -f $LOG            # see what it is"
  echo "  → $0 --reset             # kill it and boot fresh"
  exit 1
fi

# 1. build if the binary is missing
if [ ! -x "$BIN" ]; then
  echo "→ building codematch-server (first run; will take a minute)"
  (cd "$SERVER_DIR" && cargo build)
fi

# 2. fresh DB so the dev seed re-inserts 6 mock users
if [ -f "$DB" ]; then
  echo "→ trashing old DB at $DB"
  mavis-trash "$DB" 2>/dev/null || rm -f "$DB"
fi
touch "$DB"

# 3. start server
echo "→ starting codematch-server on $HOST:$PORT (log: $LOG)"
(cd "$SERVER_DIR" \
  && DATABASE_URL="sqlite:codematch.db" \
     PROTOTYPE_DIR="$PROTOTYPE_DIR" \
     DEV_MODE=1 \
     HOST="$HOST" PORT="$PORT" \
     GITHUB_CLIENT_ID="${GITHUB_CLIENT_ID:-test_dummy}" \
     GITHUB_CLIENT_SECRET="${GITHUB_CLIENT_SECRET:-test_dummy}" \
     GITHUB_REDIRECT_URI="http://$HOST:$PORT/auth/github/callback" \
     SESSION_SECRET="${SESSION_SECRET:-dev_session_secret_for_local_only}" \
     RUST_LOG="${RUST_LOG:-info,sqlx=warn}" \
     nohup "$BIN" > "$LOG" 2>&1 & disown
)

# wait for /health
for i in 1 2 3 4 5 6 7 8 9 10; do
  if curl -fsS "http://$HOST:$PORT/health" >/dev/null 2>&1; then
    break
  fi
  sleep 0.3
done
curl -fsS "http://$HOST:$PORT/health" >/dev/null || {
  echo "server didn't come up; last log lines:"
  tail -20 "$LOG"
  exit 1
}
echo "  ✓ health ok"

# 4. pre-enqueue 3 mock users so the matching screen looks alive
for h in maya raj lin; do
  TOK=$(curl -fsS -c - "http://$HOST:$PORT/auth/dev-login?as=$h" \
          | awk '$6 == "cm_session" {print $NF}')
  curl -fsS -b "cm_session=$TOK" -H "Content-Type: application/json" \
    -X POST "http://$HOST:$PORT/api/match/queue" \
    -d '{"languages":["rust"],"topic":"看 AI 在 4 人脑暴里到底有啥用","timezone":"UTC+8"}' >/dev/null
done
echo "  ✓ pre-enqueued 3 mock users (maya, raj, lin) into the queue"

cat <<EOF

╭─────────────────────────────────────────────────────────────╮
│  CodeMatch MVP is up                                        │
│                                                             │
│  Open in your browser:                                      │
│    → http://$HOST:$PORT/app/index.html
│                                                             │
│  Server log (live):                                         │
│    → tail -f $LOG
│                                                             │
│  Stop the server:                                           │
│    → pkill -f "target/debug/codematch-server"              │
╰─────────────────────────────────────────────────────────────╯
EOF

if [ "${1:-}" = "--no-tail" ]; then
  exit 0
fi

echo "→ tailing $LOG (Ctrl-C to detach; the server keeps running)"
trap 'echo "(detached — server still running in background)"' INT
tail -f "$LOG"
