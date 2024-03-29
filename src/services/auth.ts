import { NextFunction, Request, Response } from 'express';
import jwt from 'jsonwebtoken';
import bcrypt from 'bcrypt';

import db, { DEFAULT_ADMIN_USER_ID } from './db';
import { Verified } from '../types';

export const authenticate = async (req: Request, res: Response) => {
  try {
    const { email, password } = req.body || {};
    if (!email || !password) return res.status(500).json('User or password not provided!');

    const user = await db.user.findUnique({
      where: {
        email
      }
    });

    if (!user) return res.status(500).json('Email is invalid!');

    const result = await bcrypt.compare(password, user.password);

    if (!result) return res.status(500).json('Password is invalid!');

    const token = jwt.sign(
      { userId: user?.id },
      process.env.JWT_TOKEN as string
    );

    return res.json({ token, userId: user.id, userName: user.firstName });
  } catch {
    return res.status(500).json('Something went wrong, try again!');
  }
};

export const verify = (req: Request, res: Response): string | jwt.JwtPayload => {
  try {
    const { authorization } = req.headers || {};
    if (!authorization) return res.json('Token not provided!');

    const verified = jwt.verify(authorization, process.env.JWT_TOKEN as string);

    return res.json(verified);
  } catch (e) {
    return res.json('Token is invalid!');
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

const isAuthAdmin = (verified: Verified) => verified?.userId === DEFAULT_ADMIN_USER_ID;
export const mustBeAuthenticatedAdmin = (req: Request, res: Response, next: NextFunction) => {
  isAuthenticated(req, res, next, isAuthAdmin);
}