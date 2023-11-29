import { Request } from 'express';

import { getQueryTake } from '../helpers/query';
import db from './db';

export const getAllCategories = async (req: Request) => {
  try {
    const budgets = await db.category.findMany({
      take: getQueryTake(req)
    });

    return budgets;
  } catch {
    return [];
  }
};
