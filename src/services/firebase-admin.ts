import * as admin from 'firebase-admin';

let app: admin.app.App | null = null;

export function getFirebaseAdminApp(): admin.app.App {
  if (app) return app;

  const json = process.env.FIREBASE_SERVICE_ACCOUNT_JSON;
  if (json) {
    const cred = JSON.parse(json) as admin.ServiceAccount & { project_id?: string };
    app = admin.initializeApp({
      credential: admin.credential.cert(cred),
      projectId: cred.projectId ?? cred.project_id ?? process.env.FIREBASE_PROJECT_ID,
    });
    return app;
  }

  const projectId = process.env.FIREBASE_PROJECT_ID;
  const clientEmail = process.env.FIREBASE_CLIENT_EMAIL;
  const privateKey = process.env.FIREBASE_PRIVATE_KEY?.replace(/\\n/g, '\n');

  if (projectId && clientEmail && privateKey) {
    app = admin.initializeApp({
      credential: admin.credential.cert({ projectId, clientEmail, privateKey }),
      projectId,
    });
    return app;
  }

  app = admin.initializeApp({
    credential: admin.credential.applicationDefault(),
    projectId,
  });
  return app;
}
