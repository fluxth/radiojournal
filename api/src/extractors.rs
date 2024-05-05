use axum::extract::FromRequestParts;

use crate::errors::APIError;

#[derive(FromRequestParts, Debug)]
#[from_request(via(axum::extract::Path), rejection(APIError))]
pub(crate) struct Path<T>(pub(crate) T);

#[derive(FromRequestParts, Debug)]
#[from_request(via(axum::extract::Query), rejection(APIError))]
pub(crate) struct Query<T>(pub(crate) T);
