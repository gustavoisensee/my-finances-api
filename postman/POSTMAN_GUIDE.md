# Postman Collection Guide

This guide will help you import and use the My Finances API Postman collection.

## 📦 Files Included

All files are located in the `postman/` folder:

- **My-Finances-API.postman_collection.json** - Complete API collection with all endpoints
- **My-Finances-API.postman_environment.json** - Local environment variables
- **My-Finances-API.postman_environment.production.json** - Production environment variables

## 🚀 Getting Started

### Step 1: Import the Collection

1. Open Postman
2. Click **Import** button (top left)
3. Select **postman/My-Finances-API.postman_collection.json**
4. Click **Import**

### Step 2: Import Environments

1. Click **Import** again
2. Select both environment files from the `postman/` folder:
   - `My-Finances-API.postman_environment.json` (Local)
   - `My-Finances-API.postman_environment.production.json` (Production)
3. Click **Import**

### Step 3: Select Environment

1. In the top-right corner, click the environment dropdown
2. Select **My Finances API - Local** for development
3. Or select **My Finances API - Production** for production testing

## 🔐 Authentication Workflow

### Using Clerk Authentication

This API uses Clerk for authentication. To use the API:

1. **Sign up/Login through your Clerk-enabled frontend application**
2. **Copy your Clerk session token** from your frontend application
   - The token is automatically managed by Clerk's frontend SDK
   - You can find it in cookies or via `await clerk.session.getToken()`
3. **Set the token in Postman**:
   - Open your environment (Local or Production)
   - Set the `auth_token` variable to your Clerk session token
   - Save the environment
4. **User sync happens automatically via webhooks**:
   - When you create an account in Clerk, the webhook syncs the user to the database
   - When you sign in, the `session.created` webhook ensures your user is synced

### How Authentication Works

- All protected endpoints use Clerk's `requireAuth()` middleware
- Your Clerk session token is validated on every request
- The API looks up your internal user ID using your Clerk user ID
- No manual token verification or user sync endpoints are needed

## 📋 Collection Structure

### Webhooks
- **Clerk Webhook** - Receives events from Clerk (user.created, user.updated, user.deleted, session.created)
  - Note: This is a server-to-server endpoint, not meant for manual Postman testing
  - User sync happens automatically when you sign up/sign in via your frontend

### Public Endpoints
- **Index** - API index page
- **Status** - API health check

### Protected Endpoints (Require Authentication)

#### Years & Categories
- Get All Years
- Get All Categories (returns user's custom categories + system default categories)

#### Months
- Get All Months (with query params for includes)
- Get Month By ID
- Create Month
- Copy Month
- Update Month
- Delete Month

#### Budgets
- Get All Budgets
- Create Budget
- Update Budget
- Delete Budget

#### Expenses
- Get All Expenses
- Create Expense
- Update Expense
- Delete Expense

#### Incomes
- Get All Incomes
- Create Income
- Update Income
- Delete Income

### Admin Endpoints (Require Admin Token)

#### Users
- Get All Users
- Create User
- Update User
- Delete User (cannot delete admin user ID: 1)

#### Access Tokens
- Get All Access Tokens

## 🔑 Environment Variables

### Variables Available

| Variable | Description | Example |
|----------|-------------|---------|
| `base_url` | API base URL | `http://localhost:3001` |
| `auth_token` | Clerk session token | Set manually from frontend |
| `user_id` | Current user ID | Auto-set after token verification |

### Switching Environments

- **Local Development**: `http://localhost:3001`
- **Production**: `https://my-finances-api-v4.onrender.com`

## 📝 Sample Data

### Create User (Admin)
```json
{
  "clerkId": "user_2abc123def456"
}
```

**Note**: Users are automatically created via Clerk webhooks. This endpoint is for manual/admin creation only. The clerkId must be a valid Clerk user ID.

### Create Month
```json
{
  "value": 1,
  "description": "January 2025",
  "createdAt": "2025-01-01T00:00:00.000Z",
  "yearId": 1
}
```

### Copy Month
```json
{
  "value": 2,
  "yearId": 1
}
```
**Note**: Copies a month with its incomes and budgets (expenses are not copied). The description is automatically generated (e.g., "Copy from January 2025").

### Create Budget
```json
{
  "description": "Groceries Budget",
  "value": 500.00,
  "createdAt": "2025-01-01T00:00:00.000Z",
  "monthId": 1
}
```

### Create Expense
```json
{
  "description": "Supermarket Shopping",
  "value": 85.50,
  "createdAt": "2025-01-15T10:30:00.000Z",
  "budgetId": 1,
  "categoryId": 1
}
```

### Create Income
```json
{
  "description": "Monthly Salary",
  "value": 5000.00,
  "createdAt": "2025-01-01T00:00:00.000Z",
  "monthId": 1
}
```

## 🎯 Query Parameters

### Pagination
Most GET endpoints support the `take` parameter:
```
GET /budget?take=20
```

### Month Includes
Get months with related data:
```
GET /month?yearId=1&iIncomes=true&iBudgets=true&iExpenses=true
```

Query parameters:
- `yearId` (required) - Filter by year ID
- `iIncomes` - Include incomes (true/false)
- `iBudgets` - Include budgets (true/false)
- `iExpenses` - Include expenses (true/false)
- `take` - Number of records to return (default: 10)

## ⚙️ Tips & Tricks

### 1. Get Your Clerk Token

**Option A: From Browser DevTools (if using Clerk's default cookies)**
- Open browser DevTools (F12)
- Go to Application/Storage → Cookies
- Find your Clerk session cookie (usually `__session` or `__clerk_session`)
- Copy the value and set it as `auth_token` in Postman environment

**Option B: From Frontend Code (recommended)**
In your frontend application, you can get the token programmatically:
```javascript
const token = await clerk.session.getToken();
console.log('Clerk Token:', token);
```
Copy this token and set it as `auth_token` in Postman environment

### 2. Test All Endpoints Quickly
Use Postman's **Collection Runner** to test all endpoints sequentially:
1. Right-click on the collection
2. Select **Run collection**
3. Configure and run

### 3. Create Data Flow
Follow this order to create a complete data set:
1. **Sign up/Sign in via Clerk** (in your frontend application)
   - User is automatically synced to database via webhooks
2. **Get your Clerk session token** (from frontend)
3. **Set token in Postman** environment variable `auth_token`
4. **Get available years** using "Get All Years" request
   - Default years (2024, 2025, 2026) are already seeded
   - Note the year ID you want to use
5. Create Month (requires yearId from step 4)
6. (Optional) Copy Month - to duplicate a month with its incomes and budgets
7. Create Budget (requires monthId from step 5)
8. Create Expense (requires budgetId and categoryId)
9. Create Income (requires monthId from step 5)

### 4. Use Variables in Request Bodies
You can use environment variables in request bodies:
```json
{
  "monthId": {{month_id}},
  "value": 500
}
```

### 5. Duplicate Requests
Right-click any request and select **Duplicate** to create variations for testing.

## 🔒 Admin Access

Admin access is controlled by **Clerk roles** in Clerk Organizations.

**To set up admin access:**
1. **Enable Organizations** in Clerk Dashboard
2. **Create an organization** (e.g., "Admins")
3. **Add users to the organization** with the 'admin' role
4. Users with 'admin' or 'org:admin' role will have access to admin endpoints

**To test admin endpoints:**
1. Ensure your user has the 'admin' role in a Clerk organization
2. Sign in via Clerk using the admin account
3. Copy your Clerk session token to Postman environment variable `auth_token`
4. Admin-only endpoints will now work

**Note**: Admin access is determined by Clerk roles, not database user IDs. Any user with the admin role in Clerk can access admin endpoints.

## 🐛 Troubleshooting

### 401 Unauthorized Response
When you see this JSON response:
```json
{
  "error": "Unauthorized",
  "message": "Authentication required. Please provide a valid Clerk session token.",
  "code": "UNAUTHENTICATED"
}
```

**Common causes:**
- Your Clerk session token may have expired (tokens typically expire after a certain period)
- No token was provided in the request
- Token is invalid or malformed

**Solutions:**
- Get a new token from your frontend application
- Update the `auth_token` environment variable in Postman
- Ensure you're setting the token in the Authorization header (should be automatic if using the collection's auth settings)

### "Year does not exist" or "Month does not exist"
- Default years (2024, 2025, 2026) are created during database seeding
- Run "Get All Years" to see available years and their IDs
- Additional years can be created via Prisma Studio (`pnpm run db:studio`)
- Ensure you've created a month before creating budgets/incomes
- Check that the `yearId` and `monthId` are correct

### "Budget does not exist"
- Ensure you've created a budget before creating expenses
- Check that the `budgetId` is correct and belongs to your user

### 401 "User not found" Response
When you see this JSON response:
```json
{
  "error": "Unauthorized",
  "message": "User not found in database. Please ensure your account is synced.",
  "code": "USER_NOT_FOUND"
}
```

**This means:**
- You're authenticated with Clerk (valid token)
- But your user hasn't been synced to our database yet

**Solutions:**
1. Sign in again via your frontend (triggers the `session.created` webhook)
2. Check that your Clerk webhooks are configured correctly
3. Verify your user exists in the database (via Prisma Studio: `pnpm run db:studio`)
4. The webhook should have automatically created your user when you signed up or signed in

### 403 Forbidden (Admin endpoints only)
When you see this JSON response:
```json
{
  "error": "Forbidden",
  "message": "Admin access required. This endpoint is restricted to administrators only.",
  "code": "ADMIN_ACCESS_REQUIRED"
}
```

**This means:**
- You're authenticated successfully
- But you don't have the admin role in Clerk
- Only users with 'admin' or 'org:admin' role can access these endpoints

**How to fix:**
1. Go to Clerk Dashboard → Organizations
2. Ensure you're a member of an organization with the 'admin' role
3. Sign in again to get a new token with the updated role
4. Update your `auth_token` in Postman

### Connection Refused
- Ensure the API server is running locally (`pnpm run dev`)
- Check that the `base_url` matches your server address
- For production, verify the deployment is active

## 📚 Additional Resources

- [API README](../README.md) - Complete API documentation
- [Database Guide](../DB.md) - Database setup and migrations
- [Postman Documentation](https://learning.postman.com/docs/getting-started/introduction/)

## 🎉 Quick Start Checklist

- [ ] Import collection into Postman
- [ ] Import environment files
- [ ] Select appropriate environment (Local/Production)
- [ ] Start your local server (`pnpm run dev`)
- [ ] Run database migrations (`pnpm run db:migrate:dev`)
- [ ] Seed database with default years and categories (`pnpm run db:seed`)
- [ ] **Sign up/Sign in via Clerk frontend** (user automatically syncs via webhook)
- [ ] **Get session token** from your frontend application
- [ ] **Set token in Postman** environment variable `auth_token`
- [ ] Test public endpoints (Years, Categories)
- [ ] Test protected endpoints (Months, Budgets, etc.)
- [ ] Test admin endpoints (if you have admin access)

Happy testing! 🚀

