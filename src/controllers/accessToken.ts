import { Request, Response } from 'express';

import * as service from '../services/accessTokens';

export const getAllAccessTokens = async (req: Request, res: Response) => {
  res.json(await service.getAllAccessTokens(req));
};
