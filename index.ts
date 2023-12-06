import 'dotenv/config';
import express from 'express';
import { initAuthenticatedRoutes, initAuthenticationRoutes, initMiddleware, initNotAuthenticatedRoutes, initOtherRoutes } from './src/helpers/routes';

const app = express();
const port = process.env.NODE_PORT || 3001;

initMiddleware(app);
initAuthenticationRoutes(app);

initNotAuthenticatedRoutes(app);
initAuthenticatedRoutes(app);

initOtherRoutes(app)

app.listen(port, () => {
  console.log(`Example app listening on port ${port}`);
});
