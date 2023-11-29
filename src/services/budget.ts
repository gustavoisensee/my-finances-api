import { Request } from 'express';

import { getQueryTake } from '../helpers/query';
import { getUserId } from '../helpers/auth';
import db from './db';

export const getAllBudgets = async (req: Request) => {
  try {
    const userId = getUserId(req);
    const budgets = await db.budget.findMany({
      take: getQueryTake(req),
      where: {
        month: {
          userId
        }
      }
    });

    return budgets;
  } catch {
    return [];
  }
};
