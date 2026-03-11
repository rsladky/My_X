---
phase: 01-foundation
plan: 02
status: complete
completed_at: 2026-03-11
---

# 01-02 Summary: Database Migrations & SQLx Offline Mode

## One-liner
Three database migrations applied, `.sqlx/` offline metadata committed, `SQLX_OFFLINE=true cargo build` verified.

## What Was Built

### Migration files
- `migrations/20240101000001_create_users.sql` — users table: id SERIAL PK, email UNIQUE, password_hash, created_at/updated_at + idx_users_email
- `migrations/20240101000002_create_posts.sql` — posts table: id SERIAL PK, author_id FK→users, content TEXT CHECK(<=280), timestamps + idx_posts_author_id + idx_posts_pagination (created_at DESC, id DESC)
- `migrations/20240101000003_create_follows.sql` — follows table: composite PK (follower_id, following_id), both FK→users, created_at + idx_follows_following_id

### Environment config
- `.env` — local dev (not committed, contains secrets)
- `.env.example` — committed template for contributors
- `DATABASE_URL=postgres://localhost/my_x_dev`
- `SQLX_OFFLINE=true`

### SQLx offline
- `.sqlx/` directory committed with `.gitkeep` placeholder
- No queries in app code yet — will be populated in Phase 2 (Auth)
- `SQLX_OFFLINE=true cargo build` exits 0 ✓

## Verification
- `psql my_x_dev -c "\dt"` shows users, posts, follows, _sqlx_migrations ✓
- `sqlx migrate run` applied all 3 migrations without errors ✓
- `SQLX_OFFLINE=true cargo build` exits 0 (no DATABASE_URL required) ✓

## Commits
- `cb9df66` — feat(01-02): add database migrations for users, posts, follows
- `aa5d224` — feat(01-02): configure SQLx offline mode and commit .sqlx/ placeholder

## Deviations
- **PostgreSQL installed via brew** — not pre-installed on this machine; installed `postgresql@16` via Homebrew and started as a brew service
- **sqlx-cli installed** — `sqlx-cli v0.8.6` installed via cargo (not pre-installed)
- **.sqlx/ is empty** — no app queries yet; placeholder committed to establish the directory in git
