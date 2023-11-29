import { Request, Response } from 'express';

import { getQueryTake } from '../helpers/query';
import db from './db';
import { getUserId } from '../helpers/auth';

export const getAllExpenses = async (req: Request, res: Response) => {
  try {
    const userId = getUserId(req);
    const budgets = await db.expense.findMany({
      take: getQueryTake(req),
      where: {
        budget: {
          month: {
            userId
          }
        }
      }
    });

    return res.json(budgets);
  } catch {
    return res.json([]);
  }
};
