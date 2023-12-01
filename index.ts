import 'dotenv/config';
import express from 'express';
import cors from 'cors';

import { getAllAccessTokens } from './src/controllers/accessToken';
import { createBudget, deleteBudget, getAllBudgets, updateBudget } from './src/controllers/budget';
import { getAllCategories } from './src/controllers/category';
import { createExpense, deleteExpense, getAllExpenses, updateExpense } from './src/controllers/expense';
import { createIncome, deleteIncome, getAllIncomes, updateIncome } from './src/controllers/income';
import { createMonth, updateMonth, getAllMonths, deleteMonth } from './src/controllers/month';
import { createUser, deleteUser, getAllUsers, updateUser } from './src/controllers/user';
import { getAllYears } from './src/controllers/year';
import * as auth from './src/controllers/auth';

const app = express();
const port = 3001;

app.use(cors({
  origin: ['http://localhost:3000']
}));
app.use(express.json());

app.get('/', (req: any, res: any) => {
  res.send('Index');
});

// Authentication routes
app.post('/auth', auth.authenticate);

// --- Admin routes ----------------------

// Access tokens routes
app.get('/access-token', auth.mustBeAuthenticatedAdmin, getAllAccessTokens);

// User routes
app.get('/user', auth.mustBeAuthenticatedAdmin, getAllUsers);
app.post('/user', auth.mustBeAuthenticatedAdmin, createUser);
app.put('/user/:id', auth.mustBeAuthenticatedAdmin, updateUser);
app.delete('/user/:id', auth.mustBeAuthenticatedAdmin, deleteUser);


// --- Authenticated routes --------------

// Auth routes
app.get('/auth/verify', auth.verify);

// Budget routes
app.get('/budget', auth.mustBeAuthenticated, getAllBudgets);
app.post('/budget', auth.mustBeAuthenticated, createBudget);
app.put('/budget/:id', auth.mustBeAuthenticated, updateBudget);
app.delete('/budget/:id', auth.mustBeAuthenticated, deleteBudget);

// Expense routes
app.get('/expense', auth.mustBeAuthenticated, getAllExpenses);
app.post('/expense', auth.mustBeAuthenticated, createExpense);
app.put('/expense/:id', auth.mustBeAuthenticated, updateExpense);
app.delete('/expense/:id', auth.mustBeAuthenticated, deleteExpense);

// Income routes
app.get('/income', auth.mustBeAuthenticated, getAllIncomes);
app.post('/income', auth.mustBeAuthenticated, createIncome);
app.put('/income/:id', auth.mustBeAuthenticated, updateIncome);
app.delete('/income/:id', auth.mustBeAuthenticated, deleteIncome);

// Month routes
app.get('/month', auth.mustBeAuthenticated, getAllMonths);
app.post('/month', auth.mustBeAuthenticated, createMonth);
app.put('/month/:id', auth.mustBeAuthenticated, updateMonth);
app.delete('/month/:id', auth.mustBeAuthenticated, deleteMonth);


// --- Not Authenticated routes -----------

// Year routes
app.get('/year', auth.mustBeAuthenticated, getAllYears);

// Category routes
app.get('/category', auth.mustBeAuthenticated, getAllCategories);

app.use('/*', (req, res) => {
  res.send('Not Found');
})

app.listen(port, () => {
  console.log(`Example app listening on port ${port}`);
});
