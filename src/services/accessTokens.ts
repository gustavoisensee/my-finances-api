import db from './db';

export const getAllAccessTokens = async (take: number) => {
  return db.accessToken.findMany({ take });
};
