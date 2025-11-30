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

### 1. Login

First, authenticate to get your JWT token:

1. Navigate to **Authentication → Login**
2. Update the request body with valid credentials:
   ```json
   {
     "email": "your-email@example.com",
     "password": "your-password"
   }
   ```
3. Click **Send**
4. The token will be **automatically saved** to the environment variable `auth_token`

### 2. Use Protected Endpoints

All protected endpoints automatically use the `{{auth_token}}` variable in the `authorization` header.

## 📋 Collection Structure

### Authentication
- **Login** - Get JWT token (saves token automatically)
- **Verify Token** - Check if token is valid

### Public Endpoints
- **Get All Years** - No authentication required
- **Get All Categories** - No authentication required (returns user's categories + admin's default categories if authenticated)

### Protected Endpoints (Require Authentication)

#### Months
- Get All Months (with query params for includes)
- Get Month By ID
- Create Month
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
| `auth_token` | JWT authentication token | Auto-set after login |
| `user_id` | Current user ID | Auto-set after login |

### Switching Environments

- **Local Development**: `http://localhost:3001`
- **Production**: `https://my-finances-api-v4.onrender.com`

## 📝 Sample Data

### Create User (Admin)
```json
{
  "firstName": "John",
  "lastName": "Doe",
  "dateOfBirth": "1990-05-15T00:00:00.000Z",
  "email": "john.doe@example.com",
  "password": "securePassword123",
  "createdAt": "2025-01-01T00:00:00.000Z",
  "genderId": 1
}
```

### Create Month
```json
{
  "value": 1,
  "description": "January 2025",
  "createdAt": "2025-01-01T00:00:00.000Z",
  "yearId": 1
}
```

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

### 1. Auto-Save Token
The Login request has a test script that automatically saves the JWT token and user ID to environment variables. You don't need to copy/paste manually!

### 2. Test All Endpoints Quickly
Use Postman's **Collection Runner** to test all endpoints sequentially:
1. Right-click on the collection
2. Select **Run collection**
3. Configure and run

### 3. Create Data Flow
Follow this order to create a complete data set:
1. Login (get token)
2. **Create Year** (via Prisma Studio - no API endpoint exists yet)
   - Run `pnpm run db:studio`
   - Add a Year record (e.g., 2025)
3. Create Month (requires yearId)
4. Create Budget (requires monthId)
5. Create Expense (requires budgetId and categoryId)
6. Create Income (requires monthId)

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

To test admin endpoints:
1. Login with admin credentials (user ID: 1)
2. The token will be automatically set
3. Admin-only endpoints will now work

**Note**: Regular users cannot access admin endpoints even with a valid token.

## 🐛 Troubleshooting

### "Token is invalid"
- Your token may have expired
- Re-run the **Login** request to get a new token

### "Year does not exist" or "Month does not exist"
- **Years must be created via Prisma Studio** (no API endpoint exists)
  - Run `pnpm run db:studio` and manually add Year records
- Ensure you've created a month before creating budgets/incomes
- Check that the `yearId` and `monthId` are correct

### "Budget does not exist"
- Ensure you've created a budget before creating expenses
- Check that the `budgetId` is correct and belongs to your user

### 401 Unauthorized
- Make sure you've run the **Login** request first
- Check that the `auth_token` environment variable is set
- Verify you're using the correct environment

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
- [ ] Seed database with admin user (`pnpm run db:seed`)
- [ ] **Create at least one Year** via Prisma Studio (`pnpm run db:studio`)
- [ ] Run **Login** request to authenticate
- [ ] Verify token is saved (check environment variables)
- [ ] Test public endpoints (Years, Categories)
- [ ] Test protected endpoints (Months, Budgets, etc.)
- [ ] Test admin endpoints (if you have admin access)

Happy testing! 🚀

