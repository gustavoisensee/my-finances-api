import express, { Express, Request, Response } from 'express';
import cors from 'cors';
import helmet from 'helmet';

import { getAllAccessTokens } from '../controllers/accessToken';
import { createBudget, deleteBudget, getAllBudgets, reorderBudgets, updateBudget } from '../controllers/budget';
import { createCategory, deleteCategory, getAllCategories, updateCategory } from '../controllers/category';
import { createExpense, deleteExpense, getAllExpenses, updateExpense } from '../controllers/expense';
import { createIncome, deleteIncome, getAllIncomes, reorderIncomes, updateIncome } from '../controllers/income';
import { createMonth, updateMonth, getAllMonths, deleteMonth, getMonthById, copyMonth } from '../controllers/month';
import { createUser, deleteUser, getAllUsers, updateUser } from '../controllers/user';
import { getAllYears } from '../controllers/year';
import { requireAdmin, requireFirebaseAuth } from './auth';
import { errorHandler } from '../middleware/errorHandler';

export const initMiddlewares = (app: Express) => {
  app.use(helmet());

  const allowedOrigins = ['https://my-finances-web.gustavoisensee.com'];
  if (process.env.NODE_ENV !== 'production') {
    allowedOrigins.push('http://localhost:3000');
  }

  app.use(cors({ origin: allowedOrigins }));
  app.use(express.json({ limit: '1mb' }));
};

export const initNotAuthenticatedRoutes = (app: Express) => {
  app.get('/', (req: Request, res: Response) => {
    res.send('Index');
  });
  app.get('/status', (req: Request, res: Response) => {
    res.send('Ok');
  });
};

export const initAuthenticatedRoutes = (app: Express) => {
  app.get('/year', requireFirebaseAuth(), getAllYears);
  app.get('/category', requireFirebaseAuth(), getAllCategories);
  app.post('/category', requireFirebaseAuth(), createCategory);
  app.put('/category/:id', requireFirebaseAuth(), updateCategory);
  app.delete('/category/:id', requireFirebaseAuth(), deleteCategory);

  app.get('/budget', requireFirebaseAuth(), getAllBudgets);
  app.post('/budget', requireFirebaseAuth(), createBudget);
  app.put('/budget/reorder', requireFirebaseAuth(), reorderBudgets);
  app.put('/budget/:id', requireFirebaseAuth(), updateBudget);
  app.delete('/budget/:id', requireFirebaseAuth(), deleteBudget);

  app.get('/expense', requireFirebaseAuth(), getAllExpenses);
  app.post('/expense', requireFirebaseAuth(), createExpense);
  app.put('/expense/:id', requireFirebaseAuth(), updateExpense);
  app.delete('/expense/:id', requireFirebaseAuth(), deleteExpense);

  app.get('/income', requireFirebaseAuth(), getAllIncomes);
  app.post('/income', requireFirebaseAuth(), createIncome);
  app.put('/income/reorder', requireFirebaseAuth(), reorderIncomes);
  app.put('/income/:id', requireFirebaseAuth(), updateIncome);
  app.delete('/income/:id', requireFirebaseAuth(), deleteIncome);

  app.get('/month', requireFirebaseAuth(), getAllMonths);
  app.get('/month/:id', requireFirebaseAuth(), getMonthById);
  app.post('/month', requireFirebaseAuth(), createMonth);
  app.post('/month/:id/copy', requireFirebaseAuth(), copyMonth);
  app.put('/month/:id', requireFirebaseAuth(), updateMonth);
  app.delete('/month/:id', requireFirebaseAuth(), deleteMonth);

  app.get('/access-token', requireFirebaseAuth(), requireAdmin, getAllAccessTokens);

  app.get('/user', requireFirebaseAuth(), requireAdmin, getAllUsers);
  app.post('/user', requireFirebaseAuth(), requireAdmin, createUser);
  app.put('/user/:id', requireFirebaseAuth(), requireAdmin, updateUser);
  app.delete('/user/:id', requireFirebaseAuth(), requireAdmin, deleteUser);
};

export const initOtherRoutes = (app: Express) => {
  app.use('/*', (req: Request, res: Response) => {
    res.status(404).json({ message: 'Not Found' });
  });
};

export const initErrorHandler = (app: Express) => {
  app.use(errorHandler);
};
