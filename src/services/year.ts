import db from './db';

export const getAllYears = async (take: number) => {
  return db.year.findMany({ take });
};
