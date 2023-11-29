import { Request, Response } from 'express';

import * as service from '../services/expense';

export const getAllExpenses = async (req: Request, res: Response) => {
  await service.getAllExpenses(req, res);
};

export const createExpense = async (req: Request, res: Response) => {
  await service.createExpense(req, res);
};

export const updateExpense = async (req: Request, res: Response) => {
  await service.updateExpense(req, res);
};

export const deleteExpense = async (req: Request, res: Response) => {
  await service.deleteExpense(req, res);
};
