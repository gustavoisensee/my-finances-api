import { Request, Response } from 'express';

import * as service from '../services/budget';

export const getAllBudgets = async (req: Request, res: Response) => {
  await service.getAllBudgets(req, res);
};

export const createBudget = async (req: Request, res: Response) => {
  await service.createBudget(req, res);
};

export const updateBudget = async (req: Request, res: Response) => {
  await service.updateBudget(req, res);
};

export const deleteBudget = async (req: Request, res: Response) => {
  await service.deleteBudget(req, res);
};