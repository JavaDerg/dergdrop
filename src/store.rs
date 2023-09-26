use crate::upload::UploadChunk;
use axum::body::Bytes;
use moka::future::Cache;
use smallvec::SmallVec;
use sqlx::Either;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::{Mutex, Notify, RwLock};
use uuid::Uuid;

#[derive(Clone)]
pub struct DataStore {
    cache: Cache<Uuid, Arc<Mutex<WriterState>>>,
}

impl DataStore {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .time_to_idle(Duration::from_secs(60))
                .build(),
        }
    }

    pub async fn init_store(&self) -> eyre::Result<Uuid> {
        let id = Uuid::new_v4();

        // let file = tokio::fs::File::create(format!("./data/{id}")).await?;
        let file = tokio::fs::File::create("/dev/null").await?;

        let writer = Arc::new(Mutex::new(WriterState {
            writer: Box::new(file),
        }));

        self.cache.insert(id, writer).await;

        Ok(id)
    }

    pub async fn append_segment(&self, id: Uuid, segment: Bytes) -> eyre::Result<Option<()>> {
        let Some(writer) = self.cache.get(&id).await else {
            return Ok(None);
        };

        let mut writer = writer.lock().await;
        writer.writer.write_all(&segment).await?;

        Ok(Some(()))
    }

    pub async fn complete_store(&self, id: Uuid) {
        self.cache.invalidate(&id).await;
    }
}

struct WriterState {
    // reorder: SmallVec<[UploadChunk; 16]>,
    // size: usize,
    // end: usize,
    writer: Box<dyn AsyncWrite + Send + Sync + Unpin>,
}

impl WriterState {
    pub async fn push(&mut self, chunk: UploadChunk) -> eyre::Result<()> {
        self.write_segment(chunk.segment).await?;

        /*
        if chunk.params.start == self.end {
        } else {
            todo!("not atm")
        }

        if let Some(idx) = self
            .reorder
            .iter()
            .enumerate()
            .find(|(_, c)| c.params.start == self.end)
            .map(|(i, _)| i)
        {
        }
        */

        Ok(())
    }

    async fn write_segment(&mut self, seg: Bytes) -> eyre::Result<()> {
        // self.end += seg.len();

        Ok(self.writer.write_all(&seg).await?)
    }
}
