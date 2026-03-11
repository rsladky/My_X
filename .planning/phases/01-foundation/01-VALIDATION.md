---
phase: 1
slug: foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-11
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in tests + Tokio (async runtime) |
| **Config file** | none — Wave 0 installs |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test && SQLX_OFFLINE=true cargo build --release` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test && SQLX_OFFLINE=true cargo build --release`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 1-01-01 | 01 | 1 | scaffolding | smoke | `timeout 10 cargo leptos watch & sleep 5 && curl -s http://localhost:3000` | ❌ W0 | ⬜ pending |
| 1-01-02 | 01 | 1 | scaffolding | integration | `sqlx migrate run` | ❌ W0 | ⬜ pending |
| 1-01-03 | 01 | 1 | scaffolding | build | `SQLX_OFFLINE=true cargo build --release` | ❌ W0 | ⬜ pending |
| 1-01-04 | 01 | 1 | scaffolding | unit | `cargo test server::error --lib` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/integration_smoke.rs` — Start `cargo leptos watch`, curl localhost:3000, verify HTML response
- [ ] `tests/migration_test.rs` — Run `sqlx migrate run` against test DB, verify all tables exist with correct schema
- [ ] `src/server/error.rs` — Implement `AppError` enum and `IntoResponse` trait, add `#[test]` for JSON serialization
- [ ] `.env.example` — Document expected env vars (DATABASE_URL, SQLX_OFFLINE) for contributors
- [ ] Database test fixture — Create a `sqlx::Pool` for integration tests (shared between test cases)

*Existing infrastructure covers all phase requirements: No — Wave 0 installs are required since this is the initial scaffold.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `cargo leptos watch` starts without error | Scaffolding SC-1 | Requires live process observation | Run `cargo leptos watch`, verify no panic/error output, check localhost renders HTML in browser |
| Bad handler returns JSON (not panic) | Scaffolding SC-4 | Requires HTTP client inspection | Hit bad route with curl, verify `{"error": "...", "message": "..."}` JSON response |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
