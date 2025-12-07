import { Request, Response } from 'express';

import { getQueryTake } from '../helpers/query';
import db from './db';

/**
 * Get all users
 * Note: This only returns basic sync data (id, clerkId, createdAt)
 * To get user profile data (name, email, etc.), use Clerk's API with the clerkId
 */
export const getAllUsers = async (req: Request, res: Response) => {
  try {
    const users = await db.user.findMany({
      take: getQueryTake(req),
      select: {
        id: true,
        clerkId: true,
        createdAt: true
      }
    });

    return res.json(users);
  } catch {
    return res.json([]);
  }
};

/**
 * Create user
 * Note: Users are automatically created via Clerk webhooks
 * This endpoint is kept for manual/admin user creation if needed
 */
export const createUser = async (req: Request, res: Response) => {
  try {
    const { clerkId } = req.body;

    if (!clerkId) {
      return res.status(400).json({
        message: 'clerkId is required.'
      });
    }

    const user = await db.user.create({
      data: {
        clerkId,
        createdAt: new Date()
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

/**
 * Update user
 * Note: User profile data is managed in Clerk, not in our database
 * This endpoint can only update the clerkId if needed
 */
export const updateUser = async (req: Request, res: Response) => {
  try {
    const { id } = req.params;
    if (!id) {
      return res.status(400).json({
        message: 'Id is required.'
      });
    }

    const { clerkId } = req.body;

    if (!clerkId) {
      return res.status(400).json({
        message: 'clerkId is required.'
      });
    }

    const user = await db.user.update({
      where: { id: Number(id) },
      data: { clerkId }
    });

    return res.json(user);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while updating user.',
      err
    });
  }
};

/**
 * Delete user
 * Note: This will cascade delete all user data (months, budgets, expenses, categories)
 */
export const deleteUser = async (req: Request, res: Response) => {
  try {
    const { id } = req.params;
    if (!id) {
      return res.status(400).json({
        message: 'Id is required.'
      });
    }

    const user = await db.user.delete({
      where: { id: Number(id) }
    });

    return res.json(user);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while deleting user.',
      err
    });
  }
}