---
phase: 2
slug: auth
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-12
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `#[cfg(test)]` modules |
| **Config file** | none — uses `cargo test` |
| **Quick run command** | `cargo test --features ssr` |
| **Full suite command** | `cargo test --features ssr` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features ssr`
- **After every plan wave:** Run `cargo test --features ssr`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 2-01-01 | 01 | 1 | AUTH-01 | unit | `cargo test --features ssr auth::tests::register_creates_user` | ❌ W0 | ⬜ pending |
| 2-01-02 | 01 | 1 | AUTH-01 | unit | `cargo test --features ssr auth::tests::password_is_hashed` | ❌ W0 | ⬜ pending |
| 2-01-03 | 01 | 1 | AUTH-02 | unit | `cargo test --features ssr auth::tests::login_valid_creds` | ❌ W0 | ⬜ pending |
| 2-01-04 | 01 | 1 | AUTH-02 | unit | `cargo test --features ssr auth::tests::login_same_error_on_failure` | ❌ W0 | ⬜ pending |
| 2-01-05 | 01 | 1 | AUTH-03 | unit | `cargo test --features ssr auth::tests::validate_token_ok` | ❌ W0 | ⬜ pending |
| 2-01-06 | 01 | 1 | AUTH-03 | unit | `cargo test --features ssr auth::tests::validate_token_expired` | ❌ W0 | ⬜ pending |
| 2-01-07 | 01 | 1 | AUTH-04 | manual | n/a — verify localStorage cleared + redirect in browser | n/a | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/server/auth/handlers.rs` — test module `#[cfg(test)] mod tests { ... }` covering AUTH-01 through AUTH-03 logic units
- [ ] No framework install needed — `cargo test` is built-in

*Existing infrastructure covers framework requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Logout clears localStorage and redirects to /login | AUTH-04 | Client-side localStorage + navigation — no server round-trip to test | 1. Log in, 2. Click logout, 3. Verify redirect to /login, 4. Check localStorage cleared, 5. Refresh — should stay on /login |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
