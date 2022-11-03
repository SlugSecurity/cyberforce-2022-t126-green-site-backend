use serde::Serialize;

#[derive(Serialize)]
#[serde(transparent)]
pub(crate) struct Token(u128);

/// Generates token and puts an SHA-256 hash of it at ``username.key``in the
/// working directory where the username is the provided parameter.
pub(crate) fn generate_token(username: &str) -> Token {
    let rand_token = rand::thread_rng().gen();

    rand_token
}

pub(crate) fn verify_token() {}
