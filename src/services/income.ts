import { Request, Response } from 'express';

import { getQueryTake } from '../helpers/query';
import { getUserId } from '../helpers/auth';
import db from './db';

export const getAllIncomes = async (req: Request, res: Response) => {
  try {
    const userId = await getUserId(req);
    const incomes = await db.income.findMany({
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

    return res.json(incomes);
  } catch {
    return res.json([]);
  }
};

export const createIncome = async (req: Request, res: Response) => {
  try {
    const userId = await getUserId(req);

    const {
      value,
      description,
      createdAt,
      monthId
    } = req.body;

    if (Number.isNaN(value) && Number(value) > 0) {
      return res.status(500).json({
        message: 'Value must be a number and greater than 0.'
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

    // Get the max index for incomes in this month
    const maxIndexIncome = await db.income.findFirst({
      where: {
        monthId: Number(monthId)
      },
      orderBy: {
        index: 'desc'
      }
    });

    const nextIndex = maxIndexIncome ? maxIndexIncome.index + 1 : 0;

    const income = await db.income.create({
      data: {
        value,
        description,
        index: nextIndex,
        createdAt,
        monthId
      }
    });

    return res.json(income);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while creating income.',
      err
    });
  }
};

export const updateIncome = async (req: Request, res: Response) => {
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
      monthId
    } = req.body;

    if (Number.isNaN(value) && Number(value) > 0) {
      return res.status(500).json({
        message: 'Value must be a number and greater than 0.'
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

    const income = await db.income.update({
      where: {
        id: Number(id),
        month: {
          userId
        }
      },
      data: {
        value,
        description,
        monthId
      }
    });

    return res.json(income);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while updating income.',
      err
    });
  }
};

export const deleteIncome = async (req: Request, res: Response) => {
  try {
    const { id } = req.params;
    if (!id) {
      return res.status(500).json({
        message: 'Id is required.'
      });
    }

    const userId = await getUserId(req);
    
    // Find the income to get its monthId before deleting
    const incomeToDelete = await db.income.findFirst({
      where: { 
        id: Number(id),
        month: { userId }
      }
    });

    if (!incomeToDelete) {
      return res.status(404).json({
        message: 'Income not found.'
      });
    }

    const monthId = incomeToDelete.monthId;

    // Delete the income and reindex remaining incomes in a transaction
    await db.$transaction(async (tx) => {
      await tx.income.delete({
        where: { id: Number(id) }
      });

      // Get remaining incomes in the month, ordered by index
      const remainingIncomes = await tx.income.findMany({
        where: { monthId },
        orderBy: { index: 'asc' }
      });

      // Reindex them sequentially
      for (let i = 0; i < remainingIncomes.length; i++) {
        await tx.income.update({
          where: { id: remainingIncomes[i].id },
          data: { index: i }
        });
      }
    });

    return res.json(incomeToDelete);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while deleting income.',
      err
    });
  }
};

export const reorderIncomes = async (req: Request, res: Response) => {
  try {
    const userId = await getUserId(req);
    const { monthId, incomeIds } = req.body;

    if (!monthId || !Array.isArray(incomeIds)) {
      return res.status(400).json({
        message: 'monthId and incomeIds array are required.'
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

    // Verify all incomes belong to this month and user
    const incomes = await db.income.findMany({
      where: {
        id: { in: incomeIds.map(id => Number(id)) },
        monthId: Number(monthId),
        month: { userId }
      }
    });

    if (incomes.length !== incomeIds.length) {
      return res.status(400).json({
        message: 'Some incomes do not exist or do not belong to this month.'
      });
    }

    // Update indices in a transaction
    await db.$transaction(
      incomeIds.map((incomeId, index) =>
        db.income.update({
          where: { id: Number(incomeId) },
          data: { index }
        })
      )
    );

    // Return updated incomes
    const updatedIncomes = await db.income.findMany({
      where: {
        monthId: Number(monthId),
        month: { userId }
      },
      orderBy: { index: 'asc' }
    });

    return res.json(updatedIncomes);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while reordering incomes.',
      err
    });
  }
};