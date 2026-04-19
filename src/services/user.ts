import db from './db';
import { ValidationError } from '../utils/errors';

export const getAllUsers = async (take: number) => {
  return db.user.findMany({
    take,
    select: {
      id: true,
      firebaseUid: true,
      clerkId: true,
      createdAt: true,
    },
  });
};

export const createUser = async (firebaseUid: string) => {
  if (!firebaseUid || typeof firebaseUid !== 'string') {
    throw new ValidationError('firebaseUid is required.');
  }

  return db.user.create({
    data: {
      firebaseUid,
      createdAt: new Date(),
    },
  });
};

export const updateUser = async (id: number, firebaseUid: string) => {
  if (!firebaseUid || typeof firebaseUid !== 'string') {
    throw new ValidationError('firebaseUid is required.');
  }

  return db.user.update({
    where: { id },
    data: { firebaseUid },
  });
};

export const deleteUser = async (id: number) => {
  return db.user.delete({
    where: { id },
  });
};
