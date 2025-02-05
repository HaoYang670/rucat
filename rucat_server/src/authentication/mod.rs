//! authentication middleware

use ::std::sync::Arc;
use std::panic::catch_unwind;

use ::axum::extract::State;
use axum::{extract::Request, http::HeaderMap, middleware::Next, response::Response};
use axum_extra::headers::authorization::{Basic, Bearer, Credentials as _};
use rucat_common::anyhow::anyhow;
use rucat_common::error::RucatError;

use crate::error::RucatServerError;

pub mod static_auth_provider;

type Result<T> = std::result::Result<T, RucatServerError>;

pub enum Credentials {
    Basic(Basic),
    Bearer(Bearer),
}

/// authentication
pub(crate) async fn auth<AuthProvider>(
    State(auth_provider): State<Arc<AuthProvider>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response>
where
    AuthProvider: Authenticate,
{
    let credentials = get_credentials(&headers)?;
    if auth_provider.validate(&credentials) {
        Ok(next.run(request).await)
    } else {
        Err(RucatError::unauthorized(anyhow!("wrong credentials")).into())
    }
}

/// Get credentials from headers
fn get_credentials(headers: &HeaderMap) -> Result<Credentials> {
    let token = headers
        .get(http::header::AUTHORIZATION)
        .ok_or_else(|| RucatError::unauthorized(anyhow!("Not found authorization header")))?;
    // Use std::panic::catch_unwind to catch the debug_assert in Basic::decode and Bearer::decode
    catch_unwind(|| Basic::decode(token))
        .unwrap_or(None)
        .map(Credentials::Basic)
        .or_else(|| {
            catch_unwind(|| Bearer::decode(token))
                .unwrap_or(None)
                .map(Credentials::Bearer)
        })
        .ok_or_else(|| RucatError::unauthorized(anyhow!("Unsupported credentials type")).into())
}

/// Trait for authentication
pub trait Authenticate: Send + Sync + 'static {
    /// Validate the credentials
    fn validate(&self, credentials: &Credentials) -> bool;
}
