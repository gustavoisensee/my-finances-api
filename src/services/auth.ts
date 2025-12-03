import { NextFunction, Request, Response } from 'express';
import jwt from 'jsonwebtoken';
import bcrypt from 'bcrypt';
import { verifyToken, createClerkClient } from '@clerk/backend';
import { WebhookEvent } from '@clerk/backend';

import db, { DEFAULT_ADMIN_USER_ID } from './db';
import { Verified } from '../types';

const clerkClient = createClerkClient({ 
  secretKey: process.env.CLERK_SECRET_KEY 
});

export const authenticate = async (req: Request, res: Response) => {
  try {
    const { email, password } = req.body || {};
    if (!email || !password) return res.status(500).json('User or password not provided!');

    const user = await db.user.findUnique({
      where: {
        email
      }
    });

    if (!user) return res.status(500).json('Email is invalid!');

    const result = await bcrypt.compare(password, user.password);

    if (!result) return res.status(500).json('Password is invalid!');

    const token = jwt.sign(
      { userId: user?.id },
      process.env.JWT_TOKEN as string
    );

    return res.json({ token, userId: user.id, userName: user.firstName });
  } catch {
    return res.status(500).json('Something went wrong, try again!');
  }
};

export const verify = async (req: Request, res: Response): Promise<any> => {
  try {
    const { authorization } = req.headers || {};
    if (!authorization) return res.json('Token not provided!');

    // Extract token (remove 'Bearer ' prefix if present and trim whitespace)
    const token = authorization.startsWith('Bearer ') 
      ? authorization.substring(7).trim() 
      : authorization.trim();

    // Verify Clerk session token
    const payload = await verifyToken(token, {
      secretKey: process.env.CLERK_SECRET_KEY as string,
      clockSkewInMs: 10000 // Allow 10 seconds clock skew
    });
    
    if (!payload || !payload.sub) {
      return res.json('Token is invalid!');
    }

    const clerkUserId = payload.sub;

    // Look up user by clerkId to get internal userId
    const user = await db.user.findFirst({
      where: { clerkId: clerkUserId },
      select: { id: true }
    });

    if (!user) {
      return res.json('User not found!');
    }

    return res.json({ userId: user.id, clerkUserId });
  } catch (e) {
    console.error('Verify error:', e);
    return res.json('Token is invalid!');
  }
}

export const isAuthenticated = async (req: Request, res: Response, next: NextFunction, cbVerified: (v: Verified) => boolean) => {
  try {
    const { authorization } = req.headers || {};
    if (!authorization) {
      res.status(401);
      return res.send('Request does not have authentication token.')
    }

    // Extract token (remove 'Bearer ' prefix if present and trim whitespace)
    const token = authorization.startsWith('Bearer ') 
      ? authorization.substring(7).trim() 
      : authorization.trim();

    // Verify Clerk session token
    const payload = await verifyToken(token, {
      secretKey: process.env.CLERK_SECRET_KEY as string,
      clockSkewInMs: 10000 // Allow 10 seconds clock skew
    });
    
    if (!payload || !payload.sub) {
      return res.send('Token is invalid!');
    }

    const clerkUserId = payload.sub;

    // Look up user by clerkId to get internal userId
    const user = await db.user.findFirst({
      where: { clerkId: clerkUserId },
      select: { id: true }
    });

    if (!user) {
      return res.send('User not found!');
    }

    const verified: Verified = { userId: user.id };
    
    if (cbVerified(verified)) {
      return next();
    }

    res.send('Something went wrong on the authentication.');
  } catch (e) {
    console.error('Auth error:', e);
    res.send('Token invalid.');
  }
};

const isAuth = (verified: Verified): boolean => !!verified;
export const mustBeAuthenticated = (req: Request, res: Response, next: NextFunction) => {
  isAuthenticated(req, res, next, isAuth);
};

const isAuthAdmin = (verified: Verified) => verified?.userId === DEFAULT_ADMIN_USER_ID;
export const mustBeAuthenticatedAdmin = (req: Request, res: Response, next: NextFunction) => {
  isAuthenticated(req, res, next, isAuthAdmin);
}

/**
 * Syncs a Clerk user with the local database
 * Creates a new user if doesn't exist, or updates existing user with clerkId
 */
export const syncClerkUser = async (req: Request, res: Response) => {
  try {
    const { authorization } = req.headers || {};
    if (!authorization) {
      return res.status(401).json({ message: 'Token not provided!' });
    }

    // Extract token
    const token = authorization.startsWith('Bearer ') 
      ? authorization.substring(7).trim() 
      : authorization.trim();

    // Verify Clerk token
    const payload = await verifyToken(token, {
      secretKey: process.env.CLERK_SECRET_KEY as string
    });
    
    if (!payload || !payload.sub) {
      return res.status(401).json({ message: 'Invalid token!' });
    }

    const clerkUserId = payload.sub;

    // Get full user data from Clerk
    const clerkUser = await clerkClient.users.getUser(clerkUserId);
    
    if (!clerkUser) {
      return res.status(404).json({ message: 'Clerk user not found!' });
    }

    // Get primary email from Clerk user
    const primaryEmail = clerkUser.emailAddresses.find(
      email => email.id === clerkUser.primaryEmailAddressId
    );

    if (!primaryEmail) {
      return res.status(400).json({ message: 'No email found for user!' });
    }

    // Check if user exists by email
    let user = await db.user.findUnique({
      where: { email: primaryEmail.emailAddress },
      select: { 
        id: true, 
        clerkId: true,
        email: true,
        firstName: true,
        lastName: true
      }
    });

    if (user) {
      // User exists - update with clerkId if not set
      if (!user.clerkId) {
        user = await db.user.update({
          where: { id: user.id },
          data: { clerkId: clerkUserId },
          select: { 
            id: true, 
            clerkId: true,
            email: true,
            firstName: true,
            lastName: true
          }
        });
      }
      
      return res.json({ 
        message: 'User synced successfully!',
        user: {
          id: user.id,
          clerkId: user.clerkId,
          email: user.email,
          firstName: user.firstName,
          lastName: user.lastName
        }
      });
    }

    // User doesn't exist - create new user
    // Note: You'll need to provide a default genderId or make it optional in your schema
    // 4 - Prefer not to say
    const defaultGenderId = 4;
    const newUser = await db.user.create({
      data: {
        clerkId: clerkUserId,
        email: primaryEmail.emailAddress,
        firstName: clerkUser.firstName || 'User',
        lastName: clerkUser.lastName || '',
        dateOfBirth: new Date(), // Default date, can be updated by user later
        password: '', // No password needed for Clerk users
        createdAt: new Date(),
        genderId: defaultGenderId
      },
      select: { 
        id: true, 
        clerkId: true,
        email: true,
        firstName: true,
        lastName: true
      }
    });

    return res.json({ 
      message: 'User created and synced successfully!',
      user: {
        id: newUser.id,
        clerkId: newUser.clerkId,
        email: newUser.email,
        firstName: newUser.firstName,
        lastName: newUser.lastName
      }
    });

  } catch (error) {
    console.error('Sync error:', error);
    return res.status(500).json({ 
      message: 'Error syncing user', 
      error: error instanceof Error ? error.message : 'Unknown error'
    });
  }
};

/**
 * Handles Clerk webhooks for automatic user sync
 * Events: user.created, user.updated, user.deleted, session.created
 */
export const handleClerkWebhook = async (req: Request, res: Response) => {
  try {
    console.log('handleClerkWebhook 1');
    
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

    console.log(`Webhook received: ${eventType}`, userData.id || userData.user_id || 'no-id');

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

    console.log('handleClerkWebhook 2 - Success!');
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
    const primaryEmail = userData.email_addresses?.find(
      (email: any) => email.id === userData.primary_email_address_id
    );

    if (!primaryEmail) {
      console.error('No primary email found for user:', clerkUserId);
      return;
    }

    // Check if user already exists
    const existingUser = await db.user.findFirst({
      where: { 
        OR: [
          { clerkId: clerkUserId },
          { email: primaryEmail.email_address }
        ]
      }
    });

    if (existingUser) {
      // Update existing user with clerkId if not set
      if (!existingUser.clerkId) {
        await db.user.update({
          where: { id: existingUser.id },
          data: { 
            clerkId: clerkUserId,
            firstName: userData.first_name || existingUser.firstName,
            lastName: userData.last_name || existingUser.lastName
          }
        });
        console.log('Updated existing user with clerkId:', existingUser.id);
      }
      return;
    }

    // Create new user
    const defaultGenderId = 4; // Prefer not to say
    const newUser = await db.user.create({
      data: {
        clerkId: clerkUserId,
        email: primaryEmail.email_address,
        firstName: userData.first_name || 'User',
        lastName: userData.last_name || '',
        dateOfBirth: new Date(),
        password: '', // No password needed for Clerk users
        createdAt: new Date(),
        genderId: defaultGenderId
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
 */
async function handleUserUpdated(userData: any) {
  try {
    const clerkUserId = userData.id;
    const primaryEmail = userData.email_addresses?.find(
      (email: any) => email.id === userData.primary_email_address_id
    );

    if (!primaryEmail) {
      console.error('No primary email found for user:', clerkUserId);
      return;
    }

    // Find user by clerkId
    const user = await db.user.findFirst({
      where: { clerkId: clerkUserId }
    });

    if (!user) {
      console.log('User not found, creating new user from update event');
      await handleUserCreated(userData);
      return;
    }

    // Update user data
    await db.user.update({
      where: { id: user.id },
      data: {
        email: primaryEmail.email_address,
        firstName: userData.first_name || user.firstName,
        lastName: userData.last_name || user.lastName
      }
    });

    console.log('Updated user from webhook:', user.id);
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

    // Check if user exists and has clerkId
    const user = await db.user.findFirst({
      where: { clerkId: clerkUserId }
    });

    if (user) {
      // User already synced, nothing to do
      console.log('User already synced on sign-in:', user.id);
      return;
    }

    // User not synced yet - fetch from Clerk and create/update
    console.log('User not synced, fetching from Clerk:', clerkUserId);
    const clerkUser = await clerkClient.users.getUser(clerkUserId);
    
    if (!clerkUser) {
      console.error('Failed to fetch user from Clerk:', clerkUserId);
      return;
    }

    // Find primary email from Clerk User object (camelCase format!)
    const primaryEmail = clerkUser.emailAddresses?.find(
      (email: any) => email.id === clerkUser.primaryEmailAddressId
    );

    if (!primaryEmail) {
      console.error('No primary email found for user:', clerkUserId);
      return;
    }

    console.log('Primary email:', primaryEmail.emailAddress);

    // Check if user already exists by email
    const existingUser = await db.user.findFirst({
      where: { email: primaryEmail.emailAddress }
    });

    if (existingUser) {
      // Update existing user with clerkId if not set
      if (!existingUser.clerkId) {
        await db.user.update({
          where: { id: existingUser.id },
          data: { 
            clerkId: clerkUserId,
            firstName: clerkUser.firstName || existingUser.firstName,
            lastName: clerkUser.lastName || existingUser.lastName
          }
        });
        console.log('Updated existing user with clerkId:', existingUser.id);
      }
      return;
    }

    // Create new user
    console.log('Creating new user...');
    const defaultGenderId = 4; // Prefer not to say
    const newUser = await db.user.create({
      data: {
        clerkId: clerkUserId,
        email: primaryEmail.emailAddress,
        firstName: clerkUser.firstName || 'User',
        lastName: clerkUser.lastName || '',
        dateOfBirth: new Date(),
        password: '', // No password needed for Clerk users
        createdAt: new Date(),
        genderId: defaultGenderId
      }
    });

    console.log('Created new user from session webhook:', newUser.id);
    console.log('handleSessionCreated 2');
  } catch (error) {
    console.error('Error handling session.created:', error);
    throw error;
  }
}
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

    // Don't delete admin user
    if (user.id === DEFAULT_ADMIN_USER_ID) {
      console.log('Cannot delete admin user via webhook');
      return;
    }

    // Optional: Instead of deleting, you might want to soft-delete or just remove clerkId
    // For now, we'll just remove the clerkId to preserve user data
    await db.user.update({
      where: { id: user.id },
      data: { clerkId: null }
    });

    console.log('Removed clerkId from user (soft delete):', user.id);
  } catch (error) {
    console.error('Error handling user.deleted:', error);
    throw error;
  }
}