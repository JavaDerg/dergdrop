use axum::{Router, Server};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let _ = dotenv::dotenv();
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let app = Router::new();

    Server::bind(&"0.0.0.0:8008".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
