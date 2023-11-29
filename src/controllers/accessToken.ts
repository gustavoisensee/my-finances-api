import { Request, Response } from 'express';

import * as service from '../services/accessTokens';

export const getAllAccessTokens = async (req: Request, res: Response) => {
  await service.getAllAccessTokens(req, res);
};
