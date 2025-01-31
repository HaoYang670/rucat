use super::{Authenticate, Credentials};

pub struct StaticAuthProvider {
    username: String,
    password: String,
    bearer_token: String,
}

impl StaticAuthProvider {
    pub fn new(username: String, password: String, bearer_token: String) -> Self {
        Self {
            username,
            password,
            bearer_token,
        }
    }
}

impl Authenticate for StaticAuthProvider {
    fn validate(&self, credentials: &Credentials) -> bool {
        match credentials {
            Credentials::Basic(basic) => {
                basic.username().eq(&self.username) && basic.password().eq(&self.password)
            }
            Credentials::Bearer(bearer) => bearer.token().eq(&self.bearer_token),
        }
    }
}
