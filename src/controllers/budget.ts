import { Request, Response } from 'express';

import * as service from '../services/budget';

export const getAllBudgets = async (req: Request, res: Response) => {
  await service.getAllBudgets(req, res);
};
