import { Request, Response } from 'express';

import * as service from '../services/category'

/**
 * Category Controller
 * 
 * GET /category - Public endpoint that returns user's categories + admin's default categories
 */
export const getAllCategories = async (req: Request, res: Response) => {
  await service.getAllCategories(req, res);
};
