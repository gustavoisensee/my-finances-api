import "dotenv/config";
import express from "express";
import { clerkMiddleware } from "@clerk/express";
import {
  initAuthenticatedRoutes,
  initMiddlewares,
  initNotAuthenticatedRoutes,
  initOtherRoutes,
  initWebhookRoutes,
} from "./src/helpers/routes";

const app = express();
const port = process.env.NODE_PORT || 3001;

/**
 * IMPORTANT: Route initialization order matters!
 *
 * 1. Webhook routes FIRST (before any body parsers)
 *    - Webhooks need raw body for signature verification
 * 2. Clerk middleware (adds auth context to all requests)
 * 3. Regular middleware (express.json(), CORS, etc.)
 * 4. All other routes
 */

// Webhook routes MUST come before express.json() middleware
// They need raw body for signature verification
initWebhookRoutes(app);

// Add Clerk middleware to attach Auth object to all requests
app.use(clerkMiddleware());

// Now initialize other middleware (including express.json())
initMiddlewares(app);

// Initialize all other routes
initNotAuthenticatedRoutes(app);
initAuthenticatedRoutes(app);
initOtherRoutes(app);

app.listen(port, () => {
  console.log(`Example app listening on port ${port}`);
});
