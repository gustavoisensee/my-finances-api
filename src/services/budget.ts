import db from './db';
import { ValidationError, NotFoundError } from '../utils/errors';

export const getAllBudgets = async (userId: number, take: number) => {
  return db.budget.findMany({
    take,
    where: { month: { userId } },
    orderBy: { index: 'asc' }
  });
};

export const createBudget = async (userId: number, data: {
  description: string;
  value: number;
  color?: string;
  createdAt: string;
  monthId: number;
}) => {
  const { description, value, color, createdAt, monthId } = data;

  if (isNaN(value) || value <= 0) {
    throw new ValidationError('Value must be a number and greater than 0.');
  }
  if (!description || description.length < 3) {
    throw new ValidationError('Name must have at least 3 characters.');
  }

  const month = await db.month.findFirst({
    where: { id: Number(monthId), userId }
  });
  if (!month) {
    throw new NotFoundError('Month does not exist.');
  }

  const maxIndexBudget = await db.budget.findFirst({
    where: { monthId: Number(monthId) },
    orderBy: { index: 'desc' }
  });
  const nextIndex = maxIndexBudget ? maxIndexBudget.index + 1 : 0;

  return db.budget.create({
    data: { description, value, color, index: nextIndex, createdAt, monthId }
  });
};

export const updateBudget = async (userId: number, id: number, data: {
  description: string;
  value: number;
  color?: string;
  monthId: number;
}) => {
  const { description, value, color, monthId } = data;

  if (isNaN(value) || value <= 0) {
    throw new ValidationError('Value must be a number and greater than 0.');
  }
  if (!description || description.length < 3) {
    throw new ValidationError('Name must have at least 3 characters.');
  }

  const month = await db.month.findFirst({
    where: { id: Number(monthId), userId }
  });
  if (!month) {
    throw new NotFoundError('Month does not exist.');
  }

  return db.budget.update({
    where: { id, month: { userId } },
    data: { description, value, color, monthId }
  });
};

export const deleteBudget = async (userId: number, id: number) => {
  const budgetToDelete = await db.budget.findFirst({
    where: { id, month: { userId } }
  });

  if (!budgetToDelete) {
    throw new NotFoundError('Budget not found.');
  }

  const monthId = budgetToDelete.monthId;

  await db.$transaction(async (tx) => {
    await tx.budget.delete({ where: { id } });

    const remainingBudgets = await tx.budget.findMany({
      where: { monthId },
      orderBy: { index: 'asc' }
    });

    for (let i = 0; i < remainingBudgets.length; i++) {
      await tx.budget.update({
        where: { id: remainingBudgets[i].id },
        data: { index: i }
      });
    }
  });

  return budgetToDelete;
};

export const reorderBudgets = async (userId: number, data: {
  monthId: number;
  budgetIds: number[];
}) => {
  const { monthId, budgetIds } = data;

  if (!monthId || !Array.isArray(budgetIds)) {
    throw new ValidationError('monthId and budgetIds array are required.');
  }

  const month = await db.month.findFirst({
    where: { id: Number(monthId), userId }
  });
  if (!month) {
    throw new NotFoundError('Month does not exist.');
  }

  const budgets = await db.budget.findMany({
    where: {
      id: { in: budgetIds.map(id => Number(id)) },
      monthId: Number(monthId),
      month: { userId }
    }
  });

  if (budgets.length !== budgetIds.length) {
    throw new ValidationError('Some budgets do not exist or do not belong to this month.');
  }

  await db.$transaction(
    budgetIds.map((budgetId, index) =>
      db.budget.update({
        where: { id: Number(budgetId) },
        data: { index }
      })
    )
  );

  return db.budget.findMany({
    where: { monthId: Number(monthId), month: { userId } },
    orderBy: { index: 'asc' }
  });
};
