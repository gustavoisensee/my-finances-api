import { Request, Response, NextFunction } from 'express';
import jwt from 'jsonwebtoken';

import * as service from '../services/auth';

export const authenticate = async (req: Request, res: Response) => {
  res.json(await service.authenticate(req));
};

export const verify = (req: Request, res: Response) => {
  res.json(service.verify(req));
};

export const mustBeAuthenticated = (req: Request, res: Response, next: NextFunction) => {
  service.mustBeAuthenticated(req, res, next);
};

export const mustBeAuthenticatedAdmin = (req: Request, res: Response, next: NextFunction) => {
  service.mustBeAuthenticatedAdmin(req, res, next);
};
