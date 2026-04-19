import { getQueryTake } from '../helpers/query';
import { asyncHandler } from '../middleware/errorHandler';
import * as yearService from '../services/year';

export const getAllYears = asyncHandler(async (req, res) => {
  const years = await yearService.getAllYears(getQueryTake(req));
  res.json(years);
});
