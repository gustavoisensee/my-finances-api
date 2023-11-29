import { Request } from 'express';

import { getQueryTake } from '../helpers/query';
import { getUserId } from '../helpers/auth';
import db from './db';

export const getAllMonths = async (req: Request) => {
  try {
    const userId = getUserId(req);
    const months = await db.month.findMany({
      take: getQueryTake(req),
      where: {
        userId
      }
    });

    return months;
  } catch {
    return [];
  }
};
