import db from './db';
import { ValidationError, NotFoundError } from '../utils/errors';

const isMonthInvalid = (value: number) =>
  !Number.isInteger(value) || value > 12 || value < 1;

const isTrue = (val?: string) => val?.toLowerCase() === 'true';

const getMonthName = (value: number): string => {
  const months = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December'
  ];
  return months[value - 1] || 'Unknown';
};

type IncludeOptions = {
  iIncomes?: string;
  iBudgets?: string;
  iExpenses?: string;
};

export const getAllMonths = async (
  userId: number,
  take: number,
  yearId: number,
  includes: IncludeOptions,
) => {
  if (!yearId || isNaN(yearId)) {
    throw new ValidationError('yearId must be passed.');
  }

  return db.month.findMany({
    take,
    include: {
      incomes: isTrue(includes.iIncomes) ? { orderBy: { index: 'asc' } } : false,
      budgets: isTrue(includes.iBudgets) ? {
        orderBy: { index: 'asc' },
        include: {
          expenses: isTrue(includes.iExpenses) ? { orderBy: { id: 'asc' } } : false
        }
      } : false
    },
    where: { userId, yearId }
  });
};

export const getMonthById = async (
  userId: number,
  id: number,
  includes: IncludeOptions,
) => {
  return db.month.findFirst({
    include: {
      incomes: isTrue(includes.iIncomes) ? { orderBy: { index: 'asc' } } : false,
      budgets: isTrue(includes.iBudgets) ? {
        orderBy: { index: 'asc' },
        include: {
          expenses: isTrue(includes.iExpenses) ? { orderBy: { id: 'asc' } } : false
        }
      } : false
    },
    where: { userId, id }
  });
};

export const createMonth = async (userId: number, data: {
  value: number;
  description: string;
  createdAt: string;
  yearId: number;
}) => {
  const { value, description, createdAt, yearId } = data;

  if (isMonthInvalid(value)) {
    throw new ValidationError('Value must be a number between 1 and 12.');
  }

  return db.month.create({
    data: { value, description, createdAt, yearId, userId }
  });
};

export const updateMonth = async (userId: number, id: number, data: {
  value: number;
  description: string;
  yearId: number;
}) => {
  const { value, description, yearId } = data;

  if (isMonthInvalid(value)) {
    throw new ValidationError('Value must be a number between 1 and 12.');
  }

  return db.month.update({
    where: { id, userId },
    data: { value, description, yearId }
  });
};

export const deleteMonth = async (userId: number, id: number) => {
  return db.month.delete({
    where: { id, userId }
  });
};

export const copyMonth = async (userId: number, sourceId: number, data: {
  value: number;
  yearId: number;
}) => {
  const { value, yearId } = data;

  if (isMonthInvalid(value)) {
    throw new ValidationError('Value must be a number between 1 and 12.');
  }
  if (!yearId) {
    throw new ValidationError('yearId is required.');
  }

  const sourceMonth = await db.month.findFirst({
    where: { id: sourceId, userId },
    include: { incomes: true, budgets: true, year: true }
  });

  if (!sourceMonth) {
    throw new NotFoundError('Source month not found.');
  }

  const monthName = getMonthName(sourceMonth.value);
  const description = `Copy from ${monthName} ${sourceMonth.year.value}`;
  const now = new Date();

  return db.month.create({
    data: {
      value,
      description,
      createdAt: now,
      yearId: Number(yearId),
      userId,
      incomes: {
        create: sourceMonth.incomes.map(income => ({
          description: income.description,
          value: income.value,
          index: income.index,
          createdAt: now
        }))
      },
      budgets: {
        create: sourceMonth.budgets.map(budget => ({
          description: budget.description,
          value: budget.value,
          color: budget.color,
          index: budget.index,
          createdAt: now
        }))
      }
    },
    include: { incomes: true, budgets: true }
  });
};
