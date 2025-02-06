use axum::{body::Body, http::Response, response::IntoResponse};

pub type AppResult<T = AppOk> = Result<T, AppError>;

pub struct AppOk;

impl IntoResponse for AppOk {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .status(200)
            .body(Body::from("ok"))
            .unwrap()
    }
}

pub enum AppError {
    NotFound,
    Error(anyhow::Error),
}

impl From<bluer::Error> for AppError {
    fn from(inner: bluer::Error) -> Self {
        if inner.kind == bluer::ErrorKind::NotFound {
            Self::NotFound
        } else {
            Self::Error(inner.into())
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(inner: anyhow::Error) -> Self {
        Self::Error(inner)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        match self {
            AppError::NotFound => Response::builder()
                .status(404)
                .body(Body::from("not found"))
                .unwrap(),
            AppError::Error(error) => Response::builder()
                .status(500)
                .body(Body::from(error.to_string()))
                .unwrap(),
        }
    }
}
