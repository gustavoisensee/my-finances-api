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
   - `JWT_TOKEN`: A secure random string for JWT signing (legacy, kept for backwards compatibility)
   - `NODE_PORT`: Port for the server (default: 3001)
   - `ADMIN_EMAIL`: Email for the admin user (default: admin@myfinances.com)
   - `ADMIN_PASSWORD`: **Required** - Strong password for admin user
   - `ADMIN_FIRST_NAME`: Admin's first name (optional)
   - `ADMIN_LAST_NAME`: Admin's last name (optional)

4. **Initialize the database**
   
   See [DB.md](./DB.md) for detailed database setup instructions.
   
   Quick start:
   ```bash
   pnpm run db:generate     # Generate Prisma Client
   pnpm run db:migrate:dev  # Run migrations
   pnpm run db:seed         # Seed database with admin user, genders, and default categories
   ```
   
   **Important**: 
   - Before running `db:seed`, make sure you've set the `ADMIN_PASSWORD` in your `.env` file!
   - **Create Years manually** via Prisma Studio (`pnpm run db:studio`) before creating months
   
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

### Authentication
- `POST /auth` - Login and get JWT token (legacy endpoint)
- `GET /auth/verify` - Verify Clerk session token
- `POST /auth/sync` - Manually sync Clerk user to database (fallback)
- `POST /webhooks/clerk` - Clerk webhook endpoint (automatic user sync)

### Public Endpoints
- `GET /year` - Get all years (Note: Create years via Prisma Studio - no POST endpoint available)
- `GET /category` - Get all categories (returns user's + admin's default categories when authenticated)

### Protected Endpoints (Require Authentication)

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

**Admin Endpoints** (Require admin authentication)
- `GET /access-token` - Get all access tokens
- `GET /user` - Get all users
- `POST /user` - Create a user
- `PUT /user/:id` - Update a user
- `DELETE /user/:id` - Delete a user

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
- Set up Clerk webhooks in production for automatic user sync
- Keep `CLERK_WEBHOOK_SECRET` secure - it verifies webhook authenticity
- Set a strong `ADMIN_PASSWORD` before running the database seed
- Keep your `.env` file secure and never commit it to version control
- The default admin user ID is `1` - ensure this user is properly secured
- Use HTTPS in production environments
- Users are automatically synced via webhooks when created/updated in Clerk

## 📦 Technology Stack

- **Runtime**: Node.js
- **Language**: TypeScript
- **Framework**: Express.js
- **ORM**: Prisma
- **Database**: PostgreSQL (Supabase)
- **Authentication**: Clerk (with legacy bcrypt support)
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
