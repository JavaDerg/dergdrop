use crate::err;
use crate::store::{AppendResult, DataStoreHandle};
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

pub fn mk_router() -> Router<DataStoreHandle> {
    Router::new()
        .route("/", post(init_upload))
        .route("/:id", patch(submit_chunk))
}

async fn init_upload(State(store): State<DataStoreHandle>, meta: Bytes) -> err::Result<Response> {
    if meta.len() > 4096 {
        return Ok((
            StatusCode::BAD_REQUEST,
            "Metadata may not be larger than 4096 bytes",
        )
            .into_response());
    }

    Ok(store.init_store(meta).await?.to_string().into_response())
}

async fn submit_chunk(
    State(store): State<DataStoreHandle>,
    Path(id): Path<Uuid>,
    segment: Bytes,
) -> err::Result<Result<(), Response>> {
    if segment.is_empty() {
        store.complete_store(id).await?;
        return Ok(Err(id.to_string().into_response()));
    }

    info!("{}", segment.len());

    match store.append_chunk(id, segment).await? {
        AppendResult::Continue => Ok(Ok(())),
        AppendResult::Done => unreachable!(),
        AppendResult::Gone => Ok(Err(
            (StatusCode::NOT_FOUND, "upload not present or gone").into_response()
        )),
    }
}
