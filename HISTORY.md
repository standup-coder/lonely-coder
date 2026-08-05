# History of lonely-coder

This document captures the long arc of this repository. The current
project — `pair-terminal` — is described in the root
[`README.md`](./README.md); everything below is preserved for context.

---

## Phase 1 — Wuma / 无码 (2013–2015)

The repository was originally created in January 2013 as **Wuma (无码)**,
a.k.a. *Coder Lover* / *LoveBits* / *CoderUp*: a WeChat public-account
based social platform for programmers to find coding partners.

- **README (2014)**: `webRTC + backbone.js + flask, postgresql`, with a
  backup plan of `node.js + mongoDB`.
- **`archive/web/`** contains three unmodified 2014 PHP frameworks that
  were checked in as potential foundations:
  - `lame/` — LaneWeChat v1.4 (微信 SDK, 2014-11)
  - `weengine/` — WeiEngine v0.6 (微擎, 2014-09)
  - `weiphp/` — WeiPHP 2.0 + ThinkPHP 3.2.0 + OneThink 1.0
- **`doc/`** holds six placeholder design documents whose content is the
  literal string `appsforcoder.com / Website for appsforcoder studio.`
  — there was never real documentation written.
- **`ideas.md`** (98K) is a stream-of-consciousness brainstorm that was
  actively maintained during this era.

### Why it stopped

The project never got past the planning/placeholder stage. The PHP
frameworks were copied in but never installed or modified. Mobile
directories (`mobile/android/`, `mobile/ios/`) are empty placeholders.
The final substantive commits are from 2015-08-28; afterwards the repo
only saw automated dependabot PRs (all unmerged) until 2026.

---

## Phase 2 — Autopsy (2026-04)

In April 2026 the archived PHP code was given a thorough security and
quality audit. The result is [`analysis-report.md`](./analysis-report.md):

- 17 critical, 18 high, 16 medium security issues across the three
  PHP frameworks (RCE, SQL injection, hardcoded credentials, SSL
  verification disabled, etc.)
- All three frameworks target PHP 5.3–5.6 and are completely unusable
  on any supported PHP version (7.4+ / 8.x).
- Zero tests, zero real documentation.

**Conclusion of the report**: don't try to fix the PHP code — start
fresh on a modern stack if the social-platform idea still has value.

---

## Phase 3 — pair-terminal (2026-05 → present)

The pivot: a real, working tool that lives by the same spirit (help
developers pair up) but ships as a focused terminal CLI. The
`pair-terminal/` directory was added in May 2026.

### Key dates

- **2026-05-08** — `pair-terminal/` workspace added; initial prototype
  across three crates (`pair-common`, `pair-server`, `pair-client`).
- **2026-05-19** — Full security & quality evaluation
  ([`pair-terminal/EVALUATION_REPORT.md`](./pair-terminal/EVALUATION_REPORT.md)):
  6 P0 build errors, 4 P1 security/architecture issues, 2 P2
  doc/quality issues all addressed. Score: 4.5/10 → 7.0/10.
- **2026-07-21** — Repository hygiene pass (this commit):
  - Removed 23,374 build artifacts (`pair-terminal/target/`)
  - Removed 7 files of duplicated pre-fix code (`pair-terminal/pair-terminal/`)
  - Removed 3 `.DS_Store` files
  - Removed empty `migrations/` and `docker/` directories
  - Hardened `.gitignore`
  - Fixed 4 dead-code warnings + 2 test warnings
  - Replaced root README with a current-state-first version
  - CI now passes: `cargo fmt --check`, `cargo clippy -D warnings`,
    `cargo build --all`, `cargo test --all` (31/31).
- **2026-08-04 → 2026-08-05** — `codematch-server/` and
  `codematch-prototype/` added. A second product lane for the repo,
  this time a browser-based brainstorm-matching tool (4-person squad,
  mutual-yes lobby, WebSocket room, AI proxy). The terminal
  pair-programming tool above stays as the canonical `pair-terminal/`
  workspace. W2 status:
  - **W2a matching + lobby** — queue, 4-person lobby auto-formation,
    mutual-yes vote → room create, all on real SQLite + axum.
  - **W2b canvas + WebSocket** — in-process broadcast bus, backlog
    replay, live fan-out.
  - **W2d AI proxy** — OpenAI-compatible chat completion with
    observer-role system prompt; `ai.thinking` / `ai.done` events
    fan out via the same bus.
  - 22 tests green; E2E browser walkthrough (Playwright + Chromium)
    verifies matching → lobby → room → chat-echo end-to-end with
    screenshots in `/tmp/e2e-0{1..4}-*.png`.
  - **W2c voice** — explicitly deferred; not in W2 scope.

---

## Where things stand now

- Two parallel product lanes live in the repo:
  `pair-terminal/` (the terminal pair-programming tool, W3-ish
  features shipped) and `codematch-server/` + `codematch-prototype/`
  (the brainstorm-matching product, currently mid-W2).
- The active surface area is small: `pair-terminal/`, `codematch-server/`,
  `codematch-prototype/`, `product-design/`, `.github/`, `README.md`,
  `HISTORY.md`, `LICENSE`.
- The historical layers (`archive/`, `analysis-report.md`, `ideas.md`,
  `doc/`) are kept for reference only and should not be deployed or
  extended. Anyone reviving the social-platform idea should treat
  `analysis-report.md` as the warning label — and should look at
  `codematch-*` instead, which is the modern, well-architected
  reinterpretation of the same core idea.
