---
phase: 02-auth
plan: 01
subsystem: auth
tags: [jwt, argon2, sqlx, postgres, leptos, server-functions, rust]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: "users table, Cargo.toml with ssr-gated dependencies, main.rs with Axum router"

provides:
  - "username column on users table with unique index"
  - "register server function: argon2 hash + INSERT + auto-login JWT"
  - "login server function: argon2 verify + JWT sign with generic error"
  - "validate_token server function: JWT decode returning AuthUser"
  - "AuthUser{id: i32, username: String} and Claims types"
  - "PgPool provided via leptos_routes_with_context (not Axum state)"
  - "sqlx offline query cache for auth queries"

affects:
  - 02-auth plan 02 (UI components that call these server functions)
  - any future plan using auth context

# Tech tracking
tech-stack:
  added:
    - "jsonwebtoken 10.3.0 with rust_crypto feature (deterministic provider selection)"
    - "argon2 0.5.3 with spawn_blocking for CPU-bound hashing"
    - "web-sys 0.3 for hydrate feature (localStorage access, used in plan 02)"
  patterns:
    - "Server functions use use_context::<PgPool>() — not function parameters"
    - "PgPool injected via leptos_routes_with_context at main.rs router level"
    - "CPU-bound ops (argon2) wrapped in tokio::task::spawn_blocking"
    - "Generic error for wrong-password and not-found (prevents account enumeration)"
    - "username derived from email local-part at registration"

key-files:
  created:
    - "src/server/auth/mod.rs"
    - "src/server/auth/handlers.rs"
    - "migrations/20240101000004_add_username_to_users.sql"
    - ".sqlx/query-21c1e74a95bc47b7f5e97771e458e36a5d98a26d78046347261191752ca012ef.json"
    - ".sqlx/query-dca6d8f6994b9967ef337ea1a7470ea214b02c6e3167d581362073b9f34d22c1.json"
  modified:
    - "src/main.rs"
    - "src/server/mod.rs"
    - "Cargo.toml"

key-decisions:
  - "jsonwebtoken rust_crypto feature required — no CryptoProvider auto-detection without explicit feature flag"
  - "username derived from email local-part (3-field form, not 4-field)"
  - "JWT TTL set to 7 days (as proposed in CONTEXT.md)"
  - "PgPool provided via leptos_routes_with_context, not axum State — required for server function use_context access"
  - "Unit tests cover cryptographic logic only — integration tests with PgPool deferred to later phase"

patterns-established:
  - "Auth deviation: argon2 password_hash re-exports rand_core::OsRng — use that instead of adding rand_core directly"
  - "sqlx macros require SQLX_OFFLINE=false cargo sqlx prepare --workspace -- --features ssr to generate cache"

requirements-completed: [AUTH-01, AUTH-02, AUTH-03]

# Metrics
duration: 7min
completed: 2026-03-12
---

# Phase 2 Plan 01: Auth Server Functions Summary

**register/login/validate_token Leptos server functions with argon2 hashing and HS256 JWT via jsonwebtoken, wired to PgPool via leptos_routes_with_context**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-12T09:34:38Z
- **Completed:** 2026-03-12T09:41:28Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- Username migration applied — users table now has username column with unique index
- Three server functions fully implemented: register (argon2 hash + auto-login), login (generic error for security), validate_token (JWT decode)
- PgPool switched from Axum state pattern to Leptos context via leptos_routes_with_context
- 5 unit tests passing: argon2 hash/verify, JWT encode/decode, JWT expiry, invalid JWT, username derivation
- SQLx offline cache generated for CI/offline builds

## Task Commits

1. **Task 1: Add username migration, wire PgPool, add web-sys** - `a3eda4f` (feat)
2. **Task 2: Implement auth server functions with unit tests** - `2933641` (feat)
3. **Task 3: Update sqlx offline query cache** - `c4488a4` (chore)

## Files Created/Modified

- `migrations/20240101000004_add_username_to_users.sql` - ALTER TABLE adding username column + unique index
- `src/server/auth/mod.rs` - Auth module declaration and re-exports
- `src/server/auth/handlers.rs` - register, login, validate_token + AuthUser + Claims types + unit tests
- `src/server/mod.rs` - Added `pub mod auth;`
- `src/main.rs` - Added PgPool connection, switched to leptos_routes_with_context
- `Cargo.toml` - Added rust_crypto feature to jsonwebtoken, added web-sys for hydrate
- `.sqlx/` - Two query metadata files for offline compilation

## Decisions Made

- **jsonwebtoken rust_crypto feature**: jsonwebtoken 10.3.0 requires explicit CryptoProvider selection — adding `features = ["rust_crypto"]` to the dependency is the correct fix (not calling install_default() at runtime).
- **PgPool context pattern**: Server functions cannot access Axum State directly; they use use_context which requires leptos_routes_with_context at the router level.
- **SQLX_OFFLINE=true in .env**: cargo sqlx prepare must be run with SQLX_OFFLINE=false and --features ssr flag to capture queries inside #[leptos::server] macros.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added rust_crypto feature to jsonwebtoken**
- **Found during:** Task 2 (implementing server functions and running tests)
- **Issue:** jsonwebtoken 10.3.0 panics at runtime with "Could not automatically determine the process-level CryptoProvider" without explicit feature selection
- **Fix:** Added `features = ["rust_crypto"]` to jsonwebtoken in Cargo.toml
- **Files modified:** Cargo.toml
- **Verification:** All 5 unit tests pass after fix
- **Committed in:** 2933641 (Task 2 commit)

**2. [Rule 3 - Blocking] Ran sqlx prepare before task 3 to unblock compilation**
- **Found during:** Task 2 (implementing sqlx::query! macros in server functions)
- **Issue:** SQLX_OFFLINE=true in .env but .sqlx cache was empty — sqlx macros failed to compile
- **Fix:** Ran `SQLX_OFFLINE=false cargo sqlx prepare --workspace -- --features ssr` to generate cache, then continued task 2
- **Files modified:** .sqlx/ directory
- **Verification:** `cargo test --lib --features ssr server::auth` passes with all 5 tests
- **Committed in:** c4488a4 (Task 3 commit)

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Both necessary for compilation and test execution. No scope creep.

## Issues Encountered

None beyond the auto-fixed deviations above.

## User Setup Required

None - no external service configuration required. JWT_SECRET is already in .env.

## Next Phase Readiness

- All server functions exported and ready for Leptos UI components in Plan 02
- AuthUser type available for UI state signal
- PgPool context pattern established for all subsequent server functions
- No blockers for Plan 02

---
*Phase: 02-auth*
*Completed: 2026-03-12*
