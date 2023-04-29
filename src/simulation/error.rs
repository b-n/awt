#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Cannot call {0} on a running simulation")]
    Enabled(String),
}
