import { Request } from 'express';

import { getQueryTake } from '../helpers/query';
import db from './db';

export const getAllUsers = async (req: Request) => {
  try {
    const users = await db.user.findMany({
      take: getQueryTake(req)
    });

    return users;
  } catch {
    return [];
  }
};
