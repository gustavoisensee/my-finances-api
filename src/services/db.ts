import { PrismaClient } from '@prisma/client';

const prisma = new PrismaClient();

// Default number of records to return when querying lists (pagination)
export const DEFAULT_TAKE = 10;

export default prisma;
