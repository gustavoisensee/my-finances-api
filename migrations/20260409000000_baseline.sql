-- Baseline of the schema previously managed by Prisma (migrations
-- 20251207213731_init_postgres, 20251215114037_add_budget_color,
-- 20260402120000_firebase_auth, plus the "index" columns that were applied
-- with `prisma db push`).
--
-- Every statement is idempotent so this can run against the existing
-- database: it only fills gaps and records itself in _sqlx_migrations.

CREATE TABLE IF NOT EXISTS "AccessToken" (
    "id" SERIAL NOT NULL,
    "name" TEXT NOT NULL,
    "value" TEXT NOT NULL,
    "expirationDate" TIMESTAMP(3) NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL,
    CONSTRAINT "AccessToken_pkey" PRIMARY KEY ("id")
);

CREATE TABLE IF NOT EXISTS "Year" (
    "id" SERIAL NOT NULL,
    "value" INTEGER NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL,
    CONSTRAINT "Year_pkey" PRIMARY KEY ("id")
);

CREATE TABLE IF NOT EXISTS "Category" (
    "id" SERIAL NOT NULL,
    "name" TEXT NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL,
    "userId" INTEGER,
    CONSTRAINT "Category_pkey" PRIMARY KEY ("id")
);

CREATE TABLE IF NOT EXISTS "User" (
    "id" SERIAL NOT NULL,
    "firebaseUid" TEXT,
    "clerkId" TEXT,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "User_pkey" PRIMARY KEY ("id")
);

CREATE TABLE IF NOT EXISTS "Month" (
    "id" SERIAL NOT NULL,
    "value" INTEGER NOT NULL,
    "description" TEXT NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL,
    "yearId" INTEGER NOT NULL,
    "userId" INTEGER NOT NULL,
    CONSTRAINT "Month_pkey" PRIMARY KEY ("id")
);

CREATE TABLE IF NOT EXISTS "Income" (
    "id" SERIAL NOT NULL,
    "description" TEXT,
    "value" DOUBLE PRECISION NOT NULL,
    "index" INTEGER NOT NULL DEFAULT 0,
    "createdAt" TIMESTAMP(3) NOT NULL,
    "monthId" INTEGER NOT NULL,
    CONSTRAINT "Income_pkey" PRIMARY KEY ("id")
);

CREATE TABLE IF NOT EXISTS "Budget" (
    "id" SERIAL NOT NULL,
    "description" TEXT NOT NULL,
    "value" DOUBLE PRECISION NOT NULL,
    "color" TEXT,
    "index" INTEGER NOT NULL DEFAULT 0,
    "createdAt" TIMESTAMP(3) NOT NULL,
    "monthId" INTEGER NOT NULL,
    CONSTRAINT "Budget_pkey" PRIMARY KEY ("id")
);

CREATE TABLE IF NOT EXISTS "Expense" (
    "id" SERIAL NOT NULL,
    "description" TEXT NOT NULL,
    "value" DOUBLE PRECISION NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL,
    "budgetId" INTEGER NOT NULL,
    "categoryId" INTEGER NOT NULL,
    CONSTRAINT "Expense_pkey" PRIMARY KEY ("id")
);

-- Columns added after the initial Prisma migration.
ALTER TABLE "User" ADD COLUMN IF NOT EXISTS "firebaseUid" TEXT;
ALTER TABLE "User" ALTER COLUMN "clerkId" DROP NOT NULL;
ALTER TABLE "Budget" ADD COLUMN IF NOT EXISTS "color" TEXT;
ALTER TABLE "Budget" ADD COLUMN IF NOT EXISTS "index" INTEGER NOT NULL DEFAULT 0;
ALTER TABLE "Income" ADD COLUMN IF NOT EXISTS "index" INTEGER NOT NULL DEFAULT 0;

CREATE UNIQUE INDEX IF NOT EXISTS "User_clerkId_key" ON "User"("clerkId");
CREATE UNIQUE INDEX IF NOT EXISTS "User_firebaseUid_key" ON "User"("firebaseUid");

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'Category_userId_fkey') THEN
        ALTER TABLE "Category" ADD CONSTRAINT "Category_userId_fkey"
            FOREIGN KEY ("userId") REFERENCES "User"("id") ON DELETE SET NULL ON UPDATE CASCADE;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'Month_yearId_fkey') THEN
        ALTER TABLE "Month" ADD CONSTRAINT "Month_yearId_fkey"
            FOREIGN KEY ("yearId") REFERENCES "Year"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'Month_userId_fkey') THEN
        ALTER TABLE "Month" ADD CONSTRAINT "Month_userId_fkey"
            FOREIGN KEY ("userId") REFERENCES "User"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'Income_monthId_fkey') THEN
        ALTER TABLE "Income" ADD CONSTRAINT "Income_monthId_fkey"
            FOREIGN KEY ("monthId") REFERENCES "Month"("id") ON DELETE CASCADE ON UPDATE CASCADE;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'Budget_monthId_fkey') THEN
        ALTER TABLE "Budget" ADD CONSTRAINT "Budget_monthId_fkey"
            FOREIGN KEY ("monthId") REFERENCES "Month"("id") ON DELETE CASCADE ON UPDATE CASCADE;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'Expense_categoryId_fkey') THEN
        ALTER TABLE "Expense" ADD CONSTRAINT "Expense_categoryId_fkey"
            FOREIGN KEY ("categoryId") REFERENCES "Category"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'Expense_budgetId_fkey') THEN
        ALTER TABLE "Expense" ADD CONSTRAINT "Expense_budgetId_fkey"
            FOREIGN KEY ("budgetId") REFERENCES "Budget"("id") ON DELETE CASCADE ON UPDATE CASCADE;
    END IF;
END $$;
