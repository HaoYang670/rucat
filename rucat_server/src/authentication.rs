use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::headers::authorization::{Basic, Bearer, Credentials as _};

/// Authentication types supported
enum Credentials {
    Basic(Basic),
    Bearer(Bearer),
}

/// Bear authentication
pub async fn auth(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match get_token(&headers) {
        Some(token) if token_is_valid(&token) => {
            let response = next.run(request).await;
            Ok(response)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Get Basic or Bearer credentials
fn get_token(headers: &HeaderMap) -> Option<Credentials> {
    let token = headers.get(http::header::AUTHORIZATION);
    token.and_then(|t| {
        Basic::decode(t)
            .map(Credentials::Basic)
            .or_else(|| Bearer::decode(t).map(Credentials::Bearer))
    })
}

fn token_is_valid(token: &Credentials) -> bool {
    match token {
        Credentials::Basic(basic) => basic.username().eq("remzi") && basic.password().eq("yang"),
        Credentials::Bearer(bearer) => bearer.token().eq("remziy"),
    }
}
