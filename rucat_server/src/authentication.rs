//! authentication middleware

use std::panic::catch_unwind;

use axum::{extract::Request, http::HeaderMap, middleware::Next, response::Response};
use axum_extra::headers::authorization::{Basic, Bearer, Credentials as _};

use ::rucat_common::error::PrimaryRucatError;
use rucat_common::error::{Result, RucatError};

enum Credentials {
    Basic(Basic),
    Bearer(Bearer),
}

/// authentication
pub(crate) async fn auth(headers: HeaderMap, request: Request, next: Next) -> Result<Response> {
    let credentials = get_credentials(&headers)?;
    if validate_credentials(&credentials) {
        let response = next.run(request).await;
        Ok(response)
    } else {
        Err(RucatError::unauthorized(PrimaryRucatError(
            "wrong credentials".to_owned(),
        )))
    }
}

/// Get Basic or Bearer credentials
fn get_credentials(headers: &HeaderMap) -> Result<Credentials> {
    let token = headers.get(http::header::AUTHORIZATION).ok_or_else(|| {
        RucatError::unauthorized(PrimaryRucatError(
            "Not found authorization header".to_owned(),
        ))
    })?;
    // Use std::panic::catch_unwind to catch the debug_assert in Basic::decode and Bearer::decode
    catch_unwind(|| Basic::decode(token))
        .unwrap_or(None)
        .map(Credentials::Basic)
        .or_else(|| {
            catch_unwind(|| Bearer::decode(token))
                .unwrap_or(None)
                .map(Credentials::Bearer)
        })
        .ok_or_else(|| {
            RucatError::unauthorized(PrimaryRucatError("Unsupported credentials type".to_owned()))
        })
}

fn validate_credentials(token: &Credentials) -> bool {
    match token {
        Credentials::Basic(basic) => basic.username().eq("remzi") && basic.password().eq("yang"),
        Credentials::Bearer(bearer) => bearer.token().eq("remziy"),
    }
}
