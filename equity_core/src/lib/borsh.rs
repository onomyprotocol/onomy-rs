use std::io;

use async_trait::async_trait;
use axum::{
    body::{Bytes, HttpBody},
    extract::{rejection::BytesRejection, FromRequest},
    http::{self, HeaderValue},
    response::{IntoResponse, Response},
    BoxError,
};
use borsh::{BorshDeserialize, BorshSerialize};
use hyper::{header, StatusCode};

/// A `Borsh` analog to `axum::Json`
#[derive(Debug, Clone, Copy, Default)]
pub struct Borsh<T>(pub T);

// TODO serialize the rejection itself on the way back?
#[derive(Debug)]
pub enum BorshRejection {
    BorshDeserializationError(io::Error),
    BytesRejectionError(BytesRejection),
}

impl IntoResponse for BorshRejection {
    fn into_response(self) -> Response {
        (
            http::StatusCode::UNPROCESSABLE_ENTITY,
            format!("{:?}", self),
        )
            .into_response()
    }
}

#[async_trait]
impl<T, B> FromRequest<B> for Borsh<T>
where
    T: BorshDeserialize,
    B: HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = BorshRejection;

    async fn from_request(
        req: &mut axum::extract::RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req)
            .await
            .map_err(BorshRejection::BytesRejectionError)?;
        match BorshDeserialize::try_from_slice(&bytes) {
            Ok(t) => Ok(Borsh(t)),
            Err(e) => Err(BorshRejection::BorshDeserializationError(e)),
        }
    }
}

impl<T> IntoResponse for Borsh<T>
where
    T: BorshSerialize,
{
    fn into_response(self) -> Response {
        match borsh::to_vec(&self.0) {
            Ok(bytes) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_OCTET_STREAM.as_ref()),
                )],
                bytes,
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
