import { Request, Response, NextFunction } from 'express';

import * as service from '../services/auth';

export const authenticate = async (req: Request, res: Response) => {
  await service.authenticate(req, res);
};

export const verify = (req: Request, res: Response) => {
  service.verify(req, res);
};

export const syncClerkUser = async (req: Request, res: Response) => {
  await service.syncClerkUser(req, res);
};

export const handleClerkWebhook = async (req: Request, res: Response) => {
  await service.handleClerkWebhook(req, res);
};

export const mustBeAuthenticated = (req: Request, res: Response, next: NextFunction) => {
  service.mustBeAuthenticated(req, res, next);
};

export const mustBeAuthenticatedAdmin = (req: Request, res: Response, next: NextFunction) => {
  service.mustBeAuthenticatedAdmin(req, res, next);
};
