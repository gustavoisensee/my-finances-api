import { Request, Response } from 'express';

import * as service from '../services/year';

export const getAllYears = async (req: Request, res: Response) => {
  await service.getAllYears(req, res);
};
