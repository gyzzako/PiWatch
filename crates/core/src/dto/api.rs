use serde::{Deserialize, Serialize};
use axum::{
    Json,
    response::{IntoResponse, Response},
    http::StatusCode
};

#[derive(Deserialize, Serialize)]
pub struct DefaultApiResponse {
    pub success: bool,
    pub message: Option<String>,
}

pub enum ApiResponse<T> {
    Success(StatusCode, Json<T>),
    Error(StatusCode, Json<DefaultApiResponse>),
    StatusOnly(StatusCode),
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        match self {
            ApiResponse::Success(code, body) => (code, body).into_response(),

            ApiResponse::Error(code, body) => (code, body).into_response(),

            ApiResponse::StatusOnly(code) => {
                let body = Json(DefaultApiResponse {
                    success: code.is_success(),
                    message: code.canonical_reason().map(|s| s.to_string()),
                });

                (code, body).into_response()
            }
        }
    }
}