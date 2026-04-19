import type { DecodedIdToken } from 'firebase-admin/auth';

declare global {
  namespace Express {
    interface Request {
      firebaseUid?: string;
      firebaseDecodedToken?: DecodedIdToken;
    }
  }
}

export {};
