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

export const createCategory = async (req: Request, res: Response) => {
  await service.createCategory(req, res);
};

export const updateCategory = async (req: Request, res: Response) => {
  await service.updateCategory(req, res);
};

export const deleteCategory = async (req: Request, res: Response) => {
  await service.deleteCategory(req, res);
};
