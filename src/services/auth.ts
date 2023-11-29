import { NextFunction, Request, Response } from 'express';
import jwt from 'jsonwebtoken';

import db from './db';
import { Verified } from '../types';

export const authenticate = async (req: Request): Promise<string> => {
  try {
    const { email, password } = req.body || {};
    if (!email || !password) return 'User or password not provided!';

    const user = await db.user.findUnique({
      where: {
        email,
        password
      }
    });

    if (!user) return 'User or password invalid!'

    const token = jwt.sign(
      { userId: user?.id },
      process.env.JWT_TOKEN as string
    );

    return token;
  } catch {
    return 'Something went wrong, try again!';
  }
};

export const verify = (req: Request): string | jwt.JwtPayload => {
  try {
    const { authorization } = req.headers || {};
    if (!authorization) return 'Token not provided!';

    const verified = jwt.verify(authorization, process.env.JWT_TOKEN as string);

    return verified;
  } catch (e) {
    return 'Token is invalid!'
  }
}

export const isAuthenticated = (req: Request, res: Response, next: NextFunction, cbVerified: (v: Verified) => boolean) => {
  try {
    const { authorization } = req.headers || {};
    if (!authorization) {
      res.status(401);
      return res.send('Request does not have authentication token.')
    }

    const verified = jwt.verify(authorization, process.env.JWT_TOKEN as string) as Verified
    if (cbVerified(verified)) {
      return next();
    }

    res.send('Something went wrong on the authentication.');
  } catch {
    res.send('Token invalid.');
  }
};

const isAuth = (verified: Verified): boolean => !!verified;
export const mustBeAuthenticated = (req: Request, res: Response, next: NextFunction) => {
  isAuthenticated(req, res, next, isAuth);
};

const isAuthAdmin = (verified: Verified) => verified?.userId === 1;
export const mustBeAuthenticatedAdmin = (req: Request, res: Response, next: NextFunction) => {
  isAuthenticated(req, res, next, isAuthAdmin);
}