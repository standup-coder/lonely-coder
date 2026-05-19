# pair-terminal

Terminal pair programming for two developers. Share your terminal session with a partner over an encrypted WebSocket connection.

## Architecture

```
pair-terminal/
├── crates/
│   ├── pair-common/    # Shared types, protocol, crypto, recording
│   ├── pair-server/    # WebSocket relay server (axum + SQLite)
│   └── pair-client/    # CLI client (pair share/join/match)
├── Cargo.toml          # Workspace root
└── README.md
```

### Crates

- **pair-common**: Protocol definitions (ClientMessage/ServerMessage), AES-128-GCM encryption with key rotation, asciinema v2 session recording, shared types (PairMode, SkillLevel, etc.)
- **pair-server**: Axum-based relay server with WebSocket handler, session management (host/guest model), SQLite persistence, and match queue with skill-based scoring
- **pair-client**: CLI binary (`pair`) with subcommands: `share`, `join`, `match`, `replay`

## Quick Start

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
# Share your terminal (host)
cargo run -p pair -- share --mode collab

# Share with recording
cargo run -p pair -- share --record

# Share a specific command
cargo run -p pair -- share -- vim
```

### Join a session

```bash
# Join using the pair:// URL provided by the host
cargo run -p pair -- join "pair://session_id#bootstrap_key"
```

### Get matched with a partner

```bash
cargo run -p pair -- match --lang rust python --skill intermediate --mode collab
```

### Replay a recording

```bash
cargo run -p pair -- replay recording.cast
```

## Features

- **E2E Encryption**: AES-128-GCM with automatic key rotation every 1M messages
- **Pair Modes**: Driver (host only), Navigator (guest only), Collaborative (both)
- **Session Recording**: asciinema v2 format with replay support
- **Matchmaking**: Skill-based matching with language preference overlap (Jaccard similarity)
- **TUI Controls**: Ctrl+T for help panel, chat, mode switching
- **Connection Limits**: Max 1000 concurrent connections, 200 terminals, 50 guests per terminal

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PAIR_SERVER` | `wss://pair.dev/ws` | WebSocket server URL |
| `PAIR_SIGNALING` | `wss://pair.dev/signal` | Signaling server URL (P2P, not yet implemented) |
| `RUST_LOG` | `pair=debug` | Log level filter |

### Server CLI

```
pair-server [OPTIONS]
    --host <HOST>        Listen address [default: 0.0.0.0]
    -p, --port <PORT>    Listen port [default: 8080]
    --database <PATH>    SQLite database path [default: pair.db]
```

## Protocol

Messages are JSON-encoded over WebSocket with tagged enums:

```json
{"type": "Handshake", "payload": {"user_id": "...", "role": "Host", ...}}
{"type": "PtyOutput", "payload": {"data": "<base64>", "encrypted": true}}
{"type": "KeyInput",  "payload": {"data": "<base64>", "encrypted": true}}
```

### Key Exchange Flow

1. Host generates session keys (bootstrap + output + input)
2. Host shares `pair://session_id#bootstrap_key` URL
3. Guest connects and sends Handshake
4. Host rotates keys and sends encrypted keys via AesKeys message
5. Guest extracts keys using bootstrap key from URL fragment
6. All subsequent communication is E2E encrypted

## Testing

```bash
cargo test --all
```

31 tests covering crypto, types, recording, protocol serialization, and match queue logic.

## Security

- Bootstrap key is in the URL fragment (never sent to server)
- AES-128-GCM with counter-based nonces prevents replay attacks
- Key rotation at 1M messages prevents nonce reuse
- Terminal dimension cap (500x500) prevents resource abuse
- Connection limiting (1000 concurrent) prevents DoS

## License

MIT
