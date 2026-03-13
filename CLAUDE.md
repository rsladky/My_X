# My_X — Claude Context

## What This Is

A full-stack Twitter/X clone built entirely in Rust as a learning project. Backend: Axum. Frontend: Leptos (SSR + WASM hydration). Database: PostgreSQL via SQLx. The goal is understanding Rust ownership, async, and real-world patterns — not production deployment.

## Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| HTTP server | axum | 0.8.8 |
| Frontend | leptos + leptos_axum | 0.8.17 |
| Build tool | cargo-leptos | 0.2.39 |
| Database | PostgreSQL | 16+ |
| DB access | sqlx | 0.8.6 |
| Auth | jsonwebtoken (rust_crypto feature) + argon2 | 10.3.0 / 0.5.3 |
| Async runtime | tokio | 1.x |

## Dev Commands

```bash
cargo leptos watch          # dev server with hot reload
cargo test --features ssr   # run tests
cargo build --features ssr  # server build
cargo build                 # WASM/hydrate build (must also pass)
sqlx migrate run            # apply DB migrations
```

## Established Patterns

**Server functions only** — all frontend↔backend data flow uses `#[leptos::server]` functions. No manual fetch, no REST client.

**Feature-gate server-only crates** — sqlx, jsonwebtoken, argon2, uuid are all under `[ssr]` in Cargo.toml. WASM compilation will fail otherwise.

**Shared types must compile in WASM** — structs shared between server and client (e.g. `PostWithAuthor`, `AuthUser`) must be defined OUTSIDE `#[cfg(feature = "ssr")]`.

**PgPool via context** — pool is provided via `leptos_routes_with_context`, not Axum State. Access with `use_context::<PgPool>()` inside server functions.

**Auth state** — `RwSignal<Option<AuthUser>>` provided via context. Access with `use_context::<RwSignal<Option<AuthUser>>>()` in components.

**Inline styles** — no CSS framework, no Tailwind. All styling is inline.

**`navigate()` after `spawn_local`** — call `navigate()` inside `spawn_local` after `validate_token` resolves to avoid flash of unauthenticated state.

**`AuthUser`** carries `user_id: i32` + `username: String`.

## Authorization Pattern

`delete_post`: use `WHERE id = $1 AND author_id = $2` + check `rows_affected() == 0` → error. The DB is authoritative; user_id comes from auth context signal.

## Design & Theme (Twitter/X Parity)

**Goal:** UI must match X's design language exactly. All styling is inline; no CSS files.

### Color Palette

| Element | Color | Hex | Usage |
|---------|-------|-----|-------|
| Primary | Twitter Blue | `#1D9BF0` | Links, CTAs, active states, highlights |
| Text (Light Mode) | Black | `#000000` | Primary text |
| Text (Dark Mode) | White | `#FFFFFF` | Primary text |
| Secondary Text | Gray | `#536471` | Timestamps, metadata |
| Borders/Dividers | Light Gray | `#EFF3F4` (light) / `#2F3336` (dark) | Post dividers, input borders |
| Background (Light) | White/Off-white | `#FFFFFF` / `#F7F9FA` | Page, cards |
| Background (Dark) | Dark Gray/Black | `#000000` / `#15181C` | Page, cards |
| Hover (Light) | Light Gray | `#F7F9FA` | Post/element hover |
| Hover (Dark) | Dark Gray | `#181B1F` | Post/element hover |

### Typography

- **Font:** System stack (e.g., `-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif`)
- **Primary text:** 15px, weight 400
- **Bold/headings:** weight 700
- **Small text:** 13px, weight 400 (timestamps, metadata)
- **Button text:** 15px, weight 700
- **Line height:** 1.5 (readability)

### Layout & Spacing

- **Post cards:** 12px horizontal padding, 12px vertical padding, 1px border-bottom divider
- **Gaps between posts:** 0 (continuous feed with dividers)
- **Sidebar:** max-width 280px (left nav), 280px–350px (right sidebar)
- **Main feed:** max-width 600px (standard X width)
- **Button padding:** 8px 16px (small), 12px 24px (large)
- **Border radius:** 20px (buttons), 16px (modals/cards), 0px (most elements)

### Component Styling Rules

**Posts**
- White background (light) / `#15181C` (dark), 1px bottom border
- Author name: bold (700), text color
- Handle: `#536471`, font-size 13px
- Timestamp: `#536471`, font-size 13px
- Post text: 15px, line-height 1.5
- Hover: light gray background (`#F7F9FA` light / `#181B1F` dark), cursor pointer

**Buttons**
- Primary: `#1D9BF0` background, white text, weight 700, 20px border-radius
- Secondary: transparent, `#1D9BF0` border + text
- Hover: opacity 0.9 or slightly darker shade
- Active: slightly darker (e.g., `#1a8cd8`)
- Disabled: gray text, cursor not-allowed

**Input fields**
- Border: 1px solid `#EFF3F4` (light) / `#2F3336` (dark)
- Focus: border-color `#1D9BF0`
- Padding: 12px
- Border-radius: 4px
- Font: 15px

**Navigation**
- Hover items: light background
- Active/selected: `#1D9BF0` text or accent

### Design Checklist (Before Merging)

When reviewing PRs with visual changes, verify:

- [ ] Colors match the palette above (use hex values, not color names)
- [ ] Text colors have sufficient contrast (WCAG AA minimum 4.5:1)
- [ ] Buttons are 20px border-radius, 15px font, weight 700
- [ ] Posts have 1px bottom border (not box-shadow)
- [ ] Hover states match the spec (light/dark mode appropriate)
- [ ] Spacing follows 4px/8px/12px/16px/24px system
- [ ] No external CSS files; all inline styles only
- [ ] Dark mode colors applied consistently (if implemented)
- [ ] Typography matches (15px primary, 13px secondary)
- [ ] Sidebar widths within spec (280px left, 280–350px right)
- [ ] Feed width not exceeding 600px
- [ ] Button padding consistent (8px 16px or 12px 24px)

## Current State

- Phase 1 (Foundation): complete
- Phase 2 (Auth): complete — register, login, JWT, persist, logout all working
- **Phase 3 (Posts + Profiles): next up** — see TODO.md

## Out of Scope (v1)

Likes, replies, retweets, media uploads, real-time updates, search, notifications, DMs, deployment.
