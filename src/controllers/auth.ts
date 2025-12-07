import { Request, Response } from 'express';

import * as clerkService from '../services/auth.clerk';

/**
 * Authentication Controller
 * 
 * Handles Clerk webhook events for automatic user synchronization.
 * This is the only authentication-related endpoint in the API.
 * 
 * Endpoint: POST /webhooks/clerk
 * 
 * All other authentication is handled by Clerk's middleware (requireAuth)
 * which validates session tokens on protected routes.
 */
export const handleClerkWebhook = async (req: Request, res: Response) => {
  await clerkService.handleClerkWebhook(req, res);
};
