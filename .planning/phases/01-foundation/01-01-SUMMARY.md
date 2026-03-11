---
phase: 01-foundation
plan: 01
subsystem: infra
tags: [leptos, axum, sqlx, tokio, argon2, jsonwebtoken, rust, wasm, ssr]

# Dependency graph
requires: []
provides:
  - Runnable Leptos 0.8.0 + Axum 0.8.0 SSR project scaffold at localhost:3000
  - Cargo.toml with all Phase 1-5 dependencies declared and feature-gated
  - SSR/hydrate feature split for server vs WASM compilation
  - cargo build exits 0 (verified)
affects: [02-foundation, 03-auth, 04-feed, 05-social]

# Tech tracking
tech-stack:
  added:
    - leptos 0.8.0 (full-stack reactive framework, SSR + WASM hydration)
    - leptos_axum 0.8.0 (Axum integration)
    - leptos_meta 0.8.0 (document head management)
    - leptos_router 0.8.0 (client-side routing)
    - axum 0.8.0 with macros feature (HTTP server)
    - tokio 1.x with rt-multi-thread (async runtime)
    - sqlx 0.8.6 with postgres, chrono, uuid, runtime-tokio-rustls (database)
    - jsonwebtoken 10.3.0 (JWT auth)
    - argon2 0.5.3 (password hashing)
    - serde 1.x + serde_json (serialization)
    - chrono 0.4 + uuid 1.x (data types)
    - thiserror 1.x (error handling)
    - tracing 0.1 + tracing-subscriber 0.3 (structured logging)
    - dotenvy 0.15 (env var management)
    - rand 0.8 (randomness)
    - wasm-bindgen 0.2.106 (WASM JS interop)
    - console_error_panic_hook 0.1 (WASM panic formatting)
  patterns:
    - Feature flags split: ssr (server) vs hydrate (WASM client)
    - Server-only crates (sqlx, jsonwebtoken, argon2) feature-gated under [ssr]
    - cargo-generate used as scaffold mechanism (cargo leptos new is interactive-only)

key-files:
  created:
    - Cargo.toml
    - Cargo.lock
    - src/main.rs
    - src/app.rs
    - src/lib.rs
    - style/main.scss
    - public/favicon.ico
    - .gitignore
    - end2end/tests/example.spec.ts
  modified: []

key-decisions:
  - "Scaffolded via cargo-generate instead of cargo leptos new (interactive terminal required)"
  - "Template uses src/main.rs not src/bin/server/main.rs — kept template structure"
  - "Server-side crates feature-gated under [ssr] to avoid WASM compilation errors"
  - "Cargo.lock committed (executable project, not library)"
  - ".env added to .gitignore for secret safety"

patterns-established:
  - "Feature-gated dependencies: all server-only crates under [features.ssr] to compile correctly to WASM"
  - "SSR/hydrate split: lib compiles twice (server + WASM); main.rs is server entry point"

requirements-completed: []

# Metrics
duration: 15min
completed: 2026-03-11
---

# Phase 1 Plan 01: Scaffold Leptos+Axum Foundation Summary

**Leptos 0.8.0 + Axum 0.8.0 full-stack scaffold with all Phase 1-5 crates (sqlx, jsonwebtoken, argon2) declared and feature-gated for SSR/WASM compilation — cargo build exits 0.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-03-11T19:00:20Z
- **Completed:** 2026-03-11T19:15:33Z
- **Tasks:** 1 of 1 auto tasks complete (checkpoint pending human verify)
- **Files modified:** 16

## Accomplishments
- Scaffolded start-axum template via cargo-generate (Leptos 0.8.0 + Axum 0.8.0)
- Renamed package from `my_x_scaffold` to `my_x`
- Added all Phase 1-5 dependencies to Cargo.toml with correct feature gating
- `cargo build` exits 0 with no errors
- .gitignore updated: Cargo.lock tracked (executable), .env excluded (security)

## Task Commits

Each task was committed atomically:

1. **Task 1: Scaffold project with cargo leptos new and add all dependencies** - `c05ca5e` (feat)

**Plan metadata:** TBD (created after checkpoint approval)

## Files Created/Modified
- `Cargo.toml` - Package renamed to my_x; all Phase 1-5 deps declared with ssr/hydrate feature gating
- `Cargo.lock` - Locked dependency tree (committed for executable)
- `src/main.rs` - Axum server entry point with #[tokio::main] and LeptosRoutes
- `src/app.rs` - Root App component, shell(), HomePage; CSS reference updated to my_x.css
- `src/lib.rs` - Leptos library root with WASM hydrate() entry point
- `style/main.scss` - Template SCSS stylesheet
- `.gitignore` - Template .gitignore + .env exclusion, Cargo.lock tracked

## Decisions Made
- **cargo-generate instead of cargo leptos new**: `cargo leptos new` requires an interactive terminal (TTY). Used `cargo generate` directly with the same template URL. Same result.
- **src/main.rs, not src/bin/server/main.rs**: The actual start-axum template uses `src/main.rs` as the binary entry point, not `src/bin/server/main.rs` as referenced in the plan. Kept the actual template structure.
- **Server-only crates feature-gated under [ssr]**: sqlx, jsonwebtoken, argon2, rand, tracing, tracing-subscriber, dotenvy all gated under `[features.ssr]` to prevent WASM compilation errors. serde, serde_json, chrono, uuid, thiserror remain ungated (used on both sides).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Scaffold mechanism: cargo leptos new requires interactive TTY**
- **Found during:** Task 1 (scaffold)
- **Issue:** `cargo leptos new` fails with "not a terminal" error in non-interactive shell. Also, macOS case-insensitive filesystem causes `my_x` to collide with existing `My_X/` directory.
- **Fix:** Used `cargo generate --git https://github.com/leptos-rs/start-axum --name my_x_scaffold --silent` to /tmp, then rsync'd to project root.
- **Files modified:** All scaffold files (same content as cargo leptos new would produce)
- **Verification:** cargo build exits 0
- **Committed in:** c05ca5e

**2. [Rule 1 - Bug] Template uses src/main.rs not src/bin/server/main.rs**
- **Found during:** Task 1 (scaffold review)
- **Issue:** Plan's `files_modified` list references `src/bin/server/main.rs`, but the actual start-axum template creates `src/main.rs` as the server binary entry point.
- **Fix:** Kept template's actual structure (src/main.rs). Updated reference from `my_x_scaffold` to `my_x` in the use statement.
- **Files modified:** src/main.rs
- **Verification:** cargo build exits 0; server entry point compiles correctly
- **Committed in:** c05ca5e

---

**Total deviations:** 2 auto-fixed (both Rule 1 - template/tooling bugs)
**Impact on plan:** Both fixes address tooling constraints. Functional outcome is identical to the plan's intent. No scope creep.

## Issues Encountered
- Leptos.toml does not exist as a separate file in the start-axum template — configuration lives in `[package.metadata.leptos]` section of Cargo.toml (consistent with cargo-leptos 0.3.x standards).

## User Setup Required
None - no external service configuration required for Task 1. The human-verify checkpoint will confirm `cargo leptos watch` starts and serves HTML at localhost:3000.

## Next Phase Readiness
- `cargo build` passes — foundation compiles cleanly
- All Phase 1-5 dependency versions locked in Cargo.lock
- Awaiting human verification that `cargo leptos watch` starts and localhost:3000 serves HTML (checkpoint pending)
- After checkpoint approval: ready for Phase 1 Plan 02 (database migrations)

---
*Phase: 01-foundation*
*Completed: 2026-03-11*
