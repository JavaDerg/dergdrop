#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![feature(try_blocks)]

mod err;
mod state;

use crate::state::{StateHandle, UploadState, UploadStateLease};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post};
use axum::{Router, Server};
use bytes::Bytes;

use axum::extract::ws::Message;
use sqlx::{query, PgPool};
use std::path::PathBuf;
use tokio::fs::File;
use tracing::{error, info, warn};

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
        .route("/api/upload/ws", get(ws_upload))
        .route("/api/upload", post(init_upload))
        .route("/api/upload/:id", patch(submit_chunk))
        .route("/api/download/:id/meta", get(get_meta))
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

    let (id, ups) = bootstrap_upload(&db, &meta).await?;

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

    query!("UPDATE files SET completed = now() WHERE id = $1", id,)
        .execute(&db)
        .await?;

    Ok(())
}

async fn ws_upload(
    State((_sh, db)): State<(StateHandle, PgPool)>,
    ws: WebSocketUpgrade,
) -> Response {
    // FIXME: use try block or put in different function w/o state machine
    ws.on_upgrade(move |mut ws| async move {
        let Some(msg_res) = ws.recv().await else {
            return;
        };
        let meta = match msg_res {
            Ok(Message::Binary(meta)) => meta,
            _ => todo!(),
        };

        // FIXME: if UploadState drops the db won't be cleaned
        let (id, mut ups) = bootstrap_upload(&db, &meta).await.expect("TODO FIXME");

        while let Some(msg_res) = ws.recv().await {
            let msg = match msg_res {
                Ok(msg) => msg,
                Err(err) => {
                    error!(err = err.to_string());
                    return;
                }
            };

            match msg {
                Message::Binary(blob) => {
                    if blob.is_empty() {
                        if let Err(err) = ups.complete().await {
                            error!(err = err.to_string(), "failed to submit chunk");
                            return;
                        };

                        if let Err(err) = ws.send(Message::Text(id.to_string())).await {
                            error!(err = err.to_string(), "failed to send response");
                            return;
                        }

                        info!("{id}");

                        let Err(err) = ws.close().await else { return };
                        error!(err = err.to_string(), "failed to close websocket?");

                        return;
                    };

                    let Err(err) = ups.submit(&blob).await else {
                        continue;
                    };
                    error!(err = err.to_string(), "failed to submit chunk");

                    return;
                }
                Message::Close(cf) => {
                    let _ = ws.send(Message::Close(cf)).await;
                    return;
                }
                _ => {
                    warn!("received invalid message type");
                    return;
                }
            }
        }
    })
}

async fn get_meta(
    State((_sh, db)): State<(StateHandle, PgPool)>,
    Path(id): Path<Uuid>,
) -> err::Result<Vec<u8>> {
    let meta = query!("SELECT meta FROM files WHERE id = $1", &id)
        .fetch_one(&db)
        .await?
        .meta;

    Ok(meta)
}

async fn bootstrap_upload(db: &PgPool, meta: &[u8]) -> eyre::Result<(Uuid, UploadState)> {
    let id = Uuid::now_v7();

    query!("INSERT INTO files (id, meta) VALUES ($1, $2)", id, meta,)
        .execute(db)
        .await?;

    let path = PathBuf::from(format!("./data/{id}"));
    let ups = UploadState::new(File::create(&path).await?, path);

    Ok((id, ups))
}
