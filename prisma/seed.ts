import { PrismaClient } from '@prisma/client';
import bcrypt from 'bcrypt';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config();

const prisma = new PrismaClient();

async function main() {
  console.log('🌱 Starting database seeding...\n');

  // 1. Create default genders if they don't exist
  console.log('📝 Creating default genders...');
  const genders = [
    { id: 1, name: 'Male' },
    { id: 2, name: 'Female' },
    { id: 3, name: 'Other' },
    { id: 4, name: 'Prefer not to say' }
  ];

  for (const gender of genders) {
    await prisma.gender.upsert({
      where: { id: gender.id },
      update: {},
      create: gender
    });
  }
  console.log('✅ Genders created/verified\n');

  // 2. Create admin user
  console.log('👤 Creating admin user...');
  
  const adminEmail = process.env.ADMIN_EMAIL || 'admin@myfinances.com';
  const adminPassword = process.env.ADMIN_PASSWORD;
  const adminFirstName = process.env.ADMIN_FIRST_NAME || 'Admin';
  const adminLastName = process.env.ADMIN_LAST_NAME || 'User';

  if (!adminPassword) {
    console.error('❌ Error: ADMIN_PASSWORD environment variable is required!');
    console.error('Please set ADMIN_PASSWORD in your .env file\n');
    process.exit(1);
  }

  // Hash the password using the same method as the user service
  const hashedPassword = await bcrypt.hash(adminPassword, 10);

  const adminUser = await prisma.user.upsert({
    where: { id: 1 },
    update: {
      email: adminEmail,
      password: hashedPassword,
      firstName: adminFirstName,
      lastName: adminLastName
    },
    create: {
      id: 1,
      email: adminEmail,
      password: hashedPassword,
      firstName: adminFirstName,
      lastName: adminLastName,
      dateOfBirth: new Date('1988-08-22'),
      genderId: 1, // Prefer not to say
      createdAt: new Date()
    }
  });

  console.log('✅ Admin user created/updated:');
  console.log(`   Email: ${adminUser.email}`);
  console.log(`   Name: ${adminUser.firstName} ${adminUser.lastName}`);
  console.log(`   ID: ${adminUser.id}\n`);

  // 3. Create some default categories (optional)
  console.log('🏷️  Creating default categories...');
  const defaultCategories = [
    { name: 'Housing', userId: 1 },
    { name: 'Transportation', userId: 1 },
    { name: 'Food & Dining', userId: 1 },
    { name: 'Utilities', userId: 1 },
    { name: 'Healthcare', userId: 1 },
    { name: 'Entertainment', userId: 1 },
    { name: 'Shopping', userId: 1 },
    { name: 'Personal Care', userId: 1 },
    { name: 'Education', userId: 1 },
    { name: 'Savings & Investments', userId: 1 },
    { name: 'Other', userId: 1 }
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

