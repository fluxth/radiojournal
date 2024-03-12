use std::borrow::Cow;

use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::models::APIJson;

pub(crate) enum APIError {
    NotFound,
    JsonRejection(JsonRejection),
}

#[derive(Debug, Serialize)]
pub struct APIErrorResponse {
    error: APIErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct APIErrorDetail {
    code: &'static str,
    message: Cow<'static, str>,
}

impl IntoResponse for APIError {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                APIJson(APIErrorResponse {
                    error: APIErrorDetail {
                        code: "NOT_FOUND",
                        message: Cow::Borrowed("The resource you requested could not be found"),
                    },
                }),
            )
                .into_response(),
            Self::JsonRejection(rejection) => (
                rejection.status(),
                APIJson(APIErrorResponse {
                    error: APIErrorDetail {
                        code: "BAD_INPUT",
                        message: Cow::Owned(rejection.body_text()),
                    },
                }),
            )
                .into_response(),
        }
    }
}

impl From<JsonRejection> for APIError {
    fn from(rejection: JsonRejection) -> Self {
        Self::JsonRejection(rejection)
    }
}
