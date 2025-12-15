import { Request, Response } from 'express';

import { getQueryTake } from '../helpers/query';
import { getUserId } from '../helpers/auth';
import db from './db';

export const getAllBudgets = async (req: Request, res: Response) => {
  try {
    const userId = await getUserId(req);
    const budgets = await db.budget.findMany({
      take: getQueryTake(req),
      where: {
        month: {
          userId
        }
      }
    });

    return res.json(budgets);
  } catch {
    return res.json([]);
  }
};

export const createBudget = async (req: Request, res: Response) => {
  try {
    const userId = await getUserId(req);

    const {
      description,
      value,
      color,
      createdAt,
      monthId
    } = req.body;

    if (Number.isNaN(value) && Number(value) > 0) {
      return res.status(500).json({
        message: 'Value must be a number and greater than 0.'
      });
    }
    if (description?.length < 3) {
      return res.status(500).json({
        message: 'Name must have at least 3 characters.'
      });
    }

    const month = await db.month.findFirst({
      where: {
        id: Number(monthId),
        userId
      }
    });

    if (!month) {
      return res.status(500).json({
        message: 'Month does not exist.'
      });
    }

    const budget = await db.budget.create({
      data: {
        description,
        value,
        color,
        createdAt,
        monthId
      }
    });

    return res.json(budget);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while creating budget.',
      err
    });
  }
};


export const updateBudget = async (req: Request, res: Response) => {
  try {
    const { id } = req.params;
    if (!id) {
      return res.status(500).json({
        message: 'Id is required.'
      });
    }
    const userId = await getUserId(req);

    const {
      description,
      value,
      color,
      monthId
    } = req.body;

    if (Number.isNaN(value) && Number(value) > 0) {
      return res.status(500).json({
        message: 'Value must be a number and greater than 0.'
      });
    }
    if (description?.length < 3) {
      return res.status(500).json({
        message: 'Name must have at least 3 characters.'
      });
    }

    const month = await db.month.findFirst({
      where: {
        id: Number(monthId),
        userId
      }
    });

    if (!month) {
      return res.status(500).json({
        message: 'Month does not exist.'
      });
    }

    const oldData = await db.budget.findFirst({
      where: {
        id: Number(id)
      }
    }) 
    const budget = await db.budget.update({
      where: {
        id: Number(id),
        month: {
          userId
        }
      },
      data: {
        ...oldData,
        description,
        value,
        color,
        monthId
      }
    });

    return res.json(budget);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while updating budget.',
      err
    });
  }
};

export const deleteBudget = async (req: Request, res: Response) => {
  try {
    const { id } = req.params;
    if (!id) {
      return res.status(500).json({
        message: 'Id is required.'
      });
    }

    const userId = await getUserId(req);
    const budget = await db.budget.delete({
      where: { id: Number(id), month: { userId }}
    });

    return res.json(budget);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while deleting budget.',
      err
    });
  }
}