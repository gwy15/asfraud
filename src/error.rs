use std::fmt;

use actix_web::ResponseError;

pub struct Error {
    code: http::StatusCode,
    e: anyhow::Error,
}
impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Self {
            code: http::StatusCode::INTERNAL_SERVER_ERROR,
            e: value,
        }
    }
}

impl Error {
    #[allow(unused)]
    pub fn with_code(mut self, code: http::StatusCode) -> Self {
        self.code = code;
        self
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.e.fmt(f)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.e.fmt(f)
    }
}
impl std::error::Error for Error {}

impl ResponseError for Error {
    fn status_code(&self) -> http::StatusCode {
        self.code
    }
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.code).json(serde_json::json!({
            "errmsg": self.e.to_string(),
            "detail": format!("{:#?}", self.e),
        }))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
pub use anyhow::Context;
