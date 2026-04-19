import { getUserId } from '../helpers/auth';
import { getQueryTake } from '../helpers/query';
import { asyncHandler } from '../middleware/errorHandler';
import * as expenseService from '../services/expense';

export const getAllExpenses = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const expenses = await expenseService.getAllExpenses(userId, getQueryTake(req));
  res.json(expenses);
});

export const createExpense = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const expense = await expenseService.createExpense(userId, req.body);
  res.status(201).json(expense);
});

export const updateExpense = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const id = Number(req.params.id);
  const expense = await expenseService.updateExpense(userId, id, req.body);
  res.json(expense);
});

export const deleteExpense = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const id = Number(req.params.id);
  const expense = await expenseService.deleteExpense(userId, id);
  res.json(expense);
});
