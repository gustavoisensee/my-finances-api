import { Request, Response } from 'express';

import * as service from '../services/income';

export const getAllIncomes = async (req: Request, res: Response) => {
  res.json(await service.getAllIncomes(req));
};

export const createIncome = async (req: Request, res: Response) => {
  await service.createIncome(req, res);
};

export const updateIncome = async (req: Request, res: Response) => {
  await service.updateIncome(req, res);
};

export const deleteIncome = async (req: Request, res: Response) => {
  await service.deleteIncome(req, res);
};
