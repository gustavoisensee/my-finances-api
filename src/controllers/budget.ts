import { Request, Response } from 'express';

import * as service from '../services/budget';

export const getAllBudgets = async (req: Request, res: Response) => {
  res.json(await service.getAllBudgets(req));
};
