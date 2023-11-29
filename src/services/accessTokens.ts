import { Request, Response } from 'express';

import { getQueryTake } from '../helpers/query';
import db from './db';

export const getAllAccessTokens = async (req: Request, res: Response) => {
  try {
    const accessTokens = await db.accessToken.findMany({
      take: getQueryTake(req)
    });

    return res.json(accessTokens);
  } catch {
    return res.json([]);
  }
};
