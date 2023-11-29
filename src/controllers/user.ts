import { Request, Response } from 'express';

import * as service from '../services/user';

export const getAllUsers = async (req: Request, res: Response) => {
  res.json(await service.getAllUsers(req));
};

export const createUser = async (req: Request, res: Response) => {
  await service.createUser(req, res);
};

export const updateUser = async (req: Request, res: Response) => {
  await service.updateUser(req, res);
};

export const deleteUser = async (req: Request, res: Response) => {
  await service.deleteUser(req, res);
};
