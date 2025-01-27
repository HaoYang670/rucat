use ::core::fmt::Display;

use ::axum::response::{IntoResponse, Response};
use ::http::StatusCode;
use ::rucat_common::error::{RucatError, RucatErrorType::*};

/// [RucatServerError] is a wrapper for [RucatError] to convert it into Axum response
pub struct RucatServerError(RucatError);

impl RucatServerError {
    fn get_status_code(&self) -> StatusCode {
        match self.0.get_error_type() {
            NotFound => StatusCode::NOT_FOUND,
            Unauthorized => StatusCode::UNAUTHORIZED,
            NotAllowed => StatusCode::FORBIDDEN,
            FailToStartServer => StatusCode::INTERNAL_SERVER_ERROR,
            FailToStartStateMonitor => StatusCode::INTERNAL_SERVER_ERROR,
            FailToStartEngine => StatusCode::INTERNAL_SERVER_ERROR,
            FailToDeleteEngine => StatusCode::INTERNAL_SERVER_ERROR,
            FailToConnectDatabase => StatusCode::INTERNAL_SERVER_ERROR,
            FailToUpdateDatabase => StatusCode::INTERNAL_SERVER_ERROR,
            FailToReadDatabase => StatusCode::INTERNAL_SERVER_ERROR,
            FailToLoadConfig => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<RucatError> for RucatServerError {
    fn from(error: RucatError) -> Self {
        Self(error)
    }
}

/// [RucatServerError] displays in the same way as [RucatError]
impl Display for RucatServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl IntoResponse for RucatServerError {
    fn into_response(self) -> Response {
        let status = self.get_status_code();
        (status, self.to_string()).into_response()
    }
}

impl<T> From<RucatServerError> for Result<T, RucatServerError> {
    fn from(val: RucatServerError) -> Self {
        Result::Err(val)
    }
}

#[cfg(test)]
mod tests {
    use ::rucat_common::engine::EngineId;

    use super::*;

    #[test]
    fn display_error() {
        let error: RucatServerError =
            RucatError::engine_not_found(&EngineId::try_from("0").unwrap()).into();
        assert!(error
            .to_string()
            .starts_with("Not found: Engine 0 not found"));
    }
}
