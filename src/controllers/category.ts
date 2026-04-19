import { getUserId } from '../helpers/auth';
import { asyncHandler } from '../middleware/errorHandler';
import * as categoryService from '../services/category';

export const getAllCategories = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const result = await categoryService.getAllCategories(userId);
  res.json(result);
});

export const createCategory = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const category = await categoryService.createCategory(userId, req.body);
  res.status(201).json(category);
});

export const updateCategory = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const id = Number(req.params.id);
  const category = await categoryService.updateCategory(userId, id, req.body);
  res.json(category);
});

export const deleteCategory = asyncHandler(async (req, res) => {
  const userId = await getUserId(req);
  const id = Number(req.params.id);
  const category = await categoryService.deleteCategory(userId, id);
  res.json(category);
});
