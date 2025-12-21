import { Request, Response } from 'express';

import { getQueryTake } from '../helpers/query';
import { getUserId } from '../helpers/auth';
import db from './db';

const isMonthValid = (value: number) =>
  !Number.isInteger(value) || (Number(value) > 12 || Number(value) < 1)

// i = include
type Params = {
  iIncomes?: string;
  iBudgets?: string;
  iExpenses?: string;
  yearId?: number;
}

const isValid = (bool: string) => bool?.toLowerCase() === 'true';

const getMonthName = (value: number): string => {
  const months = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December'
  ];
  return months[value - 1] || 'Unknown';
};

export const getAllMonths = async (req: Request, res: Response) => {
  try {
    const {
      iIncomes = 'false',
      iBudgets = 'false',
      iExpenses = 'false',
      yearId
    }: Params = req.query;

    if (!yearId) {
      return res.status(500).json('yearId must be passed!');
    }

    const userId = await getUserId(req);
    const months = await db.month.findMany({
      take: getQueryTake(req),
      include: {
        incomes: isValid(iIncomes),
        budgets: isValid(iBudgets) ? {
          include: {
            expenses: isValid(iExpenses)
          }
        } : false
      },
      where: {
        userId,
        yearId: Number(yearId)
      }
    });

    return res.json(months);
  } catch {
    return res.json([]);
  }
};

export const getMonthById = async (req: Request, res: Response) => {
  try {
    const {
      iIncomes = 'false',
      iBudgets = 'false',
      iExpenses = 'false',
    }: Params = req.query;

    const { id } = req.params;
    if (!id) {
      return res.status(500).json({
        message: 'Id is required.'
      });
    }

    const userId = await getUserId(req);
    const months = await db.month.findFirst({
      include: {
        incomes: isValid(iIncomes),
        budgets: isValid(iBudgets) ? {
          include: {
            expenses: isValid(iExpenses)
          }
        } : false
      },
      where: {
        userId,
        id: Number(id)
      }
    });

    return res.json(months);
  } catch {
    return res.json([]);
  }
};

export const createMonth = async (req: Request, res: Response) => {
  try {
    const msgNumberMustBe = 'Value must be a number between 1 and 12.';
    const userId = await getUserId(req);

    const {
      value,
      description,
      createdAt,
      yearId,
    } = req.body;

    if (isMonthValid(value)) {
      return res.status(500).json({
        message: msgNumberMustBe
      });
    }

    const month = await db.month.create({
      data: {
        value,
        description,
        createdAt,
        yearId,
        userId
      }
    });

    return res.json(month);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while creating month.',
      err
    });
  }
};

export const updateMonth = async (req: Request, res: Response) => {
  try {
    const { id } = req.params;
    if (!id) {
      return res.status(500).json({
        message: 'Id is required.'
      });
    }
    const msgNumberMustBe = 'Value must be a number between 1 and 12.';
    const userId = await getUserId(req);

    const {
      value,
      description,
      yearId,
    } = req.body;

    if (isMonthValid(value)) {
      return res.status(500).json({
        message: msgNumberMustBe
      });
    }

    const oldData = await db.month.findFirst({ where: { id: Number(id) } });
    const month = await db.month.update({
      where: { id: Number(id), userId },
      data: {
        ...oldData,
        value,
        description,
        yearId,
        userId
      }
    });

    return res.json(month);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while updating month.',
      err
    });
  }
};

export const deleteMonth = async (req: Request, res: Response) => {
  try {
    const { id } = req.params;
    if (!id) {
      return res.status(500).json({
        message: 'Id is required.'
      });
    }

    const userId = await getUserId(req);
    const month = await db.month.delete({
      where: { id: Number(id), userId }
    });

    return res.json(month);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while deleting month.',
      err
    });
  }
};

export const copyMonth = async (req: Request, res: Response) => {
  try {
    const { id } = req.params;
    if (!id) {
      return res.status(500).json({
        message: 'Id is required.'
      });
    }

    const { value, yearId } = req.body;
    const msgNumberMustBe = 'Value must be a number between 1 and 12.';

    if (isMonthValid(value)) {
      return res.status(500).json({
        message: msgNumberMustBe
      });
    }

    if (!yearId) {
      return res.status(500).json({
        message: 'yearId is required.'
      });
    }

    const userId = await getUserId(req);

    // Fetch source month with incomes and budgets (but not expenses)
    const sourceMonth = await db.month.findFirst({
      where: { id: Number(id), userId },
      include: {
        incomes: true,
        budgets: true,
        year: true
      }
    });

    if (!sourceMonth) {
      return res.status(404).json({
        message: 'Source month not found.'
      });
    }

    // Generate description: "Copy from December 2025"
    const monthName = getMonthName(sourceMonth.value);
    const description = `Copy from ${monthName} ${sourceMonth.year.value}`;
    const now = new Date();

    // Create the new month
    const newMonth = await db.month.create({
      data: {
        value,
        description,
        createdAt: now,
        yearId: Number(yearId),
        userId,
        incomes: {
          create: sourceMonth.incomes.map(income => ({
            description: income.description,
            value: income.value,
            index: income.index,
            createdAt: now
          }))
        },
        budgets: {
          create: sourceMonth.budgets.map(budget => ({
            description: budget.description,
            value: budget.value,
            color: budget.color,
            index: budget.index,
            createdAt: now
          }))
        }
      },
      include: {
        incomes: true,
        budgets: true
      }
    });

    return res.json(newMonth);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while copying month.',
      err
    });
  }
};