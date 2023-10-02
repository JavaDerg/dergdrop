#![deny(clippy::all, clippy::pedantic)]
#![feature(try_blocks)]

mod err;
mod store;
mod upload;

use crate::store::DataStoreHandle;
use axum::{Router, Server, ServiceExt};
use futures_util::StreamExt;
use sqlx::PgPool;
use std::io::Write;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    if Path::new(".env").exists() {
        let _ = dotenvy::dotenv()?;
    }
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let db = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;

    let app = Router::new()
        .nest("/api/upload", upload::mk_router())
        .with_state(DataStoreHandle::new(db));

    Server::bind(&"0.0.0.0:8008".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
