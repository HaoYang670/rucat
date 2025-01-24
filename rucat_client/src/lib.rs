pub mod engine;

pub enum Credentials<'a> {
    Basic {
        username: &'a str,
        password: &'a str,
    },
    Bearer {
        token: &'a str,
    },
}
