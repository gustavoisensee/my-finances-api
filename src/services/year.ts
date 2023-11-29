import { Request } from 'express';

import { getQueryTake } from '../helpers/query';
import db from './db';

export const getAllYears = async (req: Request) => {
  try {
    const years = await db.year.findMany({
      take: getQueryTake(req)
    });

    return years;
  } catch {
    return [];
  }
};
