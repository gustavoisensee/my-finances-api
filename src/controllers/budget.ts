import { getUserId } from '../helpers/auth';
import { getQueryTake } from '../helpers/query';
import { asyncHandler } from '../middleware/errorHandler';
import * as budgetService from '../services/budget';

export const getAllBudgets = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const budgets = await budgetService.getAllBudgets(userId, getQueryTake(req));
  res.json(budgets);
});

export const createBudget = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const budget = await budgetService.createBudget(userId, req.body);
  res.status(201).json(budget);
});

export const updateBudget = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const id = Number(req.params.id);
  const budget = await budgetService.updateBudget(userId, id, req.body);
  res.json(budget);
});

export const deleteBudget = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const id = Number(req.params.id);
  const budget = await budgetService.deleteBudget(userId, id);
  res.json(budget);
});

export const reorderBudgets = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const budgets = await budgetService.reorderBudgets(userId, req.body);
  res.json(budgets);
});
