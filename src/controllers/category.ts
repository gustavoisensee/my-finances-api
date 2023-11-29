import { Request, Response } from 'express';

import * as service from '../services/category'

export const getAllCategories = async (req: Request, res: Response) => {
  res.json(await service.getAllCategories(req));
};
