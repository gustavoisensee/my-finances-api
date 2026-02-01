import { Request, Response } from 'express';

import { getQueryTake } from '../helpers/query';
import { getUserId } from '../helpers/auth';
import db from './db';

/**
 * Get all categories
 * 
 * Returns:
 * - If authenticated: User's own categories + system default categories (userId: null)
 * - If not authenticated: Only system default categories (userId: null)
 * 
 * This allows all users to access a set of default categories (created in seed.ts)
 * while also being able to create their own custom categories.
 * 
 * Each category includes an expenseCount to help the frontend determine if it can be deleted.
 */
export const getAllCategories = async (req: Request, res: Response) => {
  try {
    const userId = await getUserId(req);

    const categories = await db.category.findMany({
      take: getQueryTake(req),
      where: {
        OR: [
          { userId },         // User's own categories
          { userId: null }    // System default categories
        ]
      },
      include: {
        _count: {
          select: { expenses: true }
        }
      }
    });

    // Transform to include expenseCount at top level for easier frontend consumption
    const categoriesWithCount = categories.map(cat => ({
      id: cat.id,
      name: cat.name,
      createdAt: cat.createdAt,
      userId: cat.userId,
      expenseCount: cat._count.expenses
    }));

    return res.json(categoriesWithCount);
  } catch {
    return res.json([]);
  }
};

/**
 * Create a new category
 */
export const createCategory = async (req: Request, res: Response) => {
  try {
    const userId = await getUserId(req);
    const { name } = req.body;

    if (!name || !name.trim()) {
      return res.status(400).json({ message: 'Name is required' });
    }

    // Enforce unique name for this user
    const exists = await db.category.findFirst({
      where: { name: name.trim(), userId }
    });

    if (exists) {
      return res.status(400).json({ message: 'Category name must be unique' });
    }

    const category = await db.category.create({
      data: {
        name: name.trim(),
        userId,
        createdAt: new Date()
      }
    });

    return res.status(201).json({
      ...category,
      expenseCount: 0
    });
  } catch (err) {
    return res.status(500).json({ message: 'Failed to create category' });
  }
};

/**
 * Update a category
 * Users can only update their own categories (not system defaults)
 */
export const updateCategory = async (req: Request, res: Response) => {
  try {
    const userId = await getUserId(req);
    const { id } = req.params;
    const { name } = req.body;

    if (!name || !name.trim()) {
      return res.status(400).json({ message: 'Name is required' });
    }

    // Verify the category belongs to this user
    const existing = await db.category.findFirst({
      where: { id: Number(id), userId }
    });

    if (!existing) {
      return res.status(404).json({ message: 'Category not found or you do not have permission to edit it' });
    }

    // Enforce unique name for this user (excluding self)
    const duplicate = await db.category.findFirst({
      where: {
        name: name.trim(),
        userId,
        NOT: { id: Number(id) }
      }
    });

    if (duplicate) {
      return res.status(400).json({ message: 'Category name must be unique' });
    }

    const updated = await db.category.update({
      where: { id: Number(id) },
      data: { name: name.trim() },
      include: {
        _count: {
          select: { expenses: true }
        }
      }
    });

    return res.json({
      id: updated.id,
      name: updated.name,
      createdAt: updated.createdAt,
      userId: updated.userId,
      expenseCount: updated._count.expenses
    });
  } catch (err) {
    return res.status(500).json({ message: 'Failed to update category' });
  }
};

/**
 * Delete a category
 * - Users can only delete their own categories (not system defaults)
 * - Cannot delete if category has any expenses linked to it
 */
export const deleteCategory = async (req: Request, res: Response) => {
  try {
    const userId = await getUserId(req);
    const { id } = req.params;

    // Verify the category belongs to this user
    const existing = await db.category.findFirst({
      where: { id: Number(id), userId }
    });

    if (!existing) {
      return res.status(404).json({ message: 'Category not found or you do not have permission to delete it' });
    }

    // Check for related expenses
    const expenseCount = await db.expense.count({
      where: { categoryId: Number(id) }
    });

    if (expenseCount > 0) {
      return res.status(400).json({
        message: `Cannot delete category: it is used by ${expenseCount} expense${expenseCount > 1 ? 's' : ''}.`
      });
    }

    await db.category.delete({
      where: { id: Number(id) }
    });

    return res.json({ success: true, message: 'Category deleted successfully' });
  } catch (err) {
    return res.status(500).json({ message: 'Failed to delete category' });
  }
};
