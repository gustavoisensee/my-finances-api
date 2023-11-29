import { Request } from 'express';

import { DEFAULT_TEMP_USER } from '../services/db';

export const getUserId = (req: Request) => {
  // TODO implement jwt decode and get user id from token
  // const authorization = req.headers?.['Authorization'] || '';

  return DEFAULT_TEMP_USER;
}