import express, { Express, Request, Response } from 'express';
import cors from 'cors';

import { getAllAccessTokens } from '../controllers/accessToken';
import { createBudget, deleteBudget, getAllBudgets, updateBudget } from '../controllers/budget';
import { getAllCategories } from '../controllers/category';
import { createExpense, deleteExpense, getAllExpenses, updateExpense } from '../controllers/expense';
import { createIncome, deleteIncome, getAllIncomes, updateIncome } from '../controllers/income';
import { createMonth, updateMonth, getAllMonths, deleteMonth, getMonthById } from '../controllers/month';
import { createUser, deleteUser, getAllUsers, updateUser } from '../controllers/user';
import { authenticate, mustBeAuthenticated, mustBeAuthenticatedAdmin, verify, syncClerkUser, handleClerkWebhook } from '../controllers/auth';
import { getAllYears } from '../controllers/year';

export const initMiddleware = (app: Express) => {
  app.use(cors({
    origin: [
      'http://localhost:3000',
      'https://mf-v4.gustavoisensee.io'
    ]
  }));
  app.use(express.json());
};

export const initWebhookRoutes = (app: Express) => {
  // Webhook endpoint needs raw body for signature verification
  // MUST be registered before express.json() middleware
  // Note: CORS is not needed for webhooks (server-to-server), but we add it anyway
  app.post('/webhooks/clerk', 
    cors({
      origin: '*', // Webhooks come from Clerk's servers
      methods: ['POST']
    }),
    express.raw({ type: 'application/json' }), 
    handleClerkWebhook
  );
};

export const initAuthenticationRoutes = (app: Express) => {
  app.get('/', (req: Request, res: Response) => {
    res.send('Index');
  });
  
  // Auth endpoints
  app.post('/auth', authenticate);
  app.get('/auth/verify', verify);
  app.post('/auth/sync', syncClerkUser); // Kept as fallback
};

export const initNotAuthenticatedRoutes = (app: Express) => {
  app.get('/year', getAllYears);
  app.get('/category', getAllCategories);
};

export const initAuthenticatedRoutes = (app: Express) => {
  // -- Authenticates ALL endpoints below this line.
  app.use(mustBeAuthenticated);

  // --- Authenticated routes --------------
  // Budget routes
  app.get('/budget', getAllBudgets);
  app.post('/budget', createBudget);
  app.put('/budget/:id', updateBudget);
  app.delete('/budget/:id', deleteBudget);

  // Expense routes
  app.get('/expense', getAllExpenses);
  app.post('/expense', createExpense);
  app.put('/expense/:id', updateExpense);
  app.delete('/expense/:id', deleteExpense);

  // Income routes
  app.get('/income', getAllIncomes);
  app.post('/income', createIncome);
  app.put('/income/:id', updateIncome);
  app.delete('/income/:id', deleteIncome);

  // Month routes
  app.get('/month', getAllMonths);
  app.get('/month/:id', getMonthById);
  app.post('/month', createMonth);
  app.put('/month/:id', updateMonth);
  app.delete('/month/:id', deleteMonth);


  // --- Admin routes ----------------------
  // Access tokens routes
  app.get('/access-token', mustBeAuthenticatedAdmin, getAllAccessTokens);

  // User routes
  app.get('/user', mustBeAuthenticatedAdmin, getAllUsers);
  app.post('/user', mustBeAuthenticatedAdmin, createUser);
  app.put('/user/:id', mustBeAuthenticatedAdmin, updateUser);
  app.delete('/user/:id', mustBeAuthenticatedAdmin, deleteUser);
};

export const initOtherRoutes = (app: Express) => {
  // Not supported routes
  app.use('/*', (req, res) => {
    res.send('Not Found');
  })
};
