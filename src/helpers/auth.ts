import { Request, Response, NextFunction } from 'express';
import type { DecodedIdToken } from 'firebase-admin/auth';

import db from '../services/db';
import { getFirebaseAdminApp } from '../services/firebase-admin';
import { UnauthorizedError } from '../utils/errors';

export const getUserId = async (req: Request): Promise<number> => {
  const firebaseUid = req.firebaseUid;
  if (!firebaseUid) {
    throw new UnauthorizedError('Authentication required.');
  }

  const user = await db.user.upsert({
    where: { firebaseUid },
    update: {},
    create: {
      firebaseUid,
      createdAt: new Date(),
    },
    select: { id: true },
  });
  return user.id;
};

function parseAdminUidAllowlist(): Set<string> {
  const raw = process.env.ADMIN_FIREBASE_UIDS;
  if (!raw?.trim()) return new Set();
  return new Set(
    raw
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean)
  );
}

export const requireAdmin = async (req: Request, res: Response, next: NextFunction) => {
  try {
    const uid = req.firebaseUid;
    const token = req.firebaseDecodedToken;

    if (!uid || !token) {
      return res.status(401).json({
        error: 'Unauthorized',
        message: 'Authentication required.',
        code: 'UNAUTHENTICATED',
      });
    }

    const allowlist = parseAdminUidAllowlist();
    const claims = token as DecodedIdToken & { role?: string };
    const isAdmin =
      claims.admin === true ||
      claims.role === 'admin' ||
      allowlist.has(uid);

    if (!isAdmin) {
      return res.status(403).json({
        error: 'Forbidden',
        message: 'Admin access required.',
        code: 'ADMIN_ACCESS_REQUIRED',
      });
    }

    next();
  } catch (error) {
    console.error('Admin auth error:', error);
    return res.status(500).json({
      error: 'Internal Server Error',
      message: 'An error occurred while verifying admin access.',
      code: 'AUTHORIZATION_ERROR',
    });
  }
};

export const requireFirebaseAuth = () => {
  return async (req: Request, res: Response, next: NextFunction) => {
    const header = req.headers.authorization;
    if (!header?.startsWith('Bearer ')) {
      return res.status(401).json({
        error: 'Unauthorized',
        message: 'Authentication required. Please provide a valid Firebase ID token.',
        code: 'UNAUTHENTICATED',
      });
    }

    const idToken = header.slice(7).trim();
    if (!idToken) {
      return res.status(401).json({
        error: 'Unauthorized',
        message: 'Authentication required. Please provide a valid Firebase ID token.',
        code: 'UNAUTHENTICATED',
      });
    }

    try {
      const adminApp = getFirebaseAdminApp();
      const decoded = await adminApp.auth().verifyIdToken(idToken);
      req.firebaseUid = decoded.uid;
      req.firebaseDecodedToken = decoded;
      next();
    } catch {
      return res.status(401).json({
        error: 'Unauthorized',
        message: 'Authentication required. Please provide a valid Firebase ID token.',
        code: 'UNAUTHENTICATED',
      });
    }
  };
};
