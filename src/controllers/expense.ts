import { Request, Response } from 'express';

import * as service from '../services/expense';

export const getAllExpenses = async (req: Request, res: Response) => {
  res.json(await service.getAllExpenses(req));
};
