use ::rucat_common::{
    config::DatabaseConfig, database::Database, error::Result, serde::Deserialize,
};
use authentication::{auth, Authenticate};
use axum::{extract::State, middleware, routing::get, Router};
use engine::router::get_engine_router;
use state::AppState;
use tower_http::trace::TraceLayer;

pub mod authentication;
pub(crate) mod engine;
pub(crate) mod error;
pub(crate) mod state;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(crate = "rucat_common::serde")]
pub enum AuthProviderVariant {
    StaticAuthProviderConfig {
        username: String,
        password: String,
        bearer_token: String,
    },
}

/// Configuration for rucat server
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[serde(crate = "rucat_common::serde")]
pub struct ServerConfig {
    pub auth_provider: Option<AuthProviderVariant>,
    pub database: DatabaseConfig,
}

/// This is the only entry for users to get the rucat server.
/// # Return the router for the server
pub fn get_server<DB, AuthProvider>(
    db_client: DB,
    auth_provider: Option<AuthProvider>,
) -> Result<Router>
where
    DB: Database,
    AuthProvider: Authenticate,
{
    let app_state = AppState::new(db_client, auth_provider);

    // go through the router from outer to inner
    let router = Router::new()
        .route(
            "/",
            get(|_: State<AppState<DB, AuthProvider>>| async { "welcome to rucat" }),
        )
        .nest("/engine", get_engine_router())
        // TODO: use tower::ServiceBuilder to build the middleware stack
        // but need to be careful with the order of the middleware and the compatibility with axum::option_layer
        .layer(middleware::from_fn_with_state(app_state.clone(), auth))
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
    fn disable_auth() {
        let config = json!(
            {
                "database": {
                    "credentials": null,
                    "uri": ""
                }
            }
        );
        let result = from_value::<ServerConfig>(config).unwrap();
        assert_eq!(
            result,
            ServerConfig {
                auth_provider: None,
                database: DatabaseConfig {
                    credentials: None,
                    uri: "".to_string()
                }
            }
        );
    }

    #[test]
    fn missing_field_database() {
        let config = json!(
            {
                "auth_provider": {
                    "StaticAuthProviderConfig": {
                        "username": "admin",
                        "password": "123",
                        "bearer_token": "abc"
                    }
                }
            }
        );
        let result = from_value::<ServerConfig>(config);
        assert_eq!(result.unwrap_err().to_string(), "missing field `database`");
    }

    #[test]
    fn deny_unknown_fields() {
        let config = json!(
            {
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
            "unknown field `unknown_field`, expected `auth_provider` or `database`"
        );
    }

    #[test]
    fn deserialize_server_config() -> Result<()> {
        let config = json!(
            {
                "auth_provider": {
                    "StaticAuthProviderConfig": {
                        "username": "admin",
                        "password": "123",
                        "bearer_token": "abc"
                    }
                },
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
                auth_provider: Some(AuthProviderVariant::StaticAuthProviderConfig {
                    username: "admin".to_string(),
                    password: "123".to_string(),
                    bearer_token: "abc".to_string()
                }),
                database: DatabaseConfig {
                    credentials: None,
                    uri: "".to_string()
                }
            }
        );
        Ok(())
    }
}
