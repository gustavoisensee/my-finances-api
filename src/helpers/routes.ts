import express, { Express, Request, Response, NextFunction } from 'express';
import cors from 'cors';
import { requireAuth, getAuth } from '@clerk/express';

import { getAllAccessTokens } from '../controllers/accessToken';
import { createBudget, deleteBudget, getAllBudgets, reorderBudgets, updateBudget } from '../controllers/budget';
import { getAllCategories } from '../controllers/category';
import { createExpense, deleteExpense, getAllExpenses, updateExpense } from '../controllers/expense';
import { createIncome, deleteIncome, getAllIncomes, reorderIncomes, updateIncome } from '../controllers/income';
import { createMonth, updateMonth, getAllMonths, deleteMonth, getMonthById, copyMonth } from '../controllers/month';
import { createUser, deleteUser, getAllUsers, updateUser } from '../controllers/user';
import { handleClerkWebhook } from '../controllers/auth';
import { getAllYears } from '../controllers/year';
import { requireAdmin } from './auth';

/**
 * Custom authentication middleware that returns proper 401 JSON response
 * Instead of redirecting or returning plain text, returns a structured JSON error
 */
const customRequireAuth = () => {
  return (req: Request, res: Response, next: NextFunction) => {
    const { userId } = getAuth(req);
    
    if (!userId) {
      return res.status(401).json({
        error: 'Unauthorized',
        message: 'Authentication required. Please provide a valid Clerk session token.',
        code: 'UNAUTHENTICATED'
      });
    }
    
    next();
  };
};

export const initMiddlewares = (app: Express) => {
  // Build allowed origins based on environment
  const allowedOrigins = [
    'https://my-finances-web.gustavoisensee.com'
  ];

  // Add localhost only in development
  if (process.env.NODE_ENV !== 'production') {
    allowedOrigins.push('http://localhost:3000');
  }

  app.use(cors({
    origin: allowedOrigins
  }));
  app.use(express.json());
};

/**
 * Initialize webhook routes
 * 
 * IMPORTANT: Webhook routes must be registered BEFORE express.json() middleware
 * because they need the raw request body for signature verification.
 * 
 * Clerk Webhook Setup:
 * 1. Go to Clerk Dashboard → Webhooks
 * 2. Add endpoint: https://your-domain.com/webhooks/clerk
 * 3. Subscribe to: user.created, user.updated, user.deleted, session.created
 * 4. Copy signing secret to CLERK_WEBHOOK_SECRET in .env
 */
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

export const initNotAuthenticatedRoutes = (app: Express) => {
  app.get('/', (req: Request, res: Response) => {
    res.send('Index');
  });
  app.get('/status', (req: Request, res: Response) => {
    res.send('Ok');
  });
};

/**
 * Initialize authenticated routes
 * 
 * All routes here require a valid Clerk session token.
 * The token should be passed in the Authorization header: "Bearer <token>"
 * 
 * Authentication is handled by customRequireAuth() middleware which:
 * - Checks if user is authenticated via Clerk
 * - Returns 401 JSON response if not authenticated (instead of redirecting)
 * - Allows request to proceed if authenticated
 * 
 * User identification is done via getUserId() helper which maps Clerk ID to internal user ID.
 */
export const initAuthenticatedRoutes = (app: Express) => {
  // --- Authenticated routes --------------
  // Year and Category routes
  app.get('/year', customRequireAuth(), getAllYears);
  app.get('/category', customRequireAuth(), getAllCategories);

  // Budget routes
  app.get('/budget', customRequireAuth(), getAllBudgets);
  app.post('/budget', customRequireAuth(), createBudget);
  app.put('/budget/reorder', customRequireAuth(), reorderBudgets);
  app.put('/budget/:id', customRequireAuth(), updateBudget);
  app.delete('/budget/:id', customRequireAuth(), deleteBudget);

  // Expense routes
  app.get('/expense', customRequireAuth(), getAllExpenses);
  app.post('/expense', customRequireAuth(), createExpense);
  app.put('/expense/:id', customRequireAuth(), updateExpense);
  app.delete('/expense/:id', customRequireAuth(), deleteExpense);

  // Income routes
  app.get('/income', customRequireAuth(), getAllIncomes);
  app.post('/income', customRequireAuth(), createIncome);
  app.put('/income/reorder', customRequireAuth(), reorderIncomes);
  app.put('/income/:id', customRequireAuth(), updateIncome);
  app.delete('/income/:id', customRequireAuth(), deleteIncome);

  // Month routes
  app.get('/month', customRequireAuth(), getAllMonths);
  app.get('/month/:id', customRequireAuth(), getMonthById);
  app.post('/month', customRequireAuth(), createMonth);
  app.post('/month/:id/copy', customRequireAuth(), copyMonth);
  app.put('/month/:id', customRequireAuth(), updateMonth);
  app.delete('/month/:id', customRequireAuth(), deleteMonth);


  // --- Admin routes ----------------------
  // Access tokens routes
  app.get('/access-token', customRequireAuth(), requireAdmin, getAllAccessTokens);

  // User routes
  app.get('/user', customRequireAuth(), requireAdmin, getAllUsers);
  app.post('/user', customRequireAuth(), requireAdmin, createUser);
  app.put('/user/:id', customRequireAuth(), requireAdmin, updateUser);
  app.delete('/user/:id', customRequireAuth(), requireAdmin, deleteUser);
};

export const initOtherRoutes = (app: Express) => {
  // Not supported routes
  app.use('/*', (req, res) => {
    res.send('Not Found');
  })
};
