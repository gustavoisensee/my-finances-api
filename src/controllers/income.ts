import { Request, Response } from 'express';

import * as service from '../services/income';

export const getAllIncomes = async (req: Request, res: Response) => {
  res.json(await service.getAllIncomes(req));
};
