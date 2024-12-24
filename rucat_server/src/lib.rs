use ::rucat_common::{
    config::DatabaseConfig, database::Database, error::Result, serde::Deserialize,
};
use authentication::auth;
use axum::{extract::State, middleware, routing::get, Router};
use axum_extra::middleware::option_layer;
use engine::router::get_engine_router;
use state::AppState;
use tower_http::trace::TraceLayer;

pub(crate) mod authentication;
pub(crate) mod engine;
pub(crate) mod state;

/// Configuration for rucat server
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[serde(crate = "rucat_common::serde")]
pub struct ServerConfig {
    pub auth_enable: bool,
    pub database: DatabaseConfig,
}

/// This is the only entry for users to get the rucat server.
/// # Return the router for the server
pub fn get_server<DB>(auth_enable: bool, db_client: DB) -> Result<Router>
where
    DB: Database,
{
    let app_state = AppState::new(db_client);

    // go through the router from outer to inner
    let router = Router::new()
        .route(
            "/",
            get(|_: State<AppState<DB>>| async { "welcome to rucat" }),
        )
        .nest("/engine", get_engine_router())
        // TODO: use tower::ServiceBuilder to build the middleware stack
        // but need to be careful with the order of the middleware and the compatibility with axum::option_layer
        .layer(option_layer(auth_enable.then(|| middleware::from_fn(auth))))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);
    Ok(router)
}

#[cfg(test)]
mod tests {
    use ::rucat_common::{
        anyhow::Result,
        serde_json::{from_value, json},
    };

    use super::*;

    #[test]
    fn missing_field_auth_enable() {
        let config = json!(
            {
                "database": {
                    "credentials": null,
                    "uri": ""
                }
            }
        );
        let result = from_value::<ServerConfig>(config);
        assert_eq!(
            result.unwrap_err().to_string(),
            "missing field `auth_enable`"
        );
    }

    #[test]
    fn missing_field_database() {
        let config = json!(
            {
                "auth_enable": true
            }
        );
        let result = from_value::<ServerConfig>(config);
        assert_eq!(result.unwrap_err().to_string(), "missing field `database`");
    }

    #[test]
    fn deny_unknown_fields() {
        let config = json!(
            {
                "auth_enable": true,
                "database": {
                    "credentials": null,
                    "uri": ""
                },
                "unknown_field": "unknown"
            }
        );
        let result = from_value::<ServerConfig>(config);
        assert_eq!(
            result.unwrap_err().to_string(),
            "unknown field `unknown_field`, expected `auth_enable` or `database`"
        );
    }

    #[test]
    fn deserialize_server_config() -> Result<()> {
        let config = json!(
            {
                "auth_enable": true,
                "database": {
                    "credentials": null,
                    "uri": "",
                }
            }
        );
        let result = from_value::<ServerConfig>(config)?;
        assert_eq!(
            result,
            ServerConfig {
                auth_enable: true,
                database: DatabaseConfig {
                    credentials: None,
                    uri: "".to_string()
                }
            }
        );
        Ok(())
    }
}
