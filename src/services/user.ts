import { Request, Response } from 'express';
import bcrypt from 'bcrypt';

import { getQueryTake } from '../helpers/query';
import db from './db';

export const getAllUsers = async (req: Request) => {
  try {
    const users = await db.user.findMany({
      take: getQueryTake(req)
    });

    return users;
  } catch {
    return [];
  }
};

export const createUser = async (req: Request, res: Response) => {
  try {
    const {
      firstName,
      lastName,
      dateOfBirth,
      email,
      password,
      createdAt,
      genderId
    } = req.body;

    if (!email && !password) {
      return res.json({
        message: 'Email and password are required.'
      });
    }

    const hash = await bcrypt.hash(password, 10);

    const user = await db.user.create({
      data: {
        firstName,
        lastName,
        dateOfBirth,
        email,
        password: hash,
        createdAt,
        genderId
      }
    });

    return res.json(user);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while creating user.',
      err
    });
  }
}