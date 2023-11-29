import { Request } from 'express';
import jwt from 'jsonwebtoken';

import { Verified } from '../types';

export const getUserId = (req: Request) => {
  try {
    const { authorization } = req.headers || {};
    
    const verified = jwt.verify(authorization || '', process.env.JWT_TOKEN as string) as Verified

    return verified?.userId || 0;
  } catch {
    return 0;
  }
}