import { Request, Response } from 'express';

import { getQueryTake } from '../helpers/query';
import { getUserId } from '../helpers/auth';
import db from './db';

export const getAllIncomes = async (req: Request, res: Response) => {
  try {
    const userId = getUserId(req);
    const incomes = await db.income.findMany({
      take: getQueryTake(req),
      where: {
        month: {
          userId
        }
      }
    });

    return res.json(incomes);
  } catch {
    return res.json([]);
  }
};

export const createIncome = async (req: Request, res: Response) => {
  try {
    const userId = getUserId(req);

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

    const income = await db.income.create({
      data: {
        value,
        description,
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
    const userId = getUserId(req);

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

    const oldData = await db.income.findFirst({
      where: {
        id: Number(id)
      }
    }) 
    const income = await db.income.update({
      where: {
        id: Number(id),
        month: {
          userId
        }
      },
      data: {
        ...oldData,
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

    const userId = getUserId(req);
    const income = await db.income.delete({
      where: { id: Number(id), month: { userId }}
    });

    return res.json(income);
  } catch (err) {
    return res.status(500).json({
      message: 'Error while deleting income.',
      err
    });
  }
}