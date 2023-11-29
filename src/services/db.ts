import { PrismaClient } from '@prisma/client';

const prisma = new PrismaClient();

export const DEFAULT_TAKE = 1;
export const DEFAULT_TEMP_USER = 1;

export default prisma;
