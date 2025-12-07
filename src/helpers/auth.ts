import { Request, Response, NextFunction } from 'express';
import { getAuth } from '@clerk/express';
import db from '../services/db';

/**
 * Helper function to get the internal user ID from the Clerk session
 * 
 * @param req - Express request object (must have been processed by Clerk middleware)
 * @returns Internal user ID from database, or 0 if not found/authenticated
 * 
 * Usage:
 * const userId = await getUserId(req);
 * if (!userId) {
 *   return res.status(401).send('Unauthorized');
 * }
 */
export const getUserId = async (req: Request): Promise<number> => {
  try {
    // Use Clerk's getAuth to get authenticated user's clerkId
    const { userId: clerkUserId } = getAuth(req);
    
    if (!clerkUserId) return 0;

    // Look up user by clerkId to get internal userId
    const user = await db.user.findFirst({
      where: { clerkId: clerkUserId },
      select: { id: true }
    });

    return user?.id || 0;
  } catch {
    return 0;
  }
}

/**
 * Middleware to require admin authorization using Clerk roles
 * 
 * This middleware checks if the authenticated user has the 'admin' role in Clerk.
 * Works with Clerk Organizations where roles are managed.
 * 
 * Must be used after customRequireAuth() middleware.
 * 
 * Usage:
 * app.get('/admin/users', customRequireAuth(), requireAdmin, getAllUsers);
 * 
 * Setup in Clerk:
 * 1. Enable Organizations in Clerk Dashboard
 * 2. Create an organization
 * 3. Assign users to the organization with the 'admin' role
 * 4. The role is automatically included in the session token
 * 
 * Supported roles: 'admin', 'org:admin'
 */
export const requireAdmin = async (req: Request, res: Response, next: NextFunction) => {
  try {
    const { userId, orgRole, sessionClaims } = getAuth(req);
    
    if (!userId) {
      return res.status(401).json({
        error: 'Unauthorized',
        message: 'Authentication required.',
        code: 'UNAUTHENTICATED'
      });
    }

    // Check if user has admin role from Clerk
    // orgRole is set if user is in an organization with a role
    // sessionClaims.metadata can also contain custom role information
    const metadata = sessionClaims?.metadata as { role?: string } | undefined;
    const isAdmin = 
      orgRole === 'admin' || 
      orgRole === 'org:admin' ||
      metadata?.role === 'admin';

    if (!isAdmin) {
      return res.status(403).json({
        error: 'Forbidden',
        message: 'Admin access required. This endpoint is restricted to administrators only.',
        code: 'ADMIN_ACCESS_REQUIRED'
      });
    }

    next();
  } catch (error) {
    console.error('Admin auth error:', error);
    return res.status(500).json({
      error: 'Internal Server Error',
      message: 'An error occurred while verifying admin access.',
      code: 'AUTHORIZATION_ERROR'
    });
  }
}