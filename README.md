# lonely-coder

> Looking for a coding partner, one terminal at a time.

This repository is the home of **`pair-terminal`** — a CLI tool for live,
end-to-end-encrypted terminal pair programming, plus a relay server. The
original "Wuma / 无码" social-platform idea from 2013–2015 still lives in
[`HISTORY.md`](./HISTORY.md) and the `archive/` directory, kept for
reference only.

## What's here

| Path | What it is | Status |
|------|------------|--------|
| [`pair-terminal/`](./pair-terminal/) | Rust workspace for terminal pair programming (the live project) | **Active** |
| [`archive/`](./archive/) | Old PHP frameworks from the Wuma/无码 era (LaneWeChat, WeiEngine, WeiPHP) | Frozen — historical |
| [`analysis-report.md`](./analysis-report.md) | Security & quality autopsy of the archived PHP code (2026-04) | Frozen — historical |
| [`ideas.md`](./ideas.md) | Long-form brainstorm notes | Reference only |
| [`doc/`](./doc/) | Placeholder design docs from 2014 (not real documentation) | Stale |
| [`HISTORY.md`](./HISTORY.md) | Origin story of this repository (Wuma → pair-terminal) | Living |

## Quick start — `pair-terminal`

```bash
cd pair-terminal

# Build everything
cargo build --all

# Run the relay server (axum + SQLite, defaults to 0.0.0.0:8080)
cargo run -p pair-server

# In another terminal: share your session
cargo run -p pair -- share --mode collab

# On a partner's machine: join using the pair:// URL
cargo run -p pair -- join "pair://<terminal_id>#<bootstrap_key>"

# Or: get auto-matched with someone looking for the same stack
cargo run -p pair -- match --lang rust --skill intermediate
```

The relay server uses `pair.db` in the current directory by default. The
client and server talk over WebSockets; all PTY traffic is end-to-end
encrypted with AES-128-GCM (key bootstrap lives in the `pair://` URL).

## Features

- **End-to-end encryption** — AES-128-GCM with automatic 1M-message key rotation
- **Three pair modes** — Driver (host only), Navigator (guest only), Collaborative (both)
- **Session recording** — asciinema v2 format, replay with `pair replay`
- **Skill-based matchmaking** — Jaccard language overlap + skill-level compatibility
- **TUI status bar** — Ctrl+T help, chat, mode switching
- **Connection limits** — server caps at 1,000 concurrent connections

Full docs, architecture, and protocol details live in
[`pair-terminal/README.md`](./pair-terminal/README.md).

## Development

```bash
cd pair-terminal

cargo fmt --all -- --check        # formatting (CI enforces)
cargo clippy --all --all-targets -- -D warnings   # lints (CI enforces)
cargo test --all                  # 31 tests across 3 crates
cargo build --all                 # release build
```

CI runs the same four checks on every push (see
[`.github/workflows/ci.yml`](./.github/workflows/ci.yml)) plus a
`rustsec/audit-check` job.

## License

MIT. See `pair-terminal/Cargo.toml` for the canonical declaration.

---

For the full backstory of how this repo got from a 2013 WeChat social
platform to a 2026 Rust terminal pair-programming tool, see
[`HISTORY.md`](./HISTORY.md).
