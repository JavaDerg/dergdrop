use crate::err;
use crate::store::{AppendResult, ChunkReq, InitReq};
use bytes::Bytes;
use eyre::eyre;
use eyre::WrapErr;
use flume::{Receiver, Sender};
use futures::future::FutureExt;
use sqlx::{query, query_as, Acquire, PgPool, Postgres, Transaction};
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time::timeout;
use tracing::trace;
use uuid::Uuid;

pub(crate) async fn start(
    db: PgPool,
    ir: InitReq,
    stream: Receiver<ChunkReq>,
    stream_in: Sender<ChunkReq>,
    stream_submit: Sender<(Uuid, Sender<ChunkReq>)>,
) -> eyre::Result<()> {
    let res: eyre::Result<(Uuid, File, PathBuf)> = try {
        // we use the transaction to rollback the id creating in the db, if creating the file fails (its gc basically lol)

        let (ta, id) = init(&db, ir.meta)
            .await
            .wrap_err("Failed to create file id")?;
        let path = PathBuf::from(format!("./data/{id}"));
        let fd = File::create(&path)
            .await
            .wrap_err("Failed to create file for made id")?;
        ta.commit().await?;

        (id, fd, path)
    };

    let (id, mut fd, path) = match res {
        Ok(vals) => vals,
        Err(err) => {
            trace!("{err}");
            let _ = ir.responder.send_async(Err(err.into())).await;

            return Err(eyre!("ignore this")).into();
        }
    };
    stream_submit.send_async((id, stream_in)).await?;
    let _ = ir.responder.send_async(Ok(id)).await;

    while let Ok(Ok(ChunkReq {
        data, responder, ..
    })) = timeout(Duration::from_secs(60), stream.recv_async()).await
    {
        let res: eyre::Result<()> = try {
            if data.is_empty() {
                fd.flush()
                    .await
                    .wrap_err("failed to flush final parts of file")?;
                responder.send_async(Ok(AppendResult::Done)).await?;
                return Ok(());
            }

            fd.write_all(&data)
                .await
                .wrap_err("failed to write to disk")?;
            responder.send_async(Ok(AppendResult::Continue)).await?;
        };

        if res.is_err() {
            responder.send_async(Err(res.unwrap_err())).await?;

            drop(fd);

            // we only try to cleanup the fs as it's the source of the fault
            let _ = tokio::fs::remove_file(&path).await;
            clean_up(&db, id).await.wrap_err("io error -> cleanup")?;

            return Err(eyre!("io error")).into();
        }
    }

    // reaching the end of the stream without a zero block means we didn't get the entire file -> clean up
    // or, a time out state has occurred
    let _ = tokio::fs::remove_file(&path).await;
    clean_up(&db, id).await.wrap_err("incomplete")?;

    Err(eyre!("incomplete file")).into()
}

async fn clean_up(db: &PgPool, id: Uuid) -> eyre::Result<()> {
    query!("DELETE FROM files WHERE id = $1", id)
        .execute(db)
        .await?;
    Ok(())
}

async fn init(db: &PgPool, meta: Bytes) -> eyre::Result<(Transaction<Postgres>, Uuid)> {
    struct DbResponse {
        id: Uuid,
    }

    let mut ta = db.begin().await?;

    let id = query_as!(
        DbResponse,
        "INSERT INTO files (meta) VALUES ($1) RETURNING id;",
        &meta[..],
    )
    .fetch_one(ta.as_mut())
    .await
    .map(|r| r.id)?;

    Ok((ta, id))
}
