# My_X

## What This Is

A full-stack Twitter/X clone built entirely in Rust as a learning project. It focuses on the core social mechanics — posting, following users, and reading a feed — using Rust end-to-end from backend API to frontend UI.

## Core Value

A working Rust full-stack app that teaches ownership, async, and real-world patterns by implementing the essential social graph of Twitter.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] User can sign up and log in with email and password
- [ ] User can create short text posts (tweets)
- [ ] User can follow and unfollow other users
- [ ] User can view a feed of posts from people they follow
- [ ] User can view their own profile and post history
- [ ] Full-stack UI built in Rust (Leptos or Yew)

### Out of Scope

- Likes, replies, retweets — keep scope to core social graph for v1
- Media uploads — complexity not needed for learning goal
- Notifications — deferred to keep focus on Rust fundamentals
- Deployment — local-only is sufficient for this learning project
- DMs, search, trending — not core, not needed for v1

## Context

- Pure learning project — correctness and understanding Rust patterns matter more than performance optimization
- Full-stack Rust: backend (likely Axum or Actix-web) + frontend (Leptos or Yew)
- PostgreSQL as the database, accessed via SQLx or Diesel
- Auth via email/password with JWT or session cookies
- Target: runs locally, no production deployment needed

## Constraints

- **Tech stack**: Rust only (both backend and frontend) — the whole point is learning Rust
- **Scope**: Core social features only — posts, follow, feed, profiles, auth
- **Environment**: Local development only — no deployment pipeline needed

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Full-stack Rust (not JS frontend) | Learning goal is Rust — using JS would defeat the purpose | — Pending |
| PostgreSQL over SQLite | More realistic, better ecosystem support in Rust | — Pending |
| Email/password auth | Straightforward to implement, teaches JWT/session patterns | — Pending |

---
*Last updated: 2026-03-11 after initialization*
