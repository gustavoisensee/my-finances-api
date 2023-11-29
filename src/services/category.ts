import { Request, Response } from 'express';

import { getQueryTake } from '../helpers/query';
import db from './db';

export const getAllCategories = async (req: Request, res: Response) => {
  try {
    const budgets = await db.category.findMany({
      take: getQueryTake(req)
    });

    return res.json(budgets);
  } catch {
    return res.json([]);
  }
};
