# Database Management Guide

This guide covers everything you need to know about managing the database for the My Finances API.

## 📚 Table of Contents

- [Database Overview](#database-overview)
- [Initial Setup](#initial-setup)
- [Environment Configuration](#environment-configuration)
- [Running Migrations](#running-migrations)
- [Creating New Migrations](#creating-new-migrations)
- [Database Schema](#database-schema)
- [Prisma Studio](#prisma-studio)
- [Common Tasks](#common-tasks)
- [Troubleshooting](#troubleshooting)

## 🗄️ Database Overview

This project uses:
- **Database**: PostgreSQL
- **Hosting**: Supabase
- **ORM**: Prisma v6.19.0
- **Migration System**: Prisma Migrate

## 🚀 Initial Setup

### 1. Create a Supabase Project

1. Go to [Supabase](https://supabase.com) and sign up/login
2. Create a new project
3. Wait for the project to be provisioned
4. Navigate to **Settings** → **Database**
5. Copy your connection strings

### 2. Configure Environment Variables

Create a `.env` file in the project root with the following variables:

```env
# Connect to Supabase via connection pooling (for runtime queries)
DATABASE_URL="postgresql://postgres.[PROJECT-REF]:[PASSWORD]@aws-0-[REGION].pooler.supabase.com:6543/postgres?pgbouncer=true"

# Direct connection to the database (required for migrations)
DIRECT_URL="postgresql://postgres.[PROJECT-REF]:[PASSWORD]@aws-0-[REGION].pooler.supabase.com:5432/postgres"

# Server Configuration
NODE_PORT=3001

# JWT Secret (IMPORTANT: Change this to a secure random string)
JWT_TOKEN=your-super-secret-jwt-token-change-this-in-production
```

**Important Notes:**
- Replace `[PROJECT-REF]`, `[PASSWORD]`, and `[REGION]` with your actual Supabase credentials
- The `DATABASE_URL` uses port 6543 with connection pooling (PgBouncer)
- The `DIRECT_URL` uses port 5432 for direct connection (required for migrations)
- Note: `DIRECT_URL` is currently commented out in `schema.prisma` - uncomment if you encounter migration issues
- Keep these credentials secure and never commit them to version control

### 3. Generate Prisma Client

Generate the Prisma Client to interact with your database:

```bash
pnpm run db:generate
# or
npx prisma generate
```

This creates the type-safe Prisma Client based on your schema.

### 4. Initialize the Database

Run the initial migration to create all tables:

```bash
pnpm run db:migrate:dev
# or
npx prisma migrate dev
```

This will:
- Create all tables according to the schema
- Update the `_prisma_migrations` table
- Regenerate the Prisma Client

### 5. Seed the Database

After running migrations, seed the database with initial data:

```bash
pnpm run db:seed
# or
tsx prisma/seed.ts
```

This will create:
- Default genders (Male, Female, Other, Prefer not to say)
- Admin user (ID: 1)
- Default expense categories for the admin user (Housing, Transportation, Food & Dining, etc.)

**Note**: Years, Months, Budgets, Expenses, and Incomes are NOT seeded automatically. 
- **Years**: Must be created manually via Prisma Studio or direct database insert (no POST endpoint exists yet)
- **Months, Budgets, Expenses, Incomes**: Created by users through the API as they use the application

**Important**: Make sure `ADMIN_PASSWORD` is set in your `.env` file before running the seed script!

## 🔄 Running Migrations

### Development Environment

```bash
pnpm run db:migrate:dev
# or
npx prisma migrate dev
```

This command will:
1. Create a new migration (if schema changed)
2. Apply all pending migrations
3. Regenerate Prisma Client
4. Update your database

### Production/Staging Environment

```bash
pnpm run db:migrate
# or
npx prisma migrate deploy
```

This command:
- Only applies existing migrations
- Does NOT create new migrations
- Does NOT regenerate Prisma Client automatically
- Safe for CI/CD pipelines

## ✏️ Creating New Migrations

### Step 1: Update the Prisma Schema

Edit `prisma/schema.prisma` to add/modify your database structure.

Example - Adding a new field:
```prisma
model User {
  id            Int       @id @default(autoincrement())
  firstName     String
  lastName      String
  phoneNumber   String?   // New field
  // ... rest of the fields
}
```

### Step 2: Create and Apply the Migration

```bash
npx prisma migrate dev --name add_phone_number_to_user
```

This will:
1. Create a new migration file in `prisma/migrations/`
2. Generate the SQL for your changes
3. Apply the migration to your database
4. Regenerate the Prisma Client

### Step 3: Review the Generated SQL

Check the generated SQL in `prisma/migrations/[timestamp]_[name]/migration.sql` to ensure it matches your expectations.

### Step 4: Commit the Migration

```bash
git add prisma/migrations
git add prisma/schema.prisma
git commit -m "Add phone number field to user table"
```

## 📊 Database Schema

### Current Models

**User**
- Stores user authentication and profile information
- Links to: Gender, Month, Category

**Gender**
- Lookup table for user genders

**Year**
- Represents a financial year
- Links to: Month

**Month**
- Represents a financial month within a year
- Belongs to: User, Year
- Links to: Income, Budget

**Category**
- Expense categories (can be user-specific)
- Belongs to: User (optional)
- Links to: Expense

**Income**
- Income entries for a month
- Belongs to: Month
- Cascade deletes with Month

**Budget**
- Budget allocations for a month
- Belongs to: Month
- Links to: Expense
- Cascade deletes with Month

**Expense**
- Individual expenses within a budget
- Belongs to: Budget, Category
- Cascade deletes with Budget

**AccessToken**
- API access tokens (for admin management)

### Relationships

```
User
├── Months (one-to-many)
├── Categories (one-to-many)
└── Gender (many-to-one)

Year
└── Months (one-to-many)

Month
├── Incomes (one-to-many, cascade delete)
└── Budgets (one-to-many, cascade delete)

Budget
└── Expenses (one-to-many, cascade delete)

Category
└── Expenses (one-to-many)
```

## 🎨 Prisma Studio

Prisma Studio is a visual database browser:

```bash
pnpm run db:studio
# or
npx prisma studio
```

This opens a browser at `http://localhost:5555` where you can:
- View all your data
- Add, edit, and delete records
- Filter and search
- Explore relationships

## 🔧 Common Tasks

### Creating Years

Years must be created manually as there's no API endpoint for creating them yet. You can use Prisma Studio:

```bash
pnpm run db:studio
```

Then:
1. Navigate to the **Year** model
2. Click **Add record**
3. Fill in:
   - `value`: The year number (e.g., 2025)
   - `createdAt`: Current timestamp
4. Click **Save**

Or use direct SQL in Prisma Studio's query tab:
```sql
INSERT INTO "Year" (value, "createdAt") VALUES (2025, NOW());
```

### Clean Database Data (Development Only)

Remove all data from the database while preserving the schema:

```bash
pnpm run db:clean
# or
tsx prisma/clean.ts
```

This will:
- Delete all records from all tables in the correct order (respecting foreign keys)
- Preserve the database schema and structure
- Useful for testing or starting fresh with new seed data

After cleaning, you can repopulate with:
```bash
pnpm run db:seed
```

### Reset the Database (Development Only)

**⚠️ WARNING: This will delete ALL data and recreate the database!**

```bash
npx prisma migrate reset
```

This will:
1. Drop the database
2. Create a new database
3. Apply all migrations
4. Run seed script (if you have a seed configuration in `package.json`)

### Check Migration Status

```bash
npx prisma migrate status
```

Shows which migrations have been applied and which are pending.

### Create Migration Without Applying

```bash
npx prisma migrate dev --create-only
```

Useful when you want to review/edit the SQL before applying.

### Format Prisma Schema

```bash
npx prisma format
```

Auto-formats your `schema.prisma` file.

### Validate Schema

```bash
npx prisma validate
```

Checks your schema for errors without making changes.

### Generate Prisma Client Only

```bash
pnpm run db:generate
# or
npx prisma generate
```

Regenerates the Prisma Client without running migrations.

## 🐛 Troubleshooting

### "Can't reach database server"

**Problem**: Prisma can't connect to the database.

**Solutions**:
1. Verify your `.env` file has the correct connection strings
2. Check that your Supabase project is running
3. Ensure you're using the correct URL:
   - Use `DIRECT_URL` for migrations
   - Use `DATABASE_URL` for runtime queries
4. Check your network connection and firewall settings
5. Verify the Supabase project isn't paused (free tier pauses after inactivity)

### "Environment variable not found"

**Problem**: Prisma can't find required environment variables.

**Solutions**:
1. Ensure `.env` file exists (not `.env.local`)
2. Check that all required variables are defined:
   - `DATABASE_URL`
   - `DIRECT_URL`
3. Restart your terminal/IDE after creating `.env`

### "Migration failed"

**Problem**: A migration failed to apply.

**Solutions**:
1. Check the error message for details
2. Review the generated SQL in the migration file
3. Manually fix the database if needed
4. Mark the migration as applied: `npx prisma migrate resolve --applied [migration-name]`
5. Or roll back: `npx prisma migrate resolve --rolled-back [migration-name]`

### "Provider mismatch" (sqlite vs postgresql)

**Problem**: You're switching from SQLite to PostgreSQL.

**Solutions**:
1. Backup old migrations: `mv prisma/migrations prisma/migrations_backup`
2. Update `schema.prisma` to use `postgresql` provider
3. Create new migrations: `npx prisma migrate dev --name init_postgresql`

### "Can't generate Prisma Client"

**Problem**: Prisma Client generation fails.

**Solutions**:
1. Delete `node_modules/.prisma` and try again
2. Run `pnpm install` to ensure all dependencies are installed
3. Check for syntax errors in `schema.prisma`: `npx prisma validate`

## 📝 Best Practices

1. **Always Review Generated SQL**: Check migration files before applying to production
2. **Use Descriptive Migration Names**: Use clear names like `add_user_phone_number` not just `update`
3. **Never Edit Applied Migrations**: Create a new migration instead
4. **Keep Migrations Small**: Make incremental changes rather than large schema overhauls
5. **Test Migrations**: Always test on development/staging before production
6. **Backup Before Major Changes**: Take database backups before running destructive migrations
7. **Use Prisma Migrate in Development**: Don't modify the database manually
8. **Commit Migrations**: Always commit migration files to version control

## 🔐 Production Deployment Checklist

- [ ] Update `JWT_TOKEN` to a secure random string
- [ ] Set a strong `ADMIN_PASSWORD` for the admin user
- [ ] Verify `DATABASE_URL` and `DIRECT_URL` are correct (uncomment `DIRECT_URL` in schema.prisma if needed)
- [ ] Run `pnpm run db:generate` to generate Prisma Client
- [ ] Run `pnpm run db:migrate` to apply migrations
- [ ] Run `pnpm run db:seed` to create admin user and initial data
- [ ] Test database connection
- [ ] Change admin password after first login
- [ ] Set up database backups
- [ ] Configure connection pooling if needed
- [ ] Set up monitoring for database performance

## 📖 Additional Resources

- [Prisma Documentation](https://www.prisma.io/docs)
- [Prisma Migrate Guide](https://www.prisma.io/docs/concepts/components/prisma-migrate)
- [Supabase Documentation](https://supabase.com/docs)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)

