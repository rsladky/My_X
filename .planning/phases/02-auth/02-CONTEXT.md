# Phase 2 Context: Auth

This file captures all implementation decisions for Phase 2 (Auth). It is consumed by the researcher and planner agents before any planning begins.

<decisions>

## Locked Decisions (from prior sessions)

- **JWT library**: `jsonwebtoken` 10.3.0 (already in Cargo.toml under `[ssr]`)
- **Password hashing**: `argon2` 0.5.3, called via `tokio::task::spawn_blocking` to avoid blocking the async runtime
- **Frontend ↔ backend**: Leptos server functions only — no separate REST client, no manual fetch calls
- **JWT storage**: `localStorage` — persists across browser refresh
- **Credential errors**: Same generic error message for wrong password AND non-existent account (prevents account enumeration)

## Form Layout

- Separate routes: `/login` and `/register`
- Each form links to the other for navigation (e.g., "Already have an account? Log in")
- Styling/layout at Claude's discretion — keep it clean and functional

## Registration Fields

- 3 fields: email, password, confirm password
- Confirm password match checked client-side before the server function is called
- Server receives only email + password (no confirm field sent over the wire)

## Validation & Error Display

- Server-side errors: single message displayed above the submit button
- No complex per-field inline errors — keeps Leptos reactive state management simple
- Client-side validation: confirm password match only (all other validation is server-side)

## Post-Auth Navigation

| Event    | Action                          |
|----------|---------------------------------|
| Register | Auto-login → redirect to `/`   |
| Login    | Redirect to `/`                 |
| Logout   | Redirect to `/login`            |

## Auth State in UI

- A Leptos reactive `RwSignal<Option<AuthUser>>` provided at App root via context
- `AuthUser` carries: user id (UUID) + username (String)
- On page load: read JWT from `localStorage` → call a server function to validate → populate signal
- All child routes can read the signal via `use_context::<RwSignal<Option<AuthUser>>>()`

</decisions>

<code_context>

## Reusable Assets

- `src/server/error.rs` — `AppError::AuthError(String)` for 401 responses
- `src/server/error.rs` — `AppError::ValidationError(String)` for bad input (email format, password too short)

## Already in Cargo.toml (ssr feature)

| Crate           | Version | Purpose               |
|-----------------|---------|-----------------------|
| `jsonwebtoken`  | 10.3.0  | JWT sign/verify       |
| `argon2`        | 0.5.3   | Password hash/verify  |
| `uuid`          | 1       | User IDs              |
| `rand`          | 0.8     | Salt/secret entropy   |
| `sqlx`          | 0.8.6   | DB queries            |

## Integration Points

- `src/app.rs` — Add `/login` and `/register` routes; provide auth context signal from root
- `src/main.rs` — Ensure `sqlx::PgPool` is in Axum state (needed by auth server functions)
- `src/server/mod.rs` — Add `auth` submodule for register/login/logout/validate-token handlers
- DB — `users` table already exists (Phase 1 migration 002)

</code_context>

<domain>

## Auth Domain Notes

- Registration creates a new user row: `id` (UUIDv4), `email`, `username` (derived from email local-part or user-provided — decide in planning), `password_hash`, `created_at`
- Login: look up user by email → verify argon2 hash → issue JWT
- JWT claims: `sub` (user id as string), `username`, `exp` (expiry — decide reasonable TTL in planning, suggest 7 days)
- JWT secret: loaded from env var `JWT_SECRET` (already expected pattern from Phase 1 scaffold)
- Logout: client-side only — delete JWT from `localStorage`, clear the auth signal

</domain>

<specifics>

## Open Implementation Details (resolve during planning)

1. **Username field on register**: Plan should decide whether to collect username explicitly (4-field form) or derive it from email. Default: derive from email local-part (keep form simple at 3 fields).
2. **JWT TTL**: Suggest 7 days — planner should confirm.
3. **Email format validation**: Server-side check for `@` presence is sufficient; no heavy regex.
4. **Password minimum length**: 8 characters minimum, checked server-side.
5. **PgPool in state**: Verify whether Phase 1 already wired `PgPool` into Axum state; if not, this phase must add it.

</specifics>

<deferred>

## Explicitly Out of Scope for Phase 2

- Email verification / confirmation flow
- Password reset / forgot-password
- OAuth / social login
- Rate limiting on auth endpoints
- Session revocation / JWT blacklist
- Remember-me / "stay logged in" toggle
- Admin roles or permission scopes

</deferred>
