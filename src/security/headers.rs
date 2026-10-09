use axum::Router;
use axum::http::{HeaderName, HeaderValue, header};
use tower_http::set_header::SetResponseHeaderLayer;

/// Helmet 8 default response headers.
pub const SECURITY_HEADERS: &[(&str, &str)] = &[
    (
        "content-security-policy",
        "default-src 'self';base-uri 'self';font-src 'self' https: data:;form-action 'self';\
         frame-ancestors 'self';img-src 'self' data:;object-src 'none';script-src 'self';\
         script-src-attr 'none';style-src 'self' https: 'unsafe-inline';upgrade-insecure-requests",
    ),
    ("cross-origin-opener-policy", "same-origin"),
    ("cross-origin-resource-policy", "same-origin"),
    ("origin-agent-cluster", "?1"),
    ("referrer-policy", "no-referrer"),
    (
        "strict-transport-security",
        "max-age=31536000; includeSubDomains",
    ),
    ("x-content-type-options", "nosniff"),
    ("x-dns-prefetch-control", "off"),
    ("x-download-options", "noopen"),
    ("x-frame-options", "SAMEORIGIN"),
    ("x-permitted-cross-domain-policies", "none"),
    ("x-xss-protection", "0"),
];

pub fn apply<S: Clone + Send + Sync + 'static>(mut router: Router<S>) -> Router<S> {
    for (name, value) in SECURITY_HEADERS {
        router = router.layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static(name),
            HeaderValue::from_static(value),
        ));
    }
    router.layer(SetResponseHeaderLayer::overriding(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    ))
}
