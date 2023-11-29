import { Request, Response } from 'express';

import { getQueryTake } from '../helpers/query';
import { getUserId } from '../helpers/auth';
import db from './db';

const isMonthValid = (value: number) =>
  !Number.isInteger(value) || (Number(value) > 12 || Number(value) < 1)

export const getAllMonths = async (req: Request) => {
  try {
    const userId = getUserId(req);
    const months = await db.month.findMany({
      take: getQueryTake(req),
      where: {
        userId
      }
    });

    return months;
  } catch {
    return [];
  }
};

export const createMonth = async (req: Request, res: Response) => {
  try {
    const msgNumberMustBe = 'Value must be a number between 1 and 12.';
    const userId = getUserId(req);

    const {
      value,
      notes,
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
        notes,
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
    const userId = getUserId(req);

    const {
      value,
      notes,
      createdAt,
      yearId,
    } = req.body;

    if (isMonthValid(value)) {
      return res.status(500).json({
        message: msgNumberMustBe
      });
    }

    const oldData = await db.month.findFirst({ where: { id: Number(id) }});
    const month = await db.month.update({
      where: { id: Number(id), userId },
      data: {
        ...oldData,
        value,
        notes,
        createdAt,
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

    const userId = getUserId(req);
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
}