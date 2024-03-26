use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

pub async fn auth(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if token_is_valid(bearer.token()) {
        let response = next.run(request).await;
        Ok(response)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

fn token_is_valid(token: &str) -> bool {
    token.eq("remziy")
}
