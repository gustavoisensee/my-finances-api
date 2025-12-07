# My Finances API

A RESTful API for personal finance management built with Node.js, Express, TypeScript, and Prisma ORM.

## 🚀 Features

- **User Authentication**: Clerk-based authentication for secure user management
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
   - `CLERK_SECRET_KEY`: Your Clerk secret key from the Clerk Dashboard
   - `CLERK_WEBHOOK_SECRET`: Your Clerk webhook signing secret (from Clerk Dashboard → Webhooks)
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
   - Users are automatically synced via Clerk webhooks when they sign up/sign in
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

### Webhooks
- `POST /webhooks/clerk` - Clerk webhook endpoint (automatic user sync on user creation/update/deletion/sign-in)

### Public Endpoints
- `GET /` - API index
- `GET /status` - API status check

### Protected Endpoints (Require Authentication)

All protected endpoints return a **401 Unauthorized** JSON response if no valid Clerk session token is provided:
```json
{
  "error": "Unauthorized",
  "message": "Authentication required. Please provide a valid Clerk session token.",
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
- `PUT /month/:id` - Update a month
- `DELETE /month/:id` - Delete a month

**Admin Endpoints** (Require admin role in Clerk)
- `GET /access-token` - Get all access tokens
- `GET /user` - Get all users (returns minimal user data: id, clerkId, createdAt)
- `POST /user` - Create a user manually (users are auto-created via webhooks)
- `PUT /user/:id` - Update a user's clerkId
- `DELETE /user/:id` - Delete a user and all their data

**Admin Access Setup:**
1. Enable Organizations in Clerk Dashboard
2. Create an organization
3. Add users to the organization with the 'admin' role
4. Users with 'admin' or 'org:admin' role can access these endpoints

**Note on User Data:**
- User profile data (name, email) is stored in Clerk, not in the database
- The database only stores minimal sync data (id, clerkId, createdAt)
- To get user profile information, use Clerk's API with the clerkId

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
- `http://localhost:3000` (local development)
- `https://mf-v4.gustavoisensee.io` (production frontend)

Update the CORS configuration in `src/helpers/routes.ts` to add more origins.

## 🔒 Security Notes

- Always use a valid `CLERK_SECRET_KEY` from your Clerk Dashboard
- **Set up Clerk webhooks** in production for automatic user sync:
  - Go to Clerk Dashboard → Webhooks
  - Add endpoint: `https://your-domain.com/webhooks/clerk`
  - Subscribe to events: `user.created`, `user.updated`, `user.deleted`, `session.created`
  - Copy the webhook signing secret to `CLERK_WEBHOOK_SECRET` in your `.env` file
- **Set up Clerk Organizations** for admin role management:
  - Enable Organizations in Clerk Dashboard
  - Create an organization for your admins
  - Assign the 'admin' or 'org:admin' role to admin users
- Keep `CLERK_WEBHOOK_SECRET` secure - it verifies webhook authenticity
- Keep your `.env` file secure and never commit it to version control
- Use HTTPS in production environments
- Users are automatically synced via webhooks when created/updated/signed-in in Clerk
- All protected endpoints require a valid Clerk session token in the Authorization header
- Admin access is controlled via Clerk roles, not database user IDs
- User profile data is stored in Clerk - the database only stores relational sync data

## 📦 Technology Stack

- **Runtime**: Node.js
- **Language**: TypeScript
- **Framework**: Express.js
- **ORM**: Prisma
- **Database**: PostgreSQL (Supabase)
- **Authentication**: Clerk
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
