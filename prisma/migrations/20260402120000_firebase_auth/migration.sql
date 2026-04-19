-- AlterTable: add Firebase identity; legacy Clerk id becomes optional
ALTER TABLE "User" ADD COLUMN     "firebaseUid" TEXT;
ALTER TABLE "User" ALTER COLUMN "clerkId" DROP NOT NULL;

CREATE UNIQUE INDEX "User_firebaseUid_key" ON "User"("firebaseUid");
