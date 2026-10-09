# Postman Guide

> Prefer the browser? This folder also has a Swagger UI: run `make dev` and `make docs`, then open http://localhost:3002 (`index.html` + `openapi.json`). Use `localhost`, not `127.0.0.1`, or CORS will block the requests.

## Import

1. Open Postman → **Import**
2. Import `My-Finances-API.postman_collection.json`
3. Import one environment:
   - `My-Finances-API.postman_environment.json` (local, `http://localhost:3001`)
   - `My-Finances-API.postman_environment.production.json` (production)
4. Select the environment in the top-right dropdown

## Variables

| Variable | Description |
| --- | --- |
| `base_url` | API base URL |
| `auth_token` | Firebase ID token (raw token, without `Bearer `) |

Every protected request sends `Authorization: Bearer {{auth_token}}`.

## Getting a Firebase ID Token

Sign in through the frontend, then in the browser console run:

```javascript
const token = await firebase.auth().currentUser.getIdToken();
console.log(token);
```

(With the modular SDK: `await getAuth().currentUser.getIdToken()`.)

Copy the value into the `auth_token` environment variable. Firebase ID tokens expire after one hour, so refresh it when requests start returning 401.

Your user record is created automatically on your first authenticated request; there is no sign-up endpoint.

## Admin Access

Admin endpoints (`/user`, `/access-token`) require one of:

1. The Firebase custom claim `admin: true`
2. The Firebase custom claim `role: 'admin'`
3. Your Firebase UID listed in the `ADMIN_FIREBASE_UIDS` environment variable of the API

After changing custom claims, get a fresh ID token so it carries the new claims.

## Suggested Flow

1. **Public → Status** to check the API is up
2. **Years → Get All Years** to find a `yearId` (years are created by `cargo run --bin seed`)
3. **Months → Create Month** with that `yearId`
4. **Budgets → Create Budget** and **Incomes → Create Income** with the new `monthId`
5. **Categories → Get All Categories** to find a `categoryId`
6. **Expenses → Create Expense** with the `budgetId` and `categoryId`
7. **Months → Get Month By ID** with `iIncomes`, `iBudgets` and `iExpenses` set to `true` to see everything together

IDs in the sample requests (`/month/1`, `"monthId": 1`, …) are placeholders; replace them with IDs from your own responses.

## Common Errors

**401 Unauthorized**
```json
{
  "error": "Unauthorized",
  "message": "Authentication required. Please provide a valid Firebase ID token.",
  "code": "UNAUTHENTICATED"
}
```
The token is missing, expired, or issued for a different Firebase project. Get a fresh token.

**403 Forbidden**
```json
{
  "error": "Forbidden",
  "message": "Admin access required.",
  "code": "ADMIN_ACCESS_REQUIRED"
}
```
The token is valid but has no admin claim and the UID isn't in `ADMIN_FIREBASE_UIDS`.

**400 Bad Request** — `{"message": "..."}` describing the invalid field (for example `"yearId must be passed."` or `"Value must be a number and greater than 0."`).

**404 Not Found** — the resource doesn't exist or belongs to another user, e.g. `{"message": "Month not found."}`.
