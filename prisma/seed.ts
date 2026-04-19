import { PrismaClient } from '@prisma/client';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config();

const prisma = new PrismaClient();

async function main() {
  console.log('🌱 Starting database seeding...\n');

  // 1. Create default years
  console.log('📅 Creating default years...');
  const defaultYears = [
    { value: 2024 },
    { value: 2025 },
    { value: 2026 }
  ];

  for (const year of defaultYears) {
    const existing = await prisma.year.findFirst({
      where: { value: year.value }
    });

    if (!existing) {
      await prisma.year.create({
        data: {
          ...year,
          createdAt: new Date()
        }
      });
    }
  }
  console.log('✅ Default years created\n');

  // 2. Create default categories available to all users
  // These categories have userId: null, making them system-wide defaults
  // Users can also create their own custom categories
  console.log('🏷️  Creating default categories...');
  const defaultCategories = [
    { name: 'Housing', userId: null },
    { name: 'Transportation', userId: null },
    { name: 'Food & Dining', userId: null },
    { name: 'Utilities', userId: null },
    { name: 'Healthcare', userId: null },
    { name: 'Entertainment', userId: null },
    { name: 'Shopping', userId: null },
    { name: 'Personal Care', userId: null },
    { name: 'Education', userId: null },
    { name: 'Savings & Investments', userId: null },
    { name: 'Other', userId: null }
  ];

  for (const category of defaultCategories) {
    const existing = await prisma.category.findFirst({
      where: {
        name: category.name,
        userId: category.userId
      }
    });

    if (!existing) {
      await prisma.category.create({
        data: {
          ...category,
          createdAt: new Date()
        }
      });
    }
  }
  console.log('✅ Default categories created\n');

  console.log('🎉 Database seeding completed successfully!');
  console.log('📊 Summary:');
  console.log(`   - ${defaultYears.length} years (2024-2026)`);
  console.log(`   - ${defaultCategories.length} default categories`);
  console.log('ℹ️  Users are created on first Firebase sign-in (API)');
}

main()
  .catch((e) => {
    console.error('❌ Error during seeding:');
    console.error(e);
    process.exit(1);
  })
  .finally(async () => {
    await prisma.$disconnect();
  });

