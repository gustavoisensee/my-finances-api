import { Request } from 'express';

import { getQueryTake } from '../helpers/query';
import db from './db';

export const getAllAccessTokens = async (req: Request) => {
  try {
    const accessTokens = await db.accessToken.findMany({
      take: getQueryTake(req)
    });

    return accessTokens;
  } catch {
    return [];
  }
};
