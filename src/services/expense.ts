import { Request, Response } from 'express';

import { getQueryTake } from '../helpers/query';
import db from './db';
import { getUserId } from '../helpers/auth';

export const getAllExpenses = async (req: Request, res: Response) => {
  try {
    const userId = await getUserId(req);
    const budgets = await db.expense.findMany({
      take: getQueryTake(req),
      where: {
        budget: {
          month: {
            userId
          }
        }
      }
    });

    return res.json(budgets);
  } catch {
    return res.json([]);
  }
};

export const createExpense = async (req: Request, res: Response) => {
  try {
    const userId = await getUserId(req);

    const {
      value,
      description,
      createdAt,
      budgetId,
      categoryId
    } = req.body;

    if (Number.isNaN(value) && Number(value) > 0) {
      return res.status(500).json({
        message: 'Value must be a number and greater than 0.'
      });
    }
    if (description?.length < 3) {
      return res.status(500).json({
        message: 'Title must have at least 3 characters.'
      });
    }

    const budget = await db.budget.findFirst({
      where: {
        id: Number(budgetId),
        month: {
          userId
        }
      }
    });

    if (!budget) {
      return res.status(500).json({
        message: 'Budget does not exist.'
      });
    }

    const expense = await db.expense.create({
      data: {
        value,
        description,
        createdAt,
        budgetId,
        categoryId
      }
    });

    return res.json(expense);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while creating expense.',
      err
    });
  }
};

export const updateExpense = async (req: Request, res: Response) => {
  try {
    const { id } = req.params;
    if (!id) {
      return res.status(500).json({
        message: 'Id is required.'
      });
    }
    const userId = await getUserId(req);

    const {
      value,
      description,
      budgetId,
      categoryId
    } = req.body;

    if (Number.isNaN(value) && Number(value) > 0) {
      return res.status(500).json({
        message: 'Value must be a number and greater than 0.'
      });
    }
    if (description?.length < 3) {
      return res.status(500).json({
        message: 'Title must have at least 3 characters.'
      });
    }

    const budget = await db.budget.findFirst({
      where: {
        id: Number(budgetId),
        month: {
          userId
        }
      }
    });

    if (!budget) {
      return res.status(500).json({
        message: 'Budget does not exist.'
      });
    }

    const expense = await db.expense.update({
      where: {
        id: Number(id),
        budget: {
          month: {
            userId
          }
        }
      },
      data: {
        value,
        description,
        budgetId,
        categoryId
      }
    });

    return res.json(expense);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while updating expense.',
      err
    });
  }
};

export const deleteExpense = async (req: Request, res: Response) => {
  try {
    const { id } = req.params;
    if (!id) {
      return res.status(500).json({
        message: 'Id is required.'
      });
    }

    const userId = await getUserId(req);
    const expense = await db.expense.delete({
      where: { id: Number(id), budget: { month: { userId } } }
    });

    return res.json(expense);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while deleting expense.',
      err
    });
  }
}