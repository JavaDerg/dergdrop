#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![feature(try_blocks)]

mod err;
mod state;

use crate::state::{StateHandle, UploadState, UploadStateLease};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{patch, post};
use axum::{Router, Server};
use bytes::Bytes;

use sqlx::{query, PgPool};
use std::path::PathBuf;
use tokio::fs::File;
use tracing::info;

use tracing_subscriber::{fmt, EnvFilter};
use uuid::Uuid;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    if std::path::Path::new(".env").exists() {
        let _ = dotenvy::dotenv()?;
    }
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let db = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
    let handle = StateHandle::start_new();

    let app = Router::new()
        .route("/api/upload", post(init_upload))
        .route("/api/upload/:id", patch(submit_chunk))
        .with_state((handle, db));

    Server::bind(&"0.0.0.0:8008".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
pub struct UploadChunk {
    pub params: ChunkParams,
    pub segment: Bytes,
}

#[derive(serde::Deserialize)]
pub struct ChunkParams {
    pub start: usize,
}

async fn init_upload(
    State((sh, db)): State<(StateHandle, PgPool)>,
    meta: Bytes,
) -> err::Result<Response> {
    if meta.len() > 4096 {
        return Ok((
            StatusCode::BAD_REQUEST,
            "Metadata may not be larger than 4096 bytes",
        )
            .into_response());
    }

    info!("starting upload");

    let id = Uuid::now_v7();
    query!(
        "INSERT INTO files (id, meta) VALUES ($1, $2)",
        id,
        &meta[..]
    )
    .execute(&db)
    .await?;

    let path = PathBuf::from(format!("./data/{id}"));
    let ups = UploadState::new(File::create(&path).await?, path);

    sh.insert(id, ups).await;

    info!("uploading under {}", id);

    Ok(id.to_string().into_response())
}

#[tracing::instrument(skip_all, fields(id))]
async fn submit_chunk(
    State((_, db)): State<(StateHandle, PgPool)>,
    Path(id): Path<Uuid>,
    mut usl: UploadStateLease,
    segment: Bytes,
) -> err::Result<()> {
    if !segment.is_empty() {
        usl.submit(&segment).await?;
        return Ok(());
    }

    info!("segment submitted");

    let id = usl.id();
    usl.complete().await?;

    query!(
        "UPDATE files SET completed = now() WHERE id = $1",
        id,
    )
    .execute(&db)
    .await?;

    Ok(())
}
