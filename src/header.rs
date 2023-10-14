use axum::extract::FromRequestParts;
use axum::http::header::RANGE;
use axum::http::request::Parts;
use axum::http::StatusCode;
use headers::Header;

pub struct Range(headers::Range);

#[axum::async_trait]
impl<S> FromRequestParts<S> for Range {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let range = parts
            .headers
            .get(RANGE)
            .ok_or((StatusCode::BAD_REQUEST, "missing range header"))?;
        Ok(Range(
            headers::Range::decode(&mut [range].into_iter())
                .map_err(|_| (StatusCode::BAD_REQUEST, "invalid range header"))?,
        ))
    }
}
