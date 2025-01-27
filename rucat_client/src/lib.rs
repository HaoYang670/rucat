pub mod resource_client;

pub enum Credentials<'a> {
    Basic {
        username: &'a str,
        password: Option<&'a str>,
    },
    Bearer {
        token: &'a str,
    },
}
