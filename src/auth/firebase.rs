use std::collections::HashMap;
use std::time::{Duration, Instant};

use chrono::Utc;
use jsonwebtoken::jwk::{AlgorithmParameters, JwkSet};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use tokio::sync::{Mutex, RwLock};

/// Public keys used to sign Firebase ID tokens, published as a JWK set.
pub const GOOGLE_JWKS_URL: &str =
    "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";

const LEEWAY_SECS: u64 = 5;
const MAX_UID_LEN: usize = 128;
const DEFAULT_KEYS_TTL: Duration = Duration::from_secs(60 * 60);
/// Minimum gap between forced refreshes triggered by an unknown `kid`, so
/// forged tokens cannot be used to hammer Google's endpoint.
const MIN_REFRESH_INTERVAL: Duration = Duration::from_secs(60);
const FETCH_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Deserialize)]
pub struct FirebaseClaims {
    pub sub: String,
    pub aud: String,
    pub iss: String,
    pub exp: i64,
    pub iat: i64,
    #[serde(default)]
    pub auth_time: Option<i64>,
    #[serde(default)]
    pub admin: Option<serde_json::Value>,
    #[serde(default)]
    pub role: Option<serde_json::Value>,
}

impl FirebaseClaims {
    /// Custom claims `admin: true` or `role: "admin"`.
    pub fn has_admin_claim(&self) -> bool {
        matches!(self.admin, Some(serde_json::Value::Bool(true)))
            || matches!(&self.role, Some(serde_json::Value::String(r)) if r == "admin")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    #[error("malformed token: {0}")]
    Malformed(#[source] jsonwebtoken::errors::Error),
    #[error("unsupported algorithm {0:?}")]
    Algorithm(Algorithm),
    #[error("token has no kid header")]
    MissingKid,
    #[error("no public key for kid")]
    UnknownKid,
    #[error("could not fetch signing keys: {0}")]
    KeyFetch(String),
    #[error("invalid token: {0}")]
    Invalid(#[source] jsonwebtoken::errors::Error),
    #[error("token issued in the future")]
    IssuedInFuture,
    #[error("auth_time missing or in the future")]
    AuthTime,
    #[error("invalid subject")]
    Subject,
}

pub struct FirebaseVerifier {
    project_id: String,
    issuer: String,
    keys: KeySource,
}

enum KeySource {
    Remote(RemoteKeys),
    Static(HashMap<String, DecodingKey>),
}

struct RemoteKeys {
    client: reqwest::Client,
    url: String,
    cache: RwLock<Option<CachedKeys>>,
    refresh: Mutex<()>,
}

struct CachedKeys {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Instant,
    expires_at: Instant,
}

impl CachedKeys {
    fn get_fresh(&self, kid: &str) -> Option<DecodingKey> {
        (Instant::now() < self.expires_at)
            .then(|| self.keys.get(kid).cloned())
            .flatten()
    }
}

impl FirebaseVerifier {
    /// Verifier backed by Google's published keys (production).
    pub fn google(project_id: &str) -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder().timeout(FETCH_TIMEOUT).build()?;
        Ok(Self::with_source(
            project_id,
            KeySource::Remote(RemoteKeys {
                client,
                url: GOOGLE_JWKS_URL.to_owned(),
                cache: RwLock::new(None),
                refresh: Mutex::new(()),
            }),
        ))
    }

    /// Verifier with a fixed set of keys, keyed by `kid` (tests).
    pub fn with_static_keys(project_id: &str, keys: HashMap<String, DecodingKey>) -> Self {
        Self::with_source(project_id, KeySource::Static(keys))
    }

    fn with_source(project_id: &str, keys: KeySource) -> Self {
        Self {
            project_id: project_id.to_owned(),
            issuer: format!("https://securetoken.google.com/{project_id}"),
            keys,
        }
    }

    pub async fn verify(&self, token: &str) -> Result<FirebaseClaims, VerifyError> {
        let header = decode_header(token).map_err(VerifyError::Malformed)?;
        if header.alg != Algorithm::RS256 {
            return Err(VerifyError::Algorithm(header.alg));
        }
        let kid = header.kid.ok_or(VerifyError::MissingKid)?;
        let key = self.key_for(&kid).await?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.project_id]);
        validation.set_issuer(&[&self.issuer]);
        validation.set_required_spec_claims(&["exp", "iat", "sub", "aud", "iss"]);
        validation.leeway = LEEWAY_SECS;
        validation.validate_exp = true;

        let claims = decode::<FirebaseClaims>(token, &key, &validation)
            .map_err(VerifyError::Invalid)?
            .claims;

        let latest_allowed = Utc::now().timestamp() + LEEWAY_SECS as i64;
        if claims.iat > latest_allowed {
            return Err(VerifyError::IssuedInFuture);
        }
        if claims.auth_time.is_none_or(|t| t > latest_allowed) {
            return Err(VerifyError::AuthTime);
        }
        if claims.sub.is_empty() || claims.sub.len() > MAX_UID_LEN {
            return Err(VerifyError::Subject);
        }
        Ok(claims)
    }

    async fn key_for(&self, kid: &str) -> Result<DecodingKey, VerifyError> {
        match &self.keys {
            KeySource::Static(keys) => keys.get(kid).cloned().ok_or(VerifyError::UnknownKid),
            KeySource::Remote(remote) => remote.key_for(kid).await,
        }
    }
}

impl RemoteKeys {
    async fn key_for(&self, kid: &str) -> Result<DecodingKey, VerifyError> {
        if let Some(key) = self
            .cache
            .read()
            .await
            .as_ref()
            .and_then(|c| c.get_fresh(kid))
        {
            return Ok(key);
        }

        let _guard = self.refresh.lock().await;
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if let Some(key) = cached.get_fresh(kid) {
                    return Ok(key);
                }
                let fresh = Instant::now() < cached.expires_at;
                if fresh && cached.fetched_at.elapsed() < MIN_REFRESH_INTERVAL {
                    return Err(VerifyError::UnknownKid);
                }
            }
        }

        let fetched = self.fetch().await?;
        let key = fetched.keys.get(kid).cloned();
        *self.cache.write().await = Some(fetched);
        key.ok_or(VerifyError::UnknownKid)
    }

    async fn fetch(&self) -> Result<CachedKeys, VerifyError> {
        let fetch_err = |e: reqwest::Error| VerifyError::KeyFetch(e.to_string());
        let response = self
            .client
            .get(&self.url)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(fetch_err)?;

        let ttl = response
            .headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok())
            .and_then(parse_max_age)
            .unwrap_or(DEFAULT_KEYS_TTL);

        let set: JwkSet = response.json().await.map_err(fetch_err)?;
        let keys = decoding_keys(&set);
        if keys.is_empty() {
            return Err(VerifyError::KeyFetch(
                "JWK set contained no RSA keys".into(),
            ));
        }

        let now = Instant::now();
        Ok(CachedKeys {
            keys,
            fetched_at: now,
            expires_at: now + ttl,
        })
    }
}

fn decoding_keys(set: &JwkSet) -> HashMap<String, DecodingKey> {
    set.keys
        .iter()
        .filter(|jwk| matches!(jwk.algorithm, AlgorithmParameters::RSA(_)))
        .filter_map(|jwk| {
            let kid = jwk.common.key_id.clone()?;
            DecodingKey::from_jwk(jwk).ok().map(|key| (kid, key))
        })
        .collect()
}

fn parse_max_age(cache_control: &str) -> Option<Duration> {
    cache_control.split(',').find_map(|directive| {
        let (name, value) = directive.trim().split_once('=')?;
        name.trim()
            .eq_ignore_ascii_case("max-age")
            .then(|| value.trim().parse::<u64>().ok())
            .flatten()
            .map(Duration::from_secs)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cache_control_max_age() {
        assert_eq!(
            parse_max_age("public, max-age=19302, must-revalidate, no-transform"),
            Some(Duration::from_secs(19302))
        );
        assert_eq!(parse_max_age("no-cache"), None);
        assert_eq!(parse_max_age("max-age=abc"), None);
    }

    fn claims(admin: Option<serde_json::Value>, role: Option<serde_json::Value>) -> FirebaseClaims {
        FirebaseClaims {
            sub: "uid".into(),
            aud: "p".into(),
            iss: "i".into(),
            exp: 0,
            iat: 0,
            auth_time: Some(0),
            admin,
            role,
        }
    }

    #[test]
    fn admin_claims() {
        use serde_json::json;
        assert!(claims(Some(json!(true)), None).has_admin_claim());
        assert!(claims(None, Some(json!("admin"))).has_admin_claim());
        assert!(!claims(Some(json!("true")), None).has_admin_claim());
        assert!(!claims(Some(json!(1)), Some(json!("user"))).has_admin_claim());
        assert!(!claims(None, None).has_admin_claim());
    }
}
