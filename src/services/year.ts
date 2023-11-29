import { Request, Response } from 'express';

import { getQueryTake } from '../helpers/query';
import db from './db';

export const getAllYears = async (req: Request, res: Response) => {
  try {
    const years = await db.year.findMany({
      take: getQueryTake(req)
    });

    return res.json(years);
  } catch {
    return res.json([]);
  }
};
