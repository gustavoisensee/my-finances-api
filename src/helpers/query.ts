import { Request } from 'express';

import { DEFAULT_TAKE } from '../services/db';

const MAX_TAKE = 100;

export const getQueryTake = (req: Request) => {
  const { take } = req.query;
  const value = Number(take) || DEFAULT_TAKE;
  return Math.min(value, MAX_TAKE);
};


