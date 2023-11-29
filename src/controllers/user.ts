import { Request, Response } from 'express';

import * as service from '../services/user';

export const getAllUsers = async (req: Request, res: Response) => {
  res.json(await service.getAllUsers(req));
};
