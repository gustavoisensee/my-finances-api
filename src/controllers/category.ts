import { Request, Response } from 'express';

import * as service from '../services/category'

export const getAllCategories = async (req: Request, res: Response) => {
  await service.getAllCategories(req, res);
};
