mod upload;

use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use flume::{bounded, Receiver, Sender};
use sqlx::PgPool;
use std::collections::{BTreeMap, HashMap};
use std::mem::swap;
use std::time::Duration;
use tokio::time::{sleep_until, Instant};
use uuid::Uuid;

pub use upload::UploadState;

struct State {
    states: HashMap<Uuid, (UploadState, (Instant, u64))>,
    timeouts: BTreeMap<(Instant, u64), Uuid>,
}

enum StateReq {
    Insert(Uuid, UploadState),
    Get(Uuid, Sender<Option<UploadState>>),
}

pub struct UploadStateLease {
    id: Uuid,
    handle: StateHandle,

    state: Option<UploadState>,
}

#[derive(Clone)]
pub struct StateHandle {
    sender: Sender<StateReq>,
}

impl StateHandle {
    pub fn start_new() -> Self {
        let mut state = State {
            states: HashMap::default(),
            timeouts: BTreeMap::default(),
        };

        let (tx, rx) = bounded(64);
        tokio::spawn(async move { state.run(rx).await });

        Self { sender: tx }
    }

    pub async fn insert(&self, id: Uuid, ups: UploadState) {
        let _ = self.sender.send_async(StateReq::Insert(id, ups)).await;
    }
}

impl State {
    async fn run(&mut self, rx: Receiver<StateReq>) {
        loop {
            tokio::select! {
                biased;
                req = rx.recv_async() => match req {
                    Ok(req) => self.handle_req(req).await,
                    Err(_) => return,
                },
                () = self.timeout_routine() => (),
            }
        }
    }

    async fn timeout_routine(&mut self) {
        loop {
            let now = Instant::now();

            let mut expired = self.timeouts.split_off(&(now, u64::MAX));
            // we swap as split_off only gives us the stuff that's still valid
            swap(&mut self.timeouts, &mut expired);

            for (_, id) in expired {
                self.states.remove(&id);
            }

            let dur = match self.timeouts.first_key_value() {
                Some(((expire, _), _)) => *expire,
                None => Instant::now() + Duration::from_secs(24 * 3600), // 24h because MAX leads to overflow
            };

            sleep_until(dur).await;
        }
    }

    async fn handle_req(&mut self, req: StateReq) {
        match req {
            StateReq::Insert(id, state) => {
                let evict = Instant::now() + Duration::from_secs(60);

                let mut sub = 0;
                while self.timeouts.contains_key(&(evict, sub)) {
                    sub += 1;
                }

                self.states.insert(id, (state, (evict, sub)));
                self.timeouts.insert((evict, sub), id);
            }
            StateReq::Get(id, resp) => {
                let Some((state, evict)) = self.states.remove(&id) else {
                    let _ = resp.send_async(None).await;
                    return;
                };
                self.timeouts.remove(&evict);

                let _ = resp.send_async(Some(state)).await;
            }
        }
    }
}

impl UploadStateLease {
    pub async fn submit(&mut self, data: &[u8]) -> eyre::Result<()> {
        self.state.as_mut().unwrap().submit(data).await
    }

    pub async fn complete(mut self) -> eyre::Result<()> {
        self.state.take().unwrap().complete().await
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}

#[axum::async_trait]
impl FromRequestParts<(StateHandle, PgPool)> for UploadStateLease {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &(StateHandle, PgPool),
    ) -> Result<Self, Self::Rejection> {
        let Path(id) = Path::<Uuid>::from_request_parts(parts, state)
            .await
            .map_err(IntoResponse::into_response)?;

        let (tx, rx) = bounded(0);
        state
            .0
            .sender
            .send_async(StateReq::Get(id, tx))
            .await
            .expect("this service should be available");

        let Ok(Some(up_state)) = rx.recv_async().await else {
            return Err(StatusCode::NOT_FOUND.into_response());
        };

        Ok(UploadStateLease {
            id,
            handle: state.0.clone(),
            state: Some(up_state),
        })
    }
}

impl Drop for UploadStateLease {
    fn drop(&mut self) {
        let Some(state) = self.state.take() else { return };

        let _ = self
            .handle
            .sender
            .send(StateReq::Insert(self.id, state));
    }
}
