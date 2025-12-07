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
 */
export const getAllCategories = async (req: Request, res: Response) => {
  try {
    const userId = await getUserId(req);

    const categories = await db.category.findMany({
      take: getQueryTake(req),
      where: {
        OR: [
          {
            userId,           // User's own categories
          },
          {
            userId: null      // System default categories
          }
        ]
      }
    });

    return res.json(categories);
  } catch {
    return res.json([]);
  }
};
