use axum::Router;
use tower_http::trace::TraceLayer;
use tracing::Level;

mod bluetooth;
mod error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // TODO: AUTH

    let app = Router::new()
        .nest("/bluetooth", bluetooth::router().await?)
        .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
