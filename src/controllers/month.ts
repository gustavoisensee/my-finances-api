import { getUserId } from '../helpers/auth';
import { getQueryTake } from '../helpers/query';
import { asyncHandler } from '../middleware/errorHandler';
import * as monthService from '../services/month';

export const getAllMonths = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const { iIncomes, iBudgets, iExpenses, yearId } = req.query;
  const months = await monthService.getAllMonths(
    userId,
    getQueryTake(req),
    Number(yearId),
    {
      iIncomes: iIncomes as string,
      iBudgets: iBudgets as string,
      iExpenses: iExpenses as string,
    }
  );
  res.json(months);
});

export const getMonthById = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const id = Number(req.params.id);
  const { iIncomes, iBudgets, iExpenses } = req.query;
  const month = await monthService.getMonthById(userId, id, {
    iIncomes: iIncomes as string,
    iBudgets: iBudgets as string,
    iExpenses: iExpenses as string,
  });
  res.json(month);
});

export const createMonth = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const month = await monthService.createMonth(userId, req.body);
  res.status(201).json(month);
});

export const updateMonth = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const id = Number(req.params.id);
  const month = await monthService.updateMonth(userId, id, req.body);
  res.json(month);
});

export const deleteMonth = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const id = Number(req.params.id);
  const month = await monthService.deleteMonth(userId, id);
  res.json(month);
});

export const copyMonth = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const id = Number(req.params.id);
  const month = await monthService.copyMonth(userId, id, req.body);
  res.status(201).json(month);
});
