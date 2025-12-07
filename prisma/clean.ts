import { PrismaClient } from '@prisma/client';

const prisma = new PrismaClient();

async function cleanDatabase() {
  console.log('🧹 Starting database cleanup...\n');

  try {
    // Delete in order of dependencies (most dependent first)
    // This respects foreign key constraints

    console.log('🗑️  Deleting Expenses...');
    const expenses = await prisma.expense.deleteMany({});
    console.log(`   ✅ Deleted ${expenses.count} expenses\n`);

    console.log('🗑️  Deleting Budgets...');
    const budgets = await prisma.budget.deleteMany({});
    console.log(`   ✅ Deleted ${budgets.count} budgets\n`);

    console.log('🗑️  Deleting Incomes...');
    const incomes = await prisma.income.deleteMany({});
    console.log(`   ✅ Deleted ${incomes.count} incomes\n`);

    console.log('🗑️  Deleting Months...');
    const months = await prisma.month.deleteMany({});
    console.log(`   ✅ Deleted ${months.count} months\n`);

    console.log('🗑️  Deleting Categories...');
    const categories = await prisma.category.deleteMany({});
    console.log(`   ✅ Deleted ${categories.count} categories\n`);

    console.log('🗑️  Deleting Years...');
    const years = await prisma.year.deleteMany({});
    console.log(`   ✅ Deleted ${years.count} years\n`);

    console.log('🗑️  Deleting Users...');
    const users = await prisma.user.deleteMany({});
    console.log(`   ✅ Deleted ${users.count} users\n`);

    console.log('🗑️  Deleting Access Tokens...');
    const accessTokens = await prisma.accessToken.deleteMany({});
    console.log(`   ✅ Deleted ${accessTokens.count} access tokens\n`);

    console.log('✨ Database cleanup completed successfully!');
    console.log('💡 Run "pnpm db:seed" to populate the database with initial data.');

  } catch (error) {
    console.error('❌ Error during database cleanup:');
    console.error(error);
    process.exit(1);
  }
}

cleanDatabase()
  .catch((e) => {
    console.error('❌ Unexpected error:');
    console.error(e);
    process.exit(1);
  })
  .finally(async () => {
    await prisma.$disconnect();
  });

