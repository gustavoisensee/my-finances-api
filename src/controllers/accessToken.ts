import { getQueryTake } from '../helpers/query';
import { asyncHandler } from '../middleware/errorHandler';
import * as accessTokenService from '../services/accessTokens';

export const getAllAccessTokens = asyncHandler(async (req, res) => {
  const tokens = await accessTokenService.getAllAccessTokens(getQueryTake(req));
  res.json(tokens);
});
