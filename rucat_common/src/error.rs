use std::fmt::Display;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub type Result<T> = std::result::Result<T, RucatError>;

#[derive(Debug, PartialEq)]
pub enum RucatError {
    CannotChangeStorageLevelError,
    IllegalArgument(String),
    NotFoundError(String),
    UnauthorizedError(String),
    NotAllowedError(String),
    IOError(String),
    DataStoreError(String),
    Other(String),
}

impl Display for RucatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: rewrite this in macro
        match self {
            Self::CannotChangeStorageLevelError => write!(f, "Cannot change storage level error"),
            Self::IllegalArgument(msg) => write!(f, "Illegal Argument error: {}", msg),
            Self::NotFoundError(msg) => write!(f, "Not found error: {}", msg),
            Self::UnauthorizedError(msg) => write!(f, "Unauthorized error: {}", msg),
            Self::NotAllowedError(msg) => write!(f, "Not allowed error: {}", msg),
            Self::IOError(msg) => write!(f, "IO error: {}", msg),
            Self::DataStoreError(msg) => write!(f, "Data store error: {}", msg),
            Self::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl IntoResponse for RucatError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::IllegalArgument(_) => StatusCode::BAD_REQUEST,
            Self::NotFoundError(_) => StatusCode::NOT_FOUND,
            Self::UnauthorizedError(_) => StatusCode::UNAUTHORIZED,
            Self::NotAllowedError(_) => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

impl<T> From<RucatError> for Result<T> {
    fn from(val: RucatError) -> Self {
        Result::Err(val)
    }
}

macro_rules! convert_to_rucat_error {
    ($err_ty: ty, $constructor: expr) => {
        impl From<$err_ty> for RucatError {
            fn from(value: $err_ty) -> Self {
                $constructor(value.to_string())
            }
        }
    };
}

convert_to_rucat_error!(std::io::Error, RucatError::IOError);
convert_to_rucat_error!(surrealdb::Error, RucatError::DataStoreError);
convert_to_rucat_error!(anyhow::Error, RucatError::Other);
convert_to_rucat_error!(String, RucatError::Other);
