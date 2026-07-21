# pair-terminal

Terminal pair programming for two developers. Share your terminal session with a partner over an encrypted WebSocket connection.

## Architecture

```
pair-terminal/
├── crates/
│   ├── pair-common/    # Shared types, protocol, crypto, recording
│   ├── pair-server/    # WebSocket relay server (axum + SQLite)
│   └── pair-client/    # CLI client (pair share/join/match/replay)
├── Cargo.toml          # Workspace root
├── README.md           # ← you are here
├── rustfmt.toml        # Formatting config
└── clippy.toml         # Clippy lints
```

### Crates

- **pair-common** — Protocol definitions (`ClientMessage` / `ServerMessage`),
  AES-128-GCM encryption with key rotation, asciinema v2 session recording,
  shared types (`PairMode`, `SkillLevel`, `UserId`, `TerminalId`).
- **pair-server** — Axum-based relay server with WebSocket handler, session
  management (host/guest model), SQLite persistence, and a match queue with
  skill-based scoring.
- **pair-client** — CLI binary (`pair`) with subcommands: `share`, `join`,
  `match`, `replay`.

## Quick start

### Build

```bash
cargo build --all
```

### Run the server

```bash
cargo run -p pair-server -- --host 0.0.0.0 --port 8080
```

### Share a terminal session

```bash
# Share your terminal as the host
cargo run -p pair -- share --mode collab

# Share with recording enabled
cargo run -p pair -- share --record

# Share a specific command (e.g. an interactive vim session)
cargo run -p pair -- share -- vim
```

After running, the host prints a `pair://terminal_id#bootstrap_key` URL.
Send it to your partner out-of-band (chat, email, etc.) — never trust the
relay with that URL.

### Join a session

```bash
cargo run -p pair -- join "pair://session_id#bootstrap_key"
```

### Get auto-matched

```bash
cargo run -p pair -- match --lang rust python --skill intermediate --mode collab
```

### Replay a recording

```bash
cargo run -p pair -- replay recording.cast
```

## Features

- **End-to-end encryption** — AES-128-GCM with automatic key rotation
  every 1M messages. The relay server only sees opaque ciphertext.
- **Three pair modes** — Driver (host only), Navigator (guest only),
  Collaborative (both).
- **Session recording** — asciinema v2 format; replayable from the CLI.
- **Skill-based matchmaking** — Jaccard language overlap + skill-level
  compatibility, with a background task on the server.
- **TUI controls** — Ctrl+T for help, chat input, mode switching.
- **Connection limits** — server caps at 1,000 concurrent connections,
  200 active terminals, 50 guests per terminal.

## Configuration

### Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `pair_server=debug` | tracing-subscriber filter |
| `PAIR_SERVER_HOST` | `0.0.0.0` | bind host |
| `PAIR_SERVER_PORT` | `8080` | bind port |
| `PAIR_SERVER_DB` | `pair.db` | SQLite path |

### CLI flags

```text
pair-server --host <HOST> --port <PORT> --database <PATH>

pair share  --mode <driver|navigator|collab> [--record] [-- <command>...]
pair join   <pair-url>
pair match  --lang <lang>... --skill <beginner|intermediate|expert> [--mode <mode>]
pair replay <recording.cast>
```

## Protocol

Messages are JSON-encoded with a `type` discriminator and a `payload`
object. See `crates/pair-common/src/protocol.rs` for the canonical
definitions. The handshake flow is:

```text
Client                            Server
  |  Handshake(user_id, role, ...)  |
  | ----------------------------->  |
  |                                 |
  |  HandshakeOk(session_id, ...)   |
  | <-----------------------------  |
  |                                 |
  |  AesKeys(b64_output_key, ...)   |   ← encrypted with bootstrap key
  | ----------------------------->  |
  |                                 |
  |  KeyInput(data, encrypted)      |   ← all subsequent traffic
  | <-------------------------->    |     is AES-GCM encrypted
```

The bootstrap key in the `pair://` URL is the only secret that must be
shared out-of-band; everything else is derived from it.

## Testing

```bash
cargo test --all            # 31 tests across 3 crates
cargo clippy --all --all-targets -- -D warnings
cargo fmt --all -- --check
```

## Security

- The relay server cannot read session contents (E2E encryption).
- No plaintext fallback exists in the client — input is silently dropped
  until keys are established.
- Connection caps prevent trivial DoS.
- A `rustsec/audit-check` job runs on CI.

For the original May 2026 audit, see
[`EVALUATION_REPORT.md`](./EVALUATION_REPORT.md).
