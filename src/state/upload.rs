use crate::state::{StateHandle, StateReq, UploadStateLease};
use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;
use axum::http::StatusCode;
use sqlx::PgPool;
use std::path::PathBuf;
use axum::response::Response;
use flume::bounded;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::error;
use uuid::Uuid;

pub struct UploadState {
    file: Option<(File, PathBuf)>,
}

impl UploadState {
    pub fn new(file: File, path: PathBuf) -> Self {
        Self {
            file: Some((file, path)),
        }
    }

    pub async fn submit(&mut self, data: &[u8]) -> eyre::Result<()> {
        Ok(self.file.as_mut().unwrap().0.write_all(data).await?)
    }

    pub async fn complete(&mut self) -> eyre::Result<()> {
        Ok(self.file.take().unwrap().0.flush().await?)
    }
}

impl Drop for UploadState {
    fn drop(&mut self) {
        let Some((mut file, path)) = self.file.take() else {
            return;
        };

        tokio::spawn(async move {
            let res: eyre::Result<()> = try {
                file.shutdown().await?;
                drop(file);
                tokio::fs::remove_file(path).await?;
            };

            if let Err(err) = res {
                error!("{}", err);
            }
        });
    }
}
