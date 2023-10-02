use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use eyre::Report;
use std::fmt::{Debug, Display, Formatter};
use tracing::error;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, WebReport>;

#[derive(Debug)]
pub struct WebReport(Report);

impl<T: Into<Report>> From<T> for WebReport {
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
