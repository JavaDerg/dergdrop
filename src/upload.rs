use crate::err;
use crate::store::DataStore;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{patch, post};
use axum::Router;
use tracing::info;
use uuid::Uuid;

pub struct UploadChunk {
    pub params: ChunkParams,
    pub segment: Bytes,
}

#[derive(serde::Deserialize)]
pub struct ChunkParams {
    pub start: usize,
}

pub fn mk_router() -> Router<DataStore> {
    Router::new()
        .route("/", post(init_upload))
        .route("/:id", patch(upload_segment))
}

async fn init_upload(State(store): State<DataStore>) -> err::Result<String> {
    info!("start");
    Ok(store.init_store().await?.to_string())
}

async fn upload_segment(
    State(store): State<DataStore>,
    Path(id): Path<Uuid>,
    segment: Bytes,
) -> err::Result<Result<(), Response>> {
    if segment.is_empty() {
        store.complete_store(id).await;
        info!("done");
        return Ok(Err(id.to_string().into_response()));
    }

    info!("{}", segment.len());
    if store.append_segment(id, segment).await?.is_none() {
        return Ok(Err(
            (StatusCode::NOT_FOUND, "upload not present or gone").into_response()
        ));
    }

    Ok(Ok(()))
}
