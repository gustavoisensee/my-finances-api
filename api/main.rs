use my_finances_api::{AppState, build_app, telemetry};
use tower::ServiceBuilder;
use vercel_runtime::Error;
use vercel_runtime::axum::VercelLayer;

#[tokio::main]
async fn main() -> Result<(), Error> {
    telemetry::init();
    let state = AppState::from_env()?;
    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(build_app(state));
    vercel_runtime::run(app).await
}
