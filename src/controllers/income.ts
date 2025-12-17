import { Request, Response } from 'express';

import * as service from '../services/income';

export const getAllIncomes = async (req: Request, res: Response) => {
  await service.getAllIncomes(req, res);
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

export const reorderIncomes = async (req: Request, res: Response) => {
  await service.reorderIncomes(req, res);
};
