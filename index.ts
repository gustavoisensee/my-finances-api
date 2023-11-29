import 'dotenv/config';
import express from 'express';

import { getAllAccessTokens } from './src/controllers/accessToken';
import { getAllBudgets } from './src/controllers/budget';
import { getAllCategories } from './src/controllers/category';
import { getAllExpenses } from './src/controllers/expense';
import { getAllIncomes } from './src/controllers/income';
import { getAllMonths } from './src/controllers/month';
import { createUser, deleteUser, getAllUsers, updateUser } from './src/controllers/user';
import { getAllYears } from './src/controllers/year';
import * as auth from './src/controllers/auth';

const app = express();
const port = 3000;

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

// Expense routes
app.get('/expense', auth.mustBeAuthenticated, getAllExpenses);

// Income routes
app.get('/income', auth.mustBeAuthenticated, getAllIncomes);

// Month routes
app.get('/month', auth.mustBeAuthenticated, getAllMonths);


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
