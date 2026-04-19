# My Finances API

A RESTful API for personal finance management built with Node.js, Express, TypeScript, and Prisma ORM.

## 🚀 Features

- **User Authentication**: Firebase-based authentication for secure user management
- **Financial Tracking**: 
  - Income management
  - Budget creation and tracking
  - Expense categorization
  - Monthly and yearly financial reports
- **Multi-user Support**: Each user has their own financial data
- **PostgreSQL Database**: Powered by Supabase
- **Type Safety**: Full TypeScript support
- **Modern Stack**: Latest versions of all dependencies

## 📋 Prerequisites

- Node.js (v18 or higher recommended)
- pnpm (v10.17.0 or higher)
- PostgreSQL database (Supabase account)

## 🛠️ Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd my-finances-api
   ```

2. **Install dependencies**
   ```bash
   pnpm install
   ```

3. **Set up environment variables**
   
   Create a `.env` file in the root directory (you can copy from `.env.example`):
   ```bash
   cp .env.example .env
   ```
   
   Update the following variables:
   - `DATABASE_URL`: Your Supabase connection string (pooled)
   - `DIRECT_URL`: Your Supabase direct connection string
   - `FIREBASE_PROJECT_ID`: Your Firebase project ID
   - `FIREBASE_SERVICE_ACCOUNT_JSON`: (optional) Inline JSON credentials for Firebase Admin SDK
   - `GOOGLE_APPLICATION_CREDENTIALS`: (optional) Path to service account key file
   - `ADMIN_FIREBASE_UIDS`: (optional) Comma-separated list of Firebase UIDs with admin access
   - `NODE_PORT`: Port for the server (default: 3001)

4. **Initialize the database**
   
   See [DB.md](./DB.md) for detailed database setup instructions.
   
   Quick start:
   ```bash
   pnpm run db:generate     # Generate Prisma Client
   pnpm run db:migrate:dev  # Run migrations
   pnpm run db:seed         # Seed database with default years and expense categories
   ```
   
   **Important**: 
   - Users are automatically created on their first Firebase sign-in
   - Default years (2024, 2025, 2026) are created during seeding
   
   **Database Management**:
   ```bash
   pnpm run db:clean        # Remove all data from database (preserves schema)
   pnpm run db:seed         # Repopulate with fresh seed data
   pnpm run db:studio       # Open Prisma Studio to browse/edit data
   ```

## 🏃‍♂️ Running the Application

### Development Mode
```bash
pnpm run dev
```
The server will start on `http://localhost:3001` (or your configured port) with hot reload enabled.

### Production Build
```bash
pnpm run build  # Compile TypeScript
pnpm start      # Start the production server
```

## 📚 API Endpoints

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
- `GET /category` - Get all categories (returns user's custom categories + system default categories)

**Budgets**
- `GET /budget` - Get all budgets
- `POST /budget` - Create a budget
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
- `PUT /income/:id` - Update an income
- `DELETE /income/:id` - Delete an income

**Months**
- `GET /month` - Get all months
- `GET /month/:id` - Get month by ID
- `POST /month` - Create a month
- `POST /month/:id/copy` - Copy a month with its incomes and budgets to a new month/year
- `PUT /month/:id` - Update a month
- `DELETE /month/:id` - Delete a month

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
  "message": "Admin access required. This endpoint is restricted to administrators only.",
  "code": "ADMIN_ACCESS_REQUIRED"
}
```

## 🗄️ Database

This project uses Prisma ORM with PostgreSQL (Supabase). For detailed database management instructions, see [DB.md](./DB.md).

## 🔧 Available Scripts

- `pnpm dev` - Start development server with hot reload
- `pnpm build` - Build for production
- `pnpm start` - Start production server
- `pnpm run db:generate` - Generate Prisma Client
- `pnpm run db:migrate` - Deploy migrations to production
- `pnpm run db:migrate:dev` - Create and apply migrations in development
- `pnpm run db:seed` - Seed database with admin user and default data
- `pnpm run db:clean` - Clean all data from the database
- `pnpm run db:studio` - Open Prisma Studio to browse database

## 🌐 CORS Configuration

The API allows requests from:
- `http://localhost:3000` (automatically enabled in development only)
- `https://my-finances-web.gustavoisensee.com` (production frontend)

The localhost origin is automatically added when `NODE_ENV` is not set to `production`. Update the CORS configuration in `src/helpers/routes.ts` to add more origins.

## 🔒 Security Notes

- Configure Firebase Admin SDK credentials via `FIREBASE_SERVICE_ACCOUNT_JSON` (inline JSON) or `GOOGLE_APPLICATION_CREDENTIALS` (file path)
- Keep your `.env` file secure and never commit it to version control
- Use HTTPS in production environments
- Users are automatically created on first sign-in via Firebase ID token verification
- All protected endpoints require a valid Firebase ID token in the `Authorization: Bearer <token>` header
- Admin access is controlled via Firebase custom claims (`admin: true` or `role: 'admin'`) or the `ADMIN_FIREBASE_UIDS` env var

## 📦 Technology Stack

- **Runtime**: Node.js
- **Language**: TypeScript
- **Framework**: Express.js
- **ORM**: Prisma
- **Database**: PostgreSQL (Supabase)
- **Authentication**: Firebase Admin SDK
- **Package Manager**: pnpm

## 🚀 Deployment

### Render.com

This project is configured for deployment on Render.com. The build command installs dependencies and compiles TypeScript.

**Keep-alive script** (prevents Render free tier from sleeping):
```bash
# Run this in a separate terminal or use a cron job
# 60 * 14 = 840 (60s, 14min, 840s)
# render.com spins down after 15min of inactivity
while true; do curl "https://your-app-name.onrender.com/"; sleep 840; done;
```

## 📄 License

MIT

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
