use async_trait::async_trait;
use axum::{
    body::{Bytes, HttpBody},
    extract::{rejection::BytesRejection, FromRequest},
    http::{self, HeaderValue},
    response::{IntoResponse, Response},
    BoxError,
};
use hyper::{header, StatusCode};
use serde::{de::DeserializeOwned, Serialize};

/// A `Ron` analog to `axum::Json`
#[derive(Debug, Clone, Copy, Default)]
pub struct Ron<T>(pub T);

// TODO serialize the rejection itself on the way back?
#[derive(Debug)]
pub enum RonRejection {
    RonDeserializationError(ron::Error),
    BytesRejectionError(BytesRejection),
}

impl IntoResponse for RonRejection {
    fn into_response(self) -> Response {
        (
            http::StatusCode::UNPROCESSABLE_ENTITY,
            format!("{:?}", self),
        )
            .into_response()
    }
}

#[async_trait]
impl<T, B> FromRequest<B> for Ron<T>
where
    T: DeserializeOwned,
    B: HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = RonRejection;

    async fn from_request(
        req: &mut axum::extract::RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req)
            .await
            .map_err(RonRejection::BytesRejectionError)?;
        match ron::de::from_bytes(&bytes) {
            Ok(t) => Ok(Ron(t)),
            Err(e) => Err(RonRejection::RonDeserializationError(e)),
        }
    }
}

impl<T> IntoResponse for Ron<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match ron::to_string(&self.0) {
            Ok(s) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_OCTET_STREAM.as_ref()),
                )],
                s,
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}
