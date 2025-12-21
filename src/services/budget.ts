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
      },
      orderBy: {
        index: 'asc'
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

    // Get the max index for budgets in this month
    const maxIndexBudget = await db.budget.findFirst({
      where: {
        monthId: Number(monthId)
      },
      orderBy: {
        index: 'desc'
      }
    });

    const nextIndex = maxIndexBudget ? maxIndexBudget.index + 1 : 0;

    const budget = await db.budget.create({
      data: {
        description,
        value,
        color,
        index: nextIndex,
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

    const budget = await db.budget.update({
      where: {
        id: Number(id),
        month: {
          userId
        }
      },
      data: {
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
    
    // Find the budget to get its monthId before deleting
    const budgetToDelete = await db.budget.findFirst({
      where: { 
        id: Number(id),
        month: { userId }
      }
    });

    if (!budgetToDelete) {
      return res.status(404).json({
        message: 'Budget not found.'
      });
    }

    const monthId = budgetToDelete.monthId;

    // Delete the budget and reindex remaining budgets in a transaction
    await db.$transaction(async (tx) => {
      await tx.budget.delete({
        where: { id: Number(id) }
      });

      // Get remaining budgets in the month, ordered by index
      const remainingBudgets = await tx.budget.findMany({
        where: { monthId },
        orderBy: { index: 'asc' }
      });

      // Reindex them sequentially
      for (let i = 0; i < remainingBudgets.length; i++) {
        await tx.budget.update({
          where: { id: remainingBudgets[i].id },
          data: { index: i }
        });
      }
    });

    return res.json(budgetToDelete);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while deleting budget.',
      err
    });
  }
};

export const reorderBudgets = async (req: Request, res: Response) => {
  try {
    const userId = await getUserId(req);
    const { monthId, budgetIds } = req.body;

    if (!monthId || !Array.isArray(budgetIds)) {
      return res.status(400).json({
        message: 'monthId and budgetIds array are required.'
      });
    }

    // Verify the month belongs to the user
    const month = await db.month.findFirst({
      where: {
        id: Number(monthId),
        userId
      }
    });

    if (!month) {
      return res.status(404).json({
        message: 'Month does not exist.'
      });
    }

    // Verify all budgets belong to this month and user
    const budgets = await db.budget.findMany({
      where: {
        id: { in: budgetIds.map(id => Number(id)) },
        monthId: Number(monthId),
        month: { userId }
      }
    });

    if (budgets.length !== budgetIds.length) {
      return res.status(400).json({
        message: 'Some budgets do not exist or do not belong to this month.'
      });
    }

    // Update indices in a transaction
    await db.$transaction(
      budgetIds.map((budgetId, index) =>
        db.budget.update({
          where: { id: Number(budgetId) },
          data: { index }
        })
      )
    );

    // Return updated budgets
    const updatedBudgets = await db.budget.findMany({
      where: {
        monthId: Number(monthId),
        month: { userId }
      },
      orderBy: { index: 'asc' }
    });

    return res.json(updatedBudgets);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while reordering budgets.',
      err
    });
  }
};