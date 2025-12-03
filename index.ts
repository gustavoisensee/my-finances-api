import 'dotenv/config';
import express from 'express';
import { initAuthenticatedRoutes, initAuthenticationRoutes, initMiddleware, initNotAuthenticatedRoutes, initOtherRoutes, initWebhookRoutes } from './src/helpers/routes';

const app = express();
const port = process.env.NODE_PORT || 3001;

// Webhook routes MUST come before express.json() middleware
// They need raw body for signature verification
initWebhookRoutes(app);

// Now initialize other middleware (including express.json())
initMiddleware(app);

// Initialize all other routes
initAuthenticationRoutes(app);
initNotAuthenticatedRoutes(app);
initAuthenticatedRoutes(app);
initOtherRoutes(app);

app.listen(port, () => {
  console.log(`Example app listening on port ${port}`);
});
