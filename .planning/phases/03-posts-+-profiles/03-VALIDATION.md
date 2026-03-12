---
phase: 3
slug: posts-profiles
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-12
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` with `#[cfg(test)]` modules (no external test runner) |
| **Config file** | none — inline test modules only (established Phase 2 pattern) |
| **Quick run command** | `cargo test --features ssr` |
| **Full suite command** | `cargo test --features ssr` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features ssr`
- **After every plan wave:** Run `cargo test --features ssr`
- **Before `/gsd:verify-work`:** Full suite must be green + manual browser smoke test
- **Max feedback latency:** ~10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 3-W0-01 | W0 | 0 | POST-01 | unit | `cargo test --features ssr post::create_post_rejects_empty` | ❌ W0 | ⬜ pending |
| 3-W0-02 | W0 | 0 | POST-01 | unit | `cargo test --features ssr post::create_post_rejects_over_limit` | ❌ W0 | ⬜ pending |
| 3-W0-03 | W0 | 0 | POST-02 | unit | `cargo test --features ssr post::delete_post_rejects_wrong_user` | ❌ W0 | ⬜ pending |
| 3-W0-04 | W0 | 0 | POST-03 | unit | `cargo test --features ssr post::post_with_author_serde_roundtrip` | ❌ W0 | ⬜ pending |
| 3-W0-05 | W0 | 0 | POST-03 | unit | `cargo test relative_timestamp_formats` | ❌ W0 | ⬜ pending |
| 3-W0-06 | W0 | 0 | PROF-01 | unit | `cargo test --features ssr post::user_posts_order` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/server/posts/handlers.rs` — create server module with stubs for POST-01, POST-02, POST-03, PROF-01
- [ ] `src/server/posts/mod.rs` — module definition
- [ ] `PostWithAuthor` struct definition (non-ssr-gated, in a shared module)
- [ ] `relative_timestamp` utility function stub
- [ ] Inline `#[cfg(test)]` test stubs in `handlers.rs` for all 6 verification entries above

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Post appears immediately in feed after submission | POST-01 | UI reactivity, requires browser | Login → compose post → submit → verify post appears in list with username + timestamp |
| Deleted post disappears from page | POST-02 | UI reactivity, requires browser | Login → click delete on own post → verify post removed from list |
| Unauthorized delete rejected | POST-02 | Auth enforcement, requires two sessions | Login as user A → attempt to delete user B's post via direct API call → verify error response |
| Own profile shows posts in reverse-chron | PROF-01 | Browser navigation required | Login → navigate to `/username` → verify posts in correct order |
| Other user profile shows their posts | PROF-02 | Browser navigation required | Navigate to `/other_username` while logged out or in → verify correct user's posts shown |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
