mod err;
mod store;
mod upload;

use crate::store::DataStore;
use axum::extract::{BodyStream, State};
use axum::routing::{get, post};
use axum::{Router, Server, ServiceExt};
use futures_util::StreamExt;
use moka::future::Cache;
use std::io::Write;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let _ = dotenv::dotenv();
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let app = Router::new()
        .nest("/api/upload", upload::mk_router())
        .with_state(DataStore::new());

    Server::bind(&"0.0.0.0:8008".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/*
async fn upload_file(State(ds): State<DataStore>, mut stream: BodyStream) -> err::Result<String> {
    let (id, mut writer) = ds.store().await?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        writer.write_all(&chunk).await?;
    }

    Ok(id.to_string())
}
*/
