pub mod cors;
pub mod headers;

/// Same limit as `express.json({ limit: '1mb' })`.
pub const BODY_LIMIT_BYTES: usize = 1024 * 1024;
