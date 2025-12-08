import { Request, Response } from 'express';

import * as service from '../services/month';

export const getAllMonths = async (req: Request, res: Response) => {
  await service.getAllMonths(req, res);
};

export const getMonthById = async (req: Request, res: Response) => {
  await service.getMonthById(req, res);
};

export const createMonth = async (req: Request, res: Response) => {
  service.createMonth(req, res);
};

export const updateMonth = async (req: Request, res: Response) => {
  service.updateMonth(req, res);
};

export const deleteMonth = async (req: Request, res: Response) => {
  service.deleteMonth(req, res);
};

export const copyMonth = async (req: Request, res: Response) => {
  service.copyMonth(req, res);
};
