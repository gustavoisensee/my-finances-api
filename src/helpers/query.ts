import { Request } from 'express';

import { DEFAULT_TAKE } from '../services/db';

export const getQueryTake = (req: Request) => {
  const { take } = req.query;
  
  return Number(take) || DEFAULT_TAKE;
};


