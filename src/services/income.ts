import db from './db';
import { ValidationError, NotFoundError } from '../utils/errors';

export const getAllIncomes = async (userId: number, take: number) => {
  return db.income.findMany({
    take,
    where: { month: { userId } },
    orderBy: { index: 'asc' }
  });
};

export const createIncome = async (userId: number, data: {
  value: number;
  description?: string;
  createdAt: string;
  monthId: number;
}) => {
  const { value, description, createdAt, monthId } = data;

  if (isNaN(value) || value <= 0) {
    throw new ValidationError('Value must be a number and greater than 0.');
  }

  const month = await db.month.findFirst({
    where: { id: Number(monthId), userId }
  });
  if (!month) {
    throw new NotFoundError('Month does not exist.');
  }

  const maxIndexIncome = await db.income.findFirst({
    where: { monthId: Number(monthId) },
    orderBy: { index: 'desc' }
  });
  const nextIndex = maxIndexIncome ? maxIndexIncome.index + 1 : 0;

  return db.income.create({
    data: { value, description, index: nextIndex, createdAt, monthId }
  });
};

export const updateIncome = async (userId: number, id: number, data: {
  value: number;
  description?: string;
  monthId: number;
}) => {
  const { value, description, monthId } = data;

  if (isNaN(value) || value <= 0) {
    throw new ValidationError('Value must be a number and greater than 0.');
  }

  const month = await db.month.findFirst({
    where: { id: Number(monthId), userId }
  });
  if (!month) {
    throw new NotFoundError('Month does not exist.');
  }

  return db.income.update({
    where: { id, month: { userId } },
    data: { value, description, monthId }
  });
};

export const deleteIncome = async (userId: number, id: number) => {
  const incomeToDelete = await db.income.findFirst({
    where: { id, month: { userId } }
  });

  if (!incomeToDelete) {
    throw new NotFoundError('Income not found.');
  }

  const monthId = incomeToDelete.monthId;

  await db.$transaction(async (tx) => {
    await tx.income.delete({ where: { id } });

    const remainingIncomes = await tx.income.findMany({
      where: { monthId },
      orderBy: { index: 'asc' }
    });

    for (let i = 0; i < remainingIncomes.length; i++) {
      await tx.income.update({
        where: { id: remainingIncomes[i].id },
        data: { index: i }
      });
    }
  });

  return incomeToDelete;
};

export const reorderIncomes = async (userId: number, data: {
  monthId: number;
  incomeIds: number[];
}) => {
  const { monthId, incomeIds } = data;

  if (!monthId || !Array.isArray(incomeIds)) {
    throw new ValidationError('monthId and incomeIds array are required.');
  }

  const month = await db.month.findFirst({
    where: { id: Number(monthId), userId }
  });
  if (!month) {
    throw new NotFoundError('Month does not exist.');
  }

  const incomes = await db.income.findMany({
    where: {
      id: { in: incomeIds.map(id => Number(id)) },
      monthId: Number(monthId),
      month: { userId }
    }
  });

  if (incomes.length !== incomeIds.length) {
    throw new ValidationError('Some incomes do not exist or do not belong to this month.');
  }

  await db.$transaction(
    incomeIds.map((incomeId, index) =>
      db.income.update({
        where: { id: Number(incomeId) },
        data: { index }
      })
    )
  );

  return db.income.findMany({
    where: { monthId: Number(monthId), month: { userId } },
    orderBy: { index: 'asc' }
  });
};
