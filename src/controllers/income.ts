import { getUserId } from '../helpers/auth';
import { getQueryTake } from '../helpers/query';
import { asyncHandler } from '../middleware/errorHandler';
import * as incomeService from '../services/income';

export const getAllIncomes = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const incomes = await incomeService.getAllIncomes(userId, getQueryTake(req));
  res.json(incomes);
});

export const createIncome = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const income = await incomeService.createIncome(userId, req.body);
  res.status(201).json(income);
});

export const updateIncome = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const id = Number(req.params.id);
  const income = await incomeService.updateIncome(userId, id, req.body);
  res.json(income);
});

export const deleteIncome = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const id = Number(req.params.id);
  const income = await incomeService.deleteIncome(userId, id);
  res.json(income);
});

export const reorderIncomes = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const incomes = await incomeService.reorderIncomes(userId, req.body);
  res.json(incomes);
});
