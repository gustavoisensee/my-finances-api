import { Request } from 'express';

import { getQueryTake } from '../helpers/query';
import db from './db';

export const getAllExpenses = async (req: Request) => {
  try {
    const budgets = await db.expense.findMany({
      take: getQueryTake(req)
    });

    return budgets;
  } catch {
    return [];
  }
};
