---
name: project-conventions
description: Conventions for the My Finances Rust API (Axum + SQLx on a single Vercel function). Use when adding or changing an endpoint, handler, SQL query, migration, model, or integration test in this repository, or when building, testing, or deploying it.
---

# My Finances API conventions

## Layout

| Path | Role |
| --- | --- |
| `api/main.rs` | Vercel entrypoint; wraps `build_app` with `vercel_runtime`. Keep it thin. |
| `src/lib.rs` | `build_app(state)`: router + layers. Shared by Vercel, `src/bin/local.rs` and tests. |
| `src/routes/mod.rs` | The only place routes are registered (Axum 0.8 path syntax: `/{id}`). |
| `src/routes/<resource>.rs` | Handlers, one file per resource. |
| `src/routes/fields.rs` | Shared body validators and ownership/lock helpers. |
| `src/models/` | Response structs (`#[serde(rename_all = "camelCase")]`). |
| `src/error.rs` | `AppError` / `AppResult`. |
| `src/extract.rs` | `ApiJson`, `ApiQuery`, `Id`, `TakeQuery`, `take_from`, `Body`, `parse_id_list`. |
| `migrations/` | sqlx-cli migrations. `.sqlx/` holds offline query metadata (committed). |
| `tests/common/mod.rs` | Test app, token signing, `Client`, seed and fixture helpers. |

## Adding an endpoint

```
- [ ] Handler in src/routes/<resource>.rs
- [ ] Route registered in src/routes/mod.rs (static segments like /reorder before /{id} is fine; Axum prefers static)
- [ ] Ownership enforced in SQL (see the rust-api-security skill)
- [ ] Integration tests in tests/<area>.rs, including another user's data -> 404
- [ ] cargo sqlx prepare -- --all-targets, commit .sqlx/
- [ ] README endpoint list, postman collection and postman/openapi.json (Swagger UI) updated
```

### Handler shape

Extractor order: `State`, then the auth extractor, then `Id`/`ApiQuery`, then `ApiJson` last (it consumes the body).

```rust
pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,                 // AdminUser for admin-only routes
    ApiJson(body): ApiJson<Body>,
) -> AppResult<(StatusCode, Json<Income>)> {
    let value = fields::amount(body.get("value"))?;
    let month_id =
        fields::month_owned(&state.db, fields::id(body.get("monthId")), user.user_id).await?;
    // ...
    Ok((StatusCode::CREATED, Json(income)))
}
```

- Request bodies are parsed as `Body` (a JSON map) and validated field by field with `fields::*`, so each missing/invalid field gets its own 400 message instead of a generic deserialization error.
- Use `ApiJson`, `ApiQuery` and `Id` rather than Axum's `Json`/`Query`/`Path`: their rejections map to the API's `{ "message": ... }` 400 bodies.
- List endpoints take `ApiQuery<TakeQuery>` and pass `take_from(query.take.as_deref())` to `LIMIT` (default 10, clamped 1..=100). Always add an `ORDER BY`.
- Response field names and messages are a contract with the web app: keep existing messages verbatim and keep camelCase names.

### Errors

- `AppError::validation(msg)` -> 400, `AppError::not_found(msg)` -> 404, both `{ "message": msg }`.
- `?` on `sqlx::Error` becomes a 500 with the detail hidden in production; never format DB errors into client messages.
- Map expected constraint errors explicitly with `error::is_unique_violation` / `is_foreign_key_violation`.
- A missing or foreign row on update/delete is 404, never 500: use `fetch_optional(...)?.ok_or_else(|| AppError::not_found(NOT_FOUND))`.

## SQL

- Use the compile-time macros (`query!`, `query_as!`, `query_scalar!`). Never build SQL with `format!`.
- Tables and camelCase columns are quoted: `"Month"`, `"userId"`, `"createdAt" AS created_at`.
- Timestamps are `TIMESTAMP(3)` in UTC (`NaiveDateTime`); serialize with `#[serde(serialize_with = "crate::time::serialize")]` so JSON matches `2025-01-01T00:00:00.000Z`.
- Multi-statement writes run in a transaction (`state.db.begin()`, pass `&mut *tx`, `tx.commit()`). Lock the parent before computing an index: `fields::lock_month`, `fields::lock_month_pair` (locks in ascending id order to avoid deadlocks), `SELECT ... FOR UPDATE OF`.
- Batch related loads with `= ANY($1)` instead of a query per row.

## Migrations and offline metadata

```bash
make db-new NAME=<name>    # creates migrations/<timestamp>_<name>.sql
make db-migrate            # local DB, or DATABASE_URL=<Supabase direct connection, port 5432> make db-migrate
make sqlx-prepare          # after any query or schema change; commit .sqlx/
```

- Vercel and CI build with `SQLX_OFFLINE=true`; a stale or missing `.sqlx/` entry fails the build.
- Write migrations to be safe on the existing production schema (`IF NOT EXISTS`, guarded `DO $$` blocks). Never edit an applied migration; add a new one.
- Migrations are not run by Vercel; run them before deploying code that depends on them.

## Tests

```rust
#[sqlx::test]                      // fresh database per test, migrations applied
async fn income_isolation(pool: PgPool) {
    let seed = seed(&pool).await;  // years 2025/2026 + global "Housing" category
    let app = app(pool);
    let alice = Client::new(&app, "alice");
    let bob = Client::new(&app, "bob");

    let month = create_month(&alice, seed.year_2025, 1).await;
    let income = create_income(&alice, month, "Salary").await;

    let res = bob.delete(&format!("/income/{}", id(&income))).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
}
```

- `Client::new(&app, uid)` signs a valid Firebase token for `uid`; `Client::admin(&app)` uses an admin claim. `token_with(uid, json!({...}))` overrides claims for auth edge cases.
- `send_request` returns `TestResponse { status, headers, body }` for raw requests (headers, CORS, malformed bodies).
- `test_config(true)` + `app_with_config` exercises production behaviour (hidden 500 details, no localhost CORS).
- Every new endpoint needs: happy path, validation 400s, and a cross-user 404.

## Verify before finishing

Prefer the `Makefile` targets (`make help` lists them all). `make ci` runs exactly these checks:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test                                   # needs DATABASE_URL to a Postgres that allows CREATE DATABASE
SQLX_OFFLINE=true cargo build --all-targets  # what Vercel does
cargo deny check advisories licenses bans sources
```

## Local run and deploy

- `cargo run` serves the router on `PORT` (default 3001) via `src/bin/local.rs`; `cargo run --bin seed` / `--bin clean` manage data (`clean` refuses in production).
- `vercel.json` rewrites every path to `/api/main`; adding a route needs no Vercel change.
- Runtime env: `DATABASE_URL` (Supabase pooled, `?pgbouncer=true`), `FIREBASE_PROJECT_ID`, optional `ADMIN_FIREBASE_UIDS`, `APP_ENV`, `DB_MAX_CONNECTIONS`, `DB_STATEMENT_CACHE_CAPACITY`. Document new variables in `.env.example` and the README.
- Keep per-instance state (pool, JWKS cache) in `AppState`, created once at startup; do not spawn background tasks, since the function may freeze between invocations.
