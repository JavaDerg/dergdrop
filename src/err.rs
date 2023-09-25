use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::error;
use uuid::{uuid, Uuid};

pub type Result<T> = std::result::Result<T, WebReport>;

pub struct WebReport(eyre::Report);

impl<T: Into<eyre::Report>> From<T> for WebReport {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl IntoResponse for WebReport {
    fn into_response(self) -> Response {
        let ray = Uuid::new_v4();
        error!(ray = ray.to_string(), "{}", self.0);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal server error; ray={ray}"),
        )
            .into_response()
    }
}
