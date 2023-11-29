import express from 'express';

import { getAllAccessTokens } from './src/controllers/accessToken';
import { getAllBudgets } from './src/controllers/budget';
import { getAllCategories } from './src/controllers/category';
import { getAllExpenses } from './src/controllers/expense';
import { getAllIncomes } from './src/controllers/income';
import { getAllMonths } from './src/controllers/month';
import { getAllUsers } from './src/controllers/user';
import { getAllYears } from './src/controllers/year';

const app = express();
const port = 3000;

app.use(express.json());

app.get('/', (req: any, res: any) => {
  res.send('Index');
});

// Access tokens routes
app.get('/access-token', getAllAccessTokens);

// Budget routes
app.get('/budget', getAllBudgets);

// Category routes
app.get('/category', getAllCategories);

// Expense routes
app.get('/expense', getAllExpenses);

// Income routes
app.get('/income', getAllIncomes);

// Month routes
app.get('/month', getAllMonths);

// User routes
app.get('/user', getAllUsers);

// Year routes
app.get('/year', getAllYears);

app.use('/*', (req, res) => {
  res.send('Not Found');
})

app.listen(port, () => {
  console.log(`Example app listening on port ${port}`);
});
