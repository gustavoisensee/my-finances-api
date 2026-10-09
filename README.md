# My Finances API

A RESTful API for personal finance management built with Rust, Axum and SQLx, deployed as a single Vercel serverless function.

## 🚀 Features

- **User Authentication**: Firebase ID tokens verified on every request (RS256 against Google's public keys); users are created automatically on first sign-in
- **Financial Tracking**:
  - Income management
  - Budget creation and tracking
  - Expense categorization
  - Monthly and yearly financial reports
- **Multi-user Support**: Each user only ever sees their own financial data
- **PostgreSQL Database**: Powered by Supabase, queries checked at compile time by SQLx
- **Serverless**: One Axum router served from a single Vercel function

## 📋 Prerequisites

- Rust (stable, edition 2024): `rustup toolchain install stable`
- `make` (preinstalled on macOS/Linux)
- PostgreSQL database (Supabase account, or a local Postgres for development)
- Node.js only for the optional Vercel CLI and agent-skills targets (run through `npx`)

## 🛠️ Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd my-finances-api
   ```

2. **Install the tooling** (rustfmt, clippy, sqlx-cli, cargo-deny, cargo-audit, cargo-watch)

   ```bash
   make install-tools
   ```

3. **Set up environment variables**

   ```bash
   make env    # copies .env.example to .env if it doesn't exist
   ```

   Update the following variables:
   - `DATABASE_URL`: Your Supabase connection string (pooled for runtime, direct for migrations)
   - `FIREBASE_PROJECT_ID`: Your Firebase project ID
   - `ADMIN_FIREBASE_UIDS`: (optional) Comma-separated list of Firebase UIDs with admin access
   - `APP_ENV`: (optional) `production` hides internal error details; falls back to `VERCEL_ENV`
   - `DB_MAX_CONNECTIONS` / `DB_STATEMENT_CACHE_CAPACITY`: (optional) pool tuning
   - `PORT`: Port for the local server (default: 3001)

4. **Initialize the database**

   ```bash
   make db-setup    # create the database (local), apply migrations, seed years 2024–2026 and categories
   ```

   **Important**:
   - Users are automatically created on their first Firebase sign-in
   - Seeding is idempotent and can be run repeatedly

   **Database Management**:
   ```bash
   make db-clean    # remove all data (preserves schema; refuses to run in production)
   make db-seed     # repopulate with fresh seed data
   ```

Steps 2–4 can also be done in one go with `make setup`.

## 🏃‍♂️ Running the Application

### Development Mode
```bash
make dev      # or `make watch` to restart on file changes
```
The server will start on `http://localhost:3001` (or your configured `PORT`).

### Tests
```bash
make test
```
Integration tests use `#[sqlx::test]`, which creates an isolated database per test, so `DATABASE_URL` must point at a Postgres server where the user can create databases.

### Lint
```bash
make fmt-check lint    # or `make ci` to run everything CI runs
```

## 🧰 Make Commands

Run `make` (or `make help`) to list every command. All commands read `DATABASE_URL` and the other variables from `.env`; to target a different database for a single command, prefix it, e.g. `DATABASE_URL=<direct-connection-url> make db-migrate`.

**Setup**

| Command | Description |
| --- | --- |
| `make setup` | First-time setup: tools, `.env`, database (create + migrate + seed) |
| `make install-tools` | Install rustfmt, clippy, sqlx-cli, cargo-deny, cargo-audit, cargo-watch |
| `make env` | Create `.env` from `.env.example` (if missing) |

**Development & build**

| Command | Description |
| --- | --- |
| `make dev` | Run the API locally on `PORT` (default 3001) |
| `make watch` | Run the API and restart on changes |
| `make run-release` | Run the API locally with the optimized release build |
| `make docs` | Serve the Swagger UI on http://localhost:3002 (run alongside `make dev`) |
| `make check` | Type-check all targets |
| `make build` | Debug build of all targets |
| `make build-release` | Release build of the Vercel function binary |
| `make build-offline` | Build without a database using `.sqlx/` (what Vercel and CI do) |
| `make clean` | Remove build artifacts |

**Quality & tests**

| Command | Description |
| --- | --- |
| `make fmt` | Format the code |
| `make fmt-check` | Check formatting |
| `make lint` | Run clippy with warnings as errors |
| `make lint-fix` | Apply clippy's automatic fixes |
| `make test` | Run all tests |
| `make test-unit` | Run unit tests only (no database needed) |
| `make test-integration` | Run integration tests only (`tests/`) |
| `make test-one T=expense` | Run tests whose name matches `T` |

**Security**

| Command | Description |
| --- | --- |
| `make deny` | Check advisories, licenses, bans and sources (cargo-deny) |
| `make audit` | Check dependencies against the RustSec advisory database |
| `make security` | Run both dependency checks |
| `make ci` | Run everything CI runs (fmt-check, lint, test, build-offline, deny) |

**Database**

| Command | Description |
| --- | --- |
| `make db-setup` | Create, migrate and seed the database |
| `make db-create` | Create the database in `DATABASE_URL` |
| `make db-drop` | Drop the database (asks for confirmation) |
| `make db-reset` | Drop, recreate, migrate and seed (asks for confirmation) |
| `make db-migrate` | Apply pending migrations (Supabase: use the direct connection, port 5432) |
| `make db-revert` | Revert the last migration (reversible migrations only) |
| `make db-status` | Show applied and pending migrations |
| `make db-new NAME=add_budget_notes` | Create a new migration file |
| `make db-seed` | Seed default years and categories (idempotent) |
| `make db-clean` | Delete all data, keep the schema (refuses in production) |
| `make sqlx-prepare` | Regenerate `.sqlx/` offline query metadata (commit the result) |
| `make sqlx-check` | Verify `.sqlx/` is up to date with the queries |

**Vercel** (uses `npx vercel`; override with `VERCEL=vercel` if installed globally)

| Command | Description |
| --- | --- |
| `make vercel-link` | Link this directory to a Vercel project |
| `make vercel-dev` | Run the function locally through the Vercel CLI |
| `make deploy-preview` | Deploy a preview build |
| `make deploy-prod` | Deploy to production |
| `make vercel-logs URL=<deployment-url>` | Show runtime logs for a deployment |

**Agent skills**

| Command | Description |
| --- | --- |
| `make skills-list` | List the project's agent skills (`.agents/skills`) |
| `make skills-update` | Update the agent skills from their `skills-lock.json` sources |

## 📚 API Endpoints

> **💡 Interactive docs (Swagger UI)**
> Run `make dev` and, in another terminal, `make docs`, then open [http://localhost:3002](http://localhost:3002).
> Click **Authorize**, paste a Firebase ID token, and use **Try it out** on any endpoint.
> The page is `postman/index.html` and the spec is `postman/openapi.json` (OpenAPI 3).
>
> **💡 Postman Collection Available!**
> Import `postman/My-Finances-API.postman_collection.json` into Postman for easy testing.
> See [postman/POSTMAN_GUIDE.md](./postman/POSTMAN_GUIDE.md) for detailed instructions.

### Public Endpoints
- `GET /` - API index
- `GET /status` - API status check

### Protected Endpoints (Require Authentication)

All protected endpoints return a **401 Unauthorized** JSON response if no valid Firebase ID token is provided:
```json
{
  "error": "Unauthorized",
  "message": "Authentication required. Please provide a valid Firebase ID token.",
  "code": "UNAUTHENTICATED"
}
```

**Years & Categories**
- `GET /year` - Get all years (default years: 2024, 2025, 2026 are seeded)
- `GET /category` - Get all categories (user's custom categories + system default categories, with `expenseCount`)
- `POST /category` - Create a custom category
- `PUT /category/:id` - Rename a custom category
- `DELETE /category/:id` - Delete a custom category (rejected while expenses use it)

**Budgets**
- `GET /budget` - Get all budgets
- `POST /budget` - Create a budget
- `PUT /budget/reorder` - Reorder the budgets of a month
- `PUT /budget/:id` - Update a budget
- `DELETE /budget/:id` - Delete a budget

**Expenses**
- `GET /expense` - Get all expenses
- `POST /expense` - Create an expense
- `PUT /expense/:id` - Update an expense
- `DELETE /expense/:id` - Delete an expense

**Income**
- `GET /income` - Get all incomes
- `POST /income` - Create an income
- `PUT /income/reorder` - Reorder the incomes of a month
- `PUT /income/:id` - Update an income
- `DELETE /income/:id` - Delete an income

**Months**
- `GET /month?yearId=` - Get all months of a year (`iIncomes`, `iBudgets`, `iExpenses` include relations)
- `GET /month/:id` - Get month by ID (404 when missing)
- `POST /month` - Create a month
- `POST /month/:id/copy` - Copy a month with its incomes and budgets to a new month/year
- `PUT /month/:id` - Update a month
- `DELETE /month/:id` - Delete a month

List endpoints accept `take` (default 10, max 100).

**Admin Endpoints** (Require admin role via Firebase custom claims or UID allowlist)
- `GET /access-token` - Get all access tokens
- `GET /user` - Get all users (returns user data: id, firebaseUid, clerkId, createdAt)
- `POST /user` - Create a user manually (users are auto-created on first Firebase sign-in)
- `PUT /user/:id` - Update a user's firebaseUid
- `DELETE /user/:id` - Delete a user and all their data

**Admin Access Setup:**
1. Set the `admin: true` custom claim on a user in Firebase, OR
2. Set `role: 'admin'` custom claim, OR
3. Add the user's Firebase UID to the `ADMIN_FIREBASE_UIDS` env var (comma-separated)

Admin endpoints return a **403 Forbidden** JSON response if the authenticated user doesn't have admin role:
```json
{
  "error": "Forbidden",
  "message": "Admin access required.",
  "code": "ADMIN_ACCESS_REQUIRED"
}
```

## 🗄️ Database

Migrations live in `migrations/` (create one with `make db-new NAME=<name>`) and are applied with `make db-migrate`. The baseline migration is idempotent, so it can be applied to a database previously managed by Prisma without changing existing tables.

Queries use SQLx's compile-time checked macros. Builds without a database (Vercel, CI) read the cached query metadata in `.sqlx/` (`SQLX_OFFLINE=true`). After adding or changing a query, regenerate it against a migrated database and commit the result:

```bash
make sqlx-prepare    # `make sqlx-check` verifies it is up to date
```

## 🌐 CORS Configuration

The API allows requests from:
- `http://localhost:3000` (local web app, outside production only)
- `http://localhost:3002` (local API docs from `make docs`, outside production only)
- `https://my-finances-web.gustavoisensee.com` (production frontend)

Update `src/config.rs` to add more origins.

## 🔒 Security Notes

- Only `FIREBASE_PROJECT_ID` is needed for token verification; no service-account credentials are stored
- Tokens must be RS256, signed by a current Google key, with the correct audience, issuer, expiry and auth time
- Keep your `.env` file secure and never commit it to version control
- Responses carry Helmet-equivalent security headers and `Cache-Control: no-store`
- Request bodies are limited to 1 MB; internal error details are hidden in production
- All protected endpoints require a valid Firebase ID token in the `Authorization: Bearer <token>` header
- Admin access is controlled via Firebase custom claims (`admin: true` or `role: 'admin'`) or the `ADMIN_FIREBASE_UIDS` env var
- CI runs `cargo-deny` (advisories, licenses, sources), Gitleaks and CodeQL

## 📦 Technology Stack

- **Language**: Rust
- **Framework**: Axum (on `vercel_runtime`)
- **Database access**: SQLx (PostgreSQL / Supabase)
- **Authentication**: Firebase ID tokens via `jsonwebtoken`
- **Hosting**: Vercel serverless functions

## 🚀 Deployment

### Vercel

`vercel.json` routes every path to the single function in `api/main.rs`, which serves the whole Axum router.

1. Link the project: `make vercel-link`
2. Set the environment variables (`DATABASE_URL` with the pooled connection, `FIREBASE_PROJECT_ID`, optionally `ADMIN_FIREBASE_UIDS`) in the Vercel dashboard or with `npx vercel env add`
3. Run migrations against the direct connection before deploying schema changes: `DATABASE_URL=<direct-connection-url> make db-migrate`
4. Deploy: `make deploy-preview` or `make deploy-prod`

Consider setting the function region close to the Supabase database region to minimise latency.

## 📄 License

MIT

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
