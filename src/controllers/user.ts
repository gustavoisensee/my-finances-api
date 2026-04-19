import { asyncHandler } from '../middleware/errorHandler';
import { getQueryTake } from '../helpers/query';
import * as userService from '../services/user';

export const getAllUsers = asyncHandler(async (req, res) => {
  const users = await userService.getAllUsers(getQueryTake(req));
  res.json(users);
});

export const createUser = asyncHandler(async (req, res) => {
  const user = await userService.createUser(req.body.firebaseUid);
  res.status(201).json(user);
});

export const updateUser = asyncHandler(async (req, res) => {
  const id = Number(req.params.id);
  const user = await userService.updateUser(id, req.body.firebaseUid);
  res.json(user);
});

export const deleteUser = asyncHandler(async (req, res) => {
  const id = Number(req.params.id);
  const user = await userService.deleteUser(id);
  res.json(user);
});
