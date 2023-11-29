import { Request, Response } from 'express';

import * as service from '../services/year';

export const getAllYears = async (req: Request, res: Response) => {
  res.json(await service.getAllYears(req));
};
