import 'dotenv/config';
import express from 'express';
import {
  initAuthenticatedRoutes,
  initErrorHandler,
  initMiddlewares,
  initNotAuthenticatedRoutes,
  initOtherRoutes,
} from './src/helpers/routes';
import db from './src/services/db';

const app = express();
const port = process.env.PORT || process.env.NODE_PORT || 3001;

initMiddlewares(app);
initNotAuthenticatedRoutes(app);
initAuthenticatedRoutes(app);
initOtherRoutes(app);
initErrorHandler(app);

const server = app.listen(port, () => {
  console.log(`Server listening on port ${port}`);
});

function shutdown() {
  console.log('Shutting down...');
  server.close(async () => {
    await db.$disconnect();
    process.exit(0);
  });
}

process.on('SIGTERM', shutdown);
process.on('SIGINT', shutdown);
