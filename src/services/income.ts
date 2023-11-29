import { Request } from 'express';

import { getQueryTake } from '../helpers/query';
import { getUserId } from '../helpers/auth';
import db from './db';

export const getAllIncomes = async (req: Request) => {
  try {
    const userId = getUserId(req);
    const incomes = await db.income.findMany({
      take: getQueryTake(req),
      where: {
        month: {
          userId
        }
      }
    });

    return incomes;
  } catch {
    return [];
  }
};
