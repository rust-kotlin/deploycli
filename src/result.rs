use salvo::{Scribe, http::StatusCode, writing::Json};
use serde::Serialize;
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("error:`{0}`")]
    AnyHow(#[from] anyhow::Error),
    #[error("io error:`{0}`")]
    Io(#[from] std::io::Error),
}

pub type AppResult = Result<Success, AppError>;

pub struct Success(Value);

impl<T: Serialize> From<T> for Success {
    fn from(data: T) -> Self {
        Success(json!(data))
    }
}

impl Scribe for Success {
    fn render(self, res: &mut salvo::Response) {
        if self.0 == 0 {
            // 0代表什么都不做
            return;
        }
        res.stuff(StatusCode::OK, Json(self.0));
    }
}

impl Scribe for AppError {
    fn render(self, res: &mut salvo::Response) {
        match self {
            AppError::AnyHow(e) => res.stuff(
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(format!("Internal Server Error: {}", e)),
            ),
            AppError::Io(e) => res.stuff(
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(format!("IO Error: {}", e)),
            ),
        }
    }
}