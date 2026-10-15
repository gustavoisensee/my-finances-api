//! Local development server: the same router as the Vercel function, served
//! directly with Tokio on `PORT` (default 3001).

use std::env;
use std::net::SocketAddr;

use my_finances_api::state::BoxError;
use my_finances_api::{AppState, build_app, telemetry};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    dotenvy::dotenv().ok();
    telemetry::init();

    let port: u16 = env::var("PORT")
        .or_else(|_| env::var("NODE_PORT"))
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    let state = AppState::from_env()?;
    let pool = state.db.clone();
    let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await?;
    tracing::info!("Server listening on port {port}");

    axum::serve(listener, build_app(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Shutting down...");
    pool.close().await;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            sig.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
