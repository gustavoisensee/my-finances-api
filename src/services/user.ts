import { Request, Response } from 'express';
import bcrypt from 'bcrypt';

import { getQueryTake } from '../helpers/query';
import db from './db';

export const getAllUsers = async (req: Request, res: Response) => {
  try {
    const users = await db.user.findMany({
      take: getQueryTake(req),
      select: {
        id: true,
        firstName: true,
        lastName: true,
        dateOfBirth: true,
        email: true,
        password: false,
        createdAt: true,
        genderId: true
      }
    });

    return res.json(users);
  } catch {
    return res.json([]);
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
      select: {
        id: true,
        firstName: true,
        lastName: true,
        dateOfBirth: true,
        email: true,
        password: false,
        createdAt: true,
        genderId: true
      },
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

export const updateUser = async (req: Request, res: Response) => {
  try {
    const { id } = req.params;
    if (!id) {
      return res.status(500).json({
        message: 'Id is required.'
      });
    }

    const {
      firstName,
      lastName,
      dateOfBirth,
      email,
      createdAt,
      genderId
    } = req.body;

    if (!email) {
      return res.status(500).json({
        message: 'Email is required.'
      });
    }

    const oldData = await db.user.findFirst({ where: { id: Number(id) }});
    const user = await db.user.update({
      where: { id: Number(id) },
      select: {
        id: true,
        firstName: true,
        lastName: true,
        dateOfBirth: true,
        email: true,
        password: false,
        createdAt: true,
        genderId: true
      },
      data: {
        ...oldData,
        firstName,
        lastName,
        dateOfBirth,
        email,
        genderId
      }
    });

    return res.json(user);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while updating user.',
      err
    });
  }
};

export const deleteUser = async (req: Request, res: Response) => {
  try {
    const { id } = req.params;
    if (!id) {
      return res.status(500).json({
        message: 'Id is required.'
      });
    }
    if (Number(id) === 1) {
      return res.status(500).json({
        message: 'Admin can not be deleted.'
      });
    }

    const user = await db.user.delete({
      where: { id: Number(id) },
      select: {
        id: true,
        firstName: true,
        lastName: true,
        dateOfBirth: true,
        email: true,
        password: false,
        createdAt: true,
        genderId: true
      }
    });

    return res.json(user);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while deleting user.',
      err
    });
  }
}