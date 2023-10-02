use crate::err;
use axum::body::Bytes;
use flume::{Receiver, Sender};
use futures_util::StreamExt;
use sqlx::PgPool;
use std::collections::HashMap;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::select;
use uuid::Uuid;

mod worker;

#[derive(Clone)]
pub struct DataStoreHandle {
    sender: Sender<DSReq>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum AppendResult {
    Continue,
    Done,
    Gone,
}

enum DSReq {
    Init(InitReq),
    Chunk(ChunkReq),
}

struct InitReq {
    meta: Bytes,
    responder: Sender<eyre::Result<Uuid>>,
}

struct ChunkReq {
    id: Uuid,
    data: Bytes,
    responder: Sender<eyre::Result<AppendResult>>,
}

struct DataStore {
    receiver: Receiver<DSReq>,
    db: PgPool,

    active: HashMap<Uuid, Sender<ChunkReq>>,

    register: Receiver<(Uuid, Sender<ChunkReq>)>,
    register_tx: Sender<(Uuid, Sender<ChunkReq>)>,
}

impl DataStore {
    pub fn start(db: PgPool) -> Sender<DSReq> {
        let (tx, rx) = flume::bounded(64);

        tokio::spawn(async move {
            let (rtx, rrx) = flume::bounded(64);
            let mut store = Self {
                receiver: rx,
                db,
                active: Default::default(),
                register: rrx,
                register_tx: rtx,
            };

            loop {
                select! {
                    biased;
                    reg = store.register.recv_async() => {
                        let Ok((id, sender)) = reg else { unreachable!() };
                        store.register(id, sender);
                    },
                    msg = store.receiver.recv_async() => {
                        let Ok(msg) = msg else { return };
                        store.handle_msg(msg).await;
                    },
                }
            }
        });

        tx
    }

    fn register(&mut self, id: Uuid, sender: Sender<ChunkReq>) {
        assert!(self.active.insert(id, sender).is_none());
    }

    async fn handle_msg(&mut self, msg: DSReq) {
        match msg {
            DSReq::Init(req) => {
                let (tx, rx) = flume::bounded(1);
                tokio::spawn(worker::start(
                    self.db.clone(),
                    req,
                    rx,
                    tx,
                    self.register_tx.clone(),
                ));
            }
            DSReq::Chunk(req) => {
                let id = req.id;
                let gone = match self.active.get(&id) {
                    Some(handle) => handle.send_async(req).await.map(|_| true).unwrap_or(false),
                    None => false,
                };

                if gone {
                    self.active.remove(&id);
                }
            }
        }
    }
}

impl DataStoreHandle {
    pub fn new(db: PgPool) -> Self {
        Self {
            sender: DataStore::start(db),
        }
    }

    pub async fn init_store(&self, meta: Bytes) -> eyre::Result<Uuid> {
        let (tx, rx) = flume::bounded(0);

        self.sender
            .send_async(DSReq::Init(InitReq {
                meta,
                responder: tx,
            }))
            .await?;

        Ok(rx.recv_async().await??)
    }

    pub async fn append_chunk(&self, id: Uuid, chunk: Bytes) -> eyre::Result<AppendResult> {
        let (tx, rx) = flume::bounded(0);

        self.sender
            .send_async(DSReq::Chunk(ChunkReq {
                id,
                data: chunk,
                responder: tx,
            }))
            .await?;

        Ok(rx.recv_async().await??)
    }

    pub async fn complete_store(&self, id: Uuid) -> eyre::Result<()> {
        self.append_chunk(id, Bytes::new()).await.map(drop)
    }
}
