import db from './db';
import { ValidationError, NotFoundError, ForbiddenError } from '../utils/errors';

export const getAllCategories = async (userId: number) => {
  const categories = await db.category.findMany({
    orderBy: { id: 'asc' },
    where: {
      OR: [
        { userId },
        { userId: null },
      ]
    },
    include: {
      _count: {
        select: { expenses: true }
      }
    }
  });

  return categories.map(cat => ({
    id: cat.id,
    name: cat.name,
    createdAt: cat.createdAt,
    userId: cat.userId,
    expenseCount: cat._count.expenses
  }));
};

export const createCategory = async (userId: number, data: { name: string }) => {
  const { name } = data;

  if (!name || !name.trim()) {
    throw new ValidationError('Name is required.');
  }

  const exists = await db.category.findFirst({
    where: { name: name.trim(), userId }
  });
  if (exists) {
    throw new ValidationError('Category name must be unique.');
  }

  const category = await db.category.create({
    data: {
      name: name.trim(),
      userId,
      createdAt: new Date()
    }
  });

  return { ...category, expenseCount: 0 };
};

export const updateCategory = async (userId: number, id: number, data: { name: string }) => {
  const { name } = data;

  if (!name || !name.trim()) {
    throw new ValidationError('Name is required.');
  }

  const existing = await db.category.findFirst({
    where: { id, userId }
  });
  if (!existing) {
    throw new NotFoundError('Category not found or you do not have permission to edit it.');
  }

  const duplicate = await db.category.findFirst({
    where: {
      name: name.trim(),
      userId,
      NOT: { id }
    }
  });
  if (duplicate) {
    throw new ValidationError('Category name must be unique.');
  }

  const updated = await db.category.update({
    where: { id },
    data: { name: name.trim() },
    include: {
      _count: {
        select: { expenses: true }
      }
    }
  });

  return {
    id: updated.id,
    name: updated.name,
    createdAt: updated.createdAt,
    userId: updated.userId,
    expenseCount: updated._count.expenses
  };
};

export const deleteCategory = async (userId: number, id: number) => {
  const existing = await db.category.findFirst({
    where: { id, userId }
  });
  if (!existing) {
    throw new NotFoundError('Category not found or you do not have permission to delete it.');
  }

  const expenseCount = await db.expense.count({
    where: { categoryId: id }
  });
  if (expenseCount > 0) {
    throw new ValidationError(
      `Cannot delete category: it is used by ${expenseCount} expense${expenseCount > 1 ? 's' : ''}.`
    );
  }

  await db.category.delete({ where: { id } });

  return { success: true, message: 'Category deleted successfully' };
};
