import { Request, Response } from 'express';

import { getQueryTake } from '../helpers/query';
import { getUserId } from '../helpers/auth';
import db from './db';

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

    if (!Number.isInteger(value)) {
      return res.status(500).json({
        message: msgNumberMustBe
      });
    }
    if (Number(value) > 12 || Number(value) < 1) {
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

/**
  id,
  value,
  notes,
  createdAt,
  yearId,
  userId,
  year,
  user
 */
