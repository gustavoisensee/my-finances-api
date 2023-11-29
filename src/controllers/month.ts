import { Request, Response } from 'express';

import * as service from '../services/month';

export const getAllMonths = async (req: Request, res: Response) => {
  res.json(await service.getAllMonths(req));
};
