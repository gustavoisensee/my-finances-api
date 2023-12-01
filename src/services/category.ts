import { Request, Response } from 'express';

import { getQueryTake } from '../helpers/query';
import { getUserId } from '../helpers/auth';
import db, { DEFAULT_ADMIN_USER_ID } from './db';

export const getAllCategories = async (req: Request, res: Response) => {
  try {
    const userId = getUserId(req);

    const budgets = await db.category.findMany({
      take: getQueryTake(req),
      where: {
        OR: [
          {
            userId,
          },
          {
            userId: DEFAULT_ADMIN_USER_ID
          }
        ]
      }
    });

    return res.json(budgets);
  } catch {
    return res.json([]);
  }
};
