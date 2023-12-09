-- AlterTable
ALTER TABLE "Income" ADD COLUMN "description" TEXT;

-- RedefineTables
ALTER TABLE "Month" RENAME COLUMN notes to description;
ALTER TABLE "Budget" RENAME COLUMN name to description;
ALTER TABLE "Expense" RENAME COLUMN title to description;
