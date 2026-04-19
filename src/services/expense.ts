import db from './db';
import { ValidationError, NotFoundError } from '../utils/errors';

export const getAllExpenses = async (userId: number, take: number) => {
  return db.expense.findMany({
    take,
    where: { budget: { month: { userId } } }
  });
};

export const createExpense = async (userId: number, data: {
  value: number;
  description: string;
  createdAt: string;
  budgetId: number;
  categoryId: number;
}) => {
  const { value, description, createdAt, budgetId, categoryId } = data;

  if (isNaN(value) || value <= 0) {
    throw new ValidationError('Value must be a number and greater than 0.');
  }
  if (!description || description.length < 3) {
    throw new ValidationError('Title must have at least 3 characters.');
  }

  const budget = await db.budget.findFirst({
    where: { id: Number(budgetId), month: { userId } }
  });
  if (!budget) {
    throw new NotFoundError('Budget does not exist.');
  }

  return db.expense.create({
    data: { value, description, createdAt, budgetId, categoryId }
  });
};

export const updateExpense = async (userId: number, id: number, data: {
  value: number;
  description: string;
  budgetId: number;
  categoryId: number;
}) => {
  const { value, description, budgetId, categoryId } = data;

  if (isNaN(value) || value <= 0) {
    throw new ValidationError('Value must be a number and greater than 0.');
  }
  if (!description || description.length < 3) {
    throw new ValidationError('Title must have at least 3 characters.');
  }

  const budget = await db.budget.findFirst({
    where: { id: Number(budgetId), month: { userId } }
  });
  if (!budget) {
    throw new NotFoundError('Budget does not exist.');
  }

  return db.expense.update({
    where: { id, budget: { month: { userId } } },
    data: { value, description, budgetId, categoryId }
  });
};

export const deleteExpense = async (userId: number, id: number) => {
  return db.expense.delete({
    where: { id, budget: { month: { userId } } }
  });
};
