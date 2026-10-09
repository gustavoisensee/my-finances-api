---
name: rust-api-security
description: Security rules for the My Finances Rust API - Firebase ID token verification, admin authorization, per-user data isolation in SQL, input validation, error hygiene, CORS, security headers, secrets and dependency auditing. Use when touching authentication, authorization, any handler or SQL that reads or writes user data, middleware/layers, logging, configuration, CI security checks, or when reviewing changes for security.
---

# Rust API security

The API is stateless: every protected request carries `Authorization: Bearer <Firebase ID token>`, verified on each call. There are no sessions or cookies.

## Authentication (`src/auth/firebase.rs`)

Do not loosen any of these checks; each one has a test in `tests/auth.rs`:

- Algorithm is **RS256 only**, chosen by the server. The token's `alg` header is never trusted (blocks `alg: none` and HS256-with-public-key confusion).
- Header `kid` is required and must match a key from Google's JWKS (`securetoken@system.gserviceaccount.com`), cached per `Cache-Control: max-age`. Unknown `kid` triggers at most one refresh per `MIN_REFRESH_INTERVAL`.
- `aud` = `FIREBASE_PROJECT_ID`, `iss` = `https://securetoken.google.com/<project>`; `exp`, `iat`, `sub` required; `iat` and `auth_time` not in the future (5 s leeway); `sub` 1-128 chars.
- Every verification failure returns the same 401 `UNAUTHENTICATED` body. Log the reason at `debug` only, never the token.
- Only `FIREBASE_PROJECT_ID` is needed. Do not add service-account credentials.

## Authorization

- Handlers declare access through extractors, never by checking headers manually:
  - `AuthUser { uid, user_id }`: any verified user; `user_id` is the internal `"User".id`.
  - `AdminUser`: claim `admin == true`, or `role == "admin"`, or uid in `ADMIN_FIREBASE_UIDS`; otherwise 403 `ADMIN_ACCESS_REQUIRED`.
- A route without one of these extractors is public. Only `/` and `/status` are public.

## Data isolation (the most important rule)

Every query on user data is scoped to `user.user_id` **in the SQL itself**. Ownership lives on `"Month"."userId"` and `"Category"."userId"`; budgets, incomes and expenses are owned through their month.

```sql
-- read/update/delete a child row: join to its owner in the same statement
SELECT i.id FROM "Income" i JOIN "Month" m ON m.id = i."monthId"
WHERE i.id = $1 AND m."userId" = $2

UPDATE "Expense" e SET ... FROM "Budget" b JOIN "Month" m ON m.id = b."monthId"
WHERE e.id = $1 AND e."budgetId" = b.id AND m."userId" = $2
```

- Every foreign id in a request body (`monthId`, `budgetId`, `categoryId`, ids in reorder lists) is checked for ownership before use (`fields::month_owned`, the budget/category checks in `routes/expense.rs`). Categories may also be global (`"userId" IS NULL`) for reads/references, but only the owner may modify them.
- Another user's resource returns **404 with the same message as a missing one**, never 403, so ids can't be probed.
- Aggregates (counts, sums) must also be filtered by user, as in `routes/category.rs`.
- Every new endpoint gets a cross-user test asserting 404 and that the other user's data is unchanged.

## Input handling

- SQL only through SQLx macros with bind parameters. No `format!`/string concatenation into SQL, including `ORDER BY`/`LIMIT` (use `take_from`).
- Validate each body field with `routes/fields.rs` (ids parse to `i32`, amounts finite and > 0, timestamps ISO-8601) rather than trusting deserialized types.
- Body size is capped at 1 MB (`BODY_LIMIT_BYTES`); keep `DefaultBodyLimit` in `build_app`.
- Don't add endpoints that fetch user-supplied URLs (SSRF) or touch the filesystem.

## Errors and logging

- Database/internal errors return 500 `{"message":"Internal server error"}`; details are attached only outside production by `internal_error_layer`. Never put `sqlx::Error` text, SQL, or stack traces in client messages.
- `internal_error_layer` logs method, path and the internal error server-side. Never log tokens, `Authorization` headers, request bodies, emails, or financial values.

## Transport and browser protections

- CORS (`src/security/cors.rs`): explicit origin allowlist from `config.rs`; `http://localhost:3000` (web app) and `http://localhost:3002` (`make docs`) only when not production. Never use `Any` origin, and don't enable credentials (auth is a header, not a cookie).
- Security headers (`src/security/headers.rs`) are the helmet defaults plus `Cache-Control: no-store`, applied to every response including errors and 404s. Keep them when adding layers; `tests/http.rs` asserts them.
- Production detection: `APP_ENV`, falling back to `VERCEL_ENV`. Code that relaxes anything for development must check `config.is_production`.

## Secrets and configuration

- Secrets come from environment variables only (Vercel project env, local `.env`). `.env` is gitignored; `.env.example` holds placeholders, never real values.
- Tests generate throwaway RSA keys at runtime; never commit key material.
- Gitleaks runs in CI; don't add allowlist entries to silence it.

## Dependencies and CI

- `cargo deny check advisories licenses bans sources` must pass (`deny.toml`); `cargo audit` uses `.cargo/audit.toml`.
- An advisory may only be ignored with a written reason proving the crate is not compiled into the API binary (check with `cargo tree -e normal --target x86_64-unknown-linux-gnu -i <crate>`), as done for `rsa`.
- Prefer rustls/ring over OpenSSL; keep `jsonwebtoken` on a backend without the `rsa` crate.
- CodeQL (Rust) and Gitleaks run in `.github/workflows/security.yml`.

## Review checklist

```
- [ ] Protected route uses AuthUser/AdminUser
- [ ] Every query on user data is scoped by user_id in SQL
- [ ] Referenced ids in the body are ownership-checked
- [ ] Foreign/missing resources -> identical 404
- [ ] No dynamic SQL, no DB error text in responses
- [ ] No tokens/bodies/PII in logs
- [ ] Cross-user and auth-failure tests added
- [ ] cargo deny passes
```
