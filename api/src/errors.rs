use std::borrow::Cow;

use axum::{
    extract::rejection::{JsonRejection, PathRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::models::APIJson;

pub(crate) enum APIError {
    NotFound,
    ValidationFailed { message: Option<&'static str> },
    InputRejection { message: String },
}

#[derive(Debug, Serialize, ToSchema)]
pub struct APIErrorResponse {
    error: APIErrorDetail,
}

#[derive(Debug, Serialize, ToSchema)]
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
            Self::ValidationFailed { message } => (
                StatusCode::BAD_REQUEST,
                APIJson(APIErrorResponse {
                    error: APIErrorDetail {
                        code: "VALIDATION_FAILED",
                        message: if let Some(message) = message {
                            Cow::Borrowed(message)
                        } else {
                            Cow::Borrowed("Validation failed on user input")
                        },
                    },
                }),
            )
                .into_response(),

            Self::InputRejection { message } => (
                StatusCode::BAD_REQUEST,
                APIJson(APIErrorResponse {
                    error: APIErrorDetail {
                        code: "BAD_REQUEST",
                        message: Cow::Owned(message),
                    },
                }),
            )
                .into_response(),
        }
    }
}

impl From<JsonRejection> for APIError {
    fn from(rejection: JsonRejection) -> Self {
        Self::InputRejection {
            message: rejection.body_text(),
        }
    }
}

impl From<QueryRejection> for APIError {
    fn from(rejection: QueryRejection) -> Self {
        Self::InputRejection {
            message: rejection.body_text(),
        }
    }
}

impl From<PathRejection> for APIError {
    fn from(rejection: PathRejection) -> Self {
        Self::InputRejection {
            message: rejection.body_text(),
        }
    }
}
