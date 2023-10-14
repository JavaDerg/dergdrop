use sqlx::PgPool;
use std::path::PathBuf;

use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::error;

pub struct UploadState {
    file: Option<(File, PathBuf)>,
    db: PgPool,
}

impl UploadState {
    pub fn new(file: File, path: PathBuf, db: PgPool) -> Self {
        Self {
            file: Some((file, path)),
            db,
        }
    }

    pub async fn submit(&mut self, data: &[u8]) -> eyre::Result<()> {
        Ok(self.file.as_mut().unwrap().0.write_all(data).await?)
    }

    pub async fn complete(&mut self) -> eyre::Result<()> {
        Ok(self.file.take().unwrap().0.flush().await?)
    }

    pub async fn rollback(&mut self) -> eyre::Result<()> {
        let Some((mut file, path)) = self.file.take() else {
            return Ok(());
        };

        file.shutdown().await?;
        drop(file);
        tokio::fs::remove_file(path).await?;

        Ok(())
    }
}

impl Drop for UploadState {
    fn drop(&mut self) {
        let mut copy = Self {
            file: self.file.take(),
            db: self.db.clone(),
        };

        tokio::spawn(async move {
            if let Err(err) = copy.rollback().await {
                error!("{}", err);
            }
        });
    }
}
