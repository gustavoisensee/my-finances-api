import { Request } from 'express';
import { verifyToken } from '@clerk/backend';
import db from '../services/db';

export const getUserId = async (req: Request): Promise<number> => {
  try {
    const { authorization } = req.headers || {};
    
    if (!authorization) return 0;

    // Extract token (remove 'Bearer ' prefix if present and trim whitespace)
    const token = authorization.startsWith('Bearer ') 
      ? authorization.substring(7).trim() 
      : authorization.trim();

    // Verify Clerk session token
    const payload = await verifyToken(token, {
      secretKey: process.env.CLERK_SECRET_KEY as string
    });
    
    if (!payload || !payload.sub) return 0;

    const clerkUserId = payload.sub;

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