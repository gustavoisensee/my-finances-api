import { Request, Response } from 'express';

import db from './db';

/**
 * Clerk Webhook Service
 * 
 * This service handles automatic user synchronization between Clerk and our database.
 * 
 * Webhook Events Handled:
 * - user.created: Creates a new user in our database when they sign up in Clerk
 * - user.updated: Updates user information when changed in Clerk
 * - user.deleted: Soft-deletes user by removing clerkId (preserves user data)
 * - session.created: Ensures user is synced when they sign in (catches users created before webhooks were set up)
 * 
 * Setup Instructions:
 * 1. Go to Clerk Dashboard → Webhooks
 * 2. Add endpoint: https://your-domain.com/webhooks/clerk
 * 3. Subscribe to: user.created, user.updated, user.deleted, session.created
 * 4. Copy signing secret to CLERK_WEBHOOK_SECRET in .env
 * 
 * Authentication Flow:
 * 1. User signs up/signs in via Clerk in frontend
 * 2. Clerk sends webhook to this endpoint
 * 3. User is automatically created/synced in our database
 * 4. User's Clerk session token is used for API authentication
 * 5. Our middleware (requireAuth) validates the token and maps clerkId to internal user ID
 */
export const handleClerkWebhook = async (req: Request, res: Response) => {
  try {
    // Get the body - it's raw buffer from express.raw()
    const payload = req.body;
    const body = Buffer.isBuffer(payload) ? payload.toString('utf8') : JSON.stringify(payload);

    // Parse the JSON payload
    let evt: any;
    try {
      evt = JSON.parse(body);
    } catch (err) {
      console.error('Error parsing webhook body:', err);
      return res.status(400).json({ message: 'Invalid JSON payload' });
    }

    // Handle the webhook
    const eventType = evt.type;
    const userData = evt.data;

    switch (eventType) {
      case 'user.created':
        await handleUserCreated(userData);
        break;
      
      case 'user.updated':
        await handleUserUpdated(userData);
        break;
      
      case 'user.deleted':
        await handleUserDeleted(userData);
        break;
      
      case 'session.created':
        // Sync user on sign-in (handles existing users who weren't synced)
        await handleSessionCreated(userData);
        break;
      
      default:
        console.log(`Unhandled event type: ${eventType}`);
    }

    return res.status(200).json({ message: 'Webhook processed successfully' });
  } catch (error) {
    console.error('Webhook error:', error);
    return res.status(500).json({ 
      message: 'Error processing webhook',
      error: error instanceof Error ? error.message : 'Unknown error'
    });
  }
};

/**
 * Handle user.created webhook event
 */
async function handleUserCreated(userData: any) {
  try {
    const clerkUserId = userData.id;

    // Check if user already exists
    const existingUser = await db.user.findFirst({
      where: { clerkId: clerkUserId }
    });

    if (existingUser) {
      console.log('User already exists:', existingUser.id);
      return;
    }

    // Create new user (minimal data - profile info comes from Clerk)
    const newUser = await db.user.create({
      data: {
        clerkId: clerkUserId,
        createdAt: new Date()
      }
    });

    console.log('Created new user from webhook:', newUser.id);
  } catch (error) {
    console.error('Error handling user.created:', error);
    throw error;
  }
}

/**
 * Handle user.updated webhook event
 * Note: Since we don't store profile data in our DB, there's nothing to update
 * We only check if user exists, and create if needed
 */
async function handleUserUpdated(userData: any) {
  try {
    const clerkUserId = userData.id;

    // Find user by clerkId
    const user = await db.user.findFirst({
      where: { clerkId: clerkUserId }
    });

    if (!user) {
      console.log('User not found, creating new user from update event');
      await handleUserCreated(userData);
      return;
    }

    // User exists - no profile data to update (it's all in Clerk)
    console.log('User update received, but no DB fields to sync:', user.id);
  } catch (error) {
    console.error('Error handling user.updated:', error);
    throw error;
  }
}

/**
 * Handle session.created webhook event
 * This fires on every sign-in, ensuring users are synced even if created before webhooks
 */
async function handleSessionCreated(sessionData: any) {
  try {
    console.log('handleSessionCreated 1', sessionData);
    const clerkUserId = sessionData.user_id;
    
    if (!clerkUserId) {
      console.error('No user_id found in session data');
      return;
    }

    // Check if user exists
    const user = await db.user.findFirst({
      where: { clerkId: clerkUserId }
    });

    if (user) {
      // User already synced, nothing to do
      console.log('User already synced on sign-in:', user.id);
      return;
    }

    // User not synced yet - create minimal user record
    console.log('User not synced, creating user:', clerkUserId);
    
    const newUser = await db.user.create({
      data: {
        clerkId: clerkUserId,
        createdAt: new Date()
      }
    });

    console.log('Created new user from session webhook:', newUser.id);
    console.log('handleSessionCreated 2');
  } catch (error) {
    console.error('Error handling session.created:', error);
    throw error;
  }
}

/**
 * Handle user.deleted webhook event
 */
async function handleUserDeleted(userData: any) {
  try {
    const clerkUserId = userData.id;

    // Find user by clerkId
    const user = await db.user.findFirst({
      where: { clerkId: clerkUserId }
    });

    if (!user) {
      console.log('User not found for deletion:', clerkUserId);
      return;
    }

    // Delete the user and all related data (cascading will handle months, budgets, expenses)
    // Note: Categories created by the user will also be deleted
    await db.user.delete({
      where: { id: user.id }
    });

    console.log('Deleted user and all related data:', user.id);
  } catch (error) {
    console.error('Error handling user.deleted:', error);
    throw error;
  }
}

