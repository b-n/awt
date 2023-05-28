use core::{fmt, fmt::Display, fmt::Formatter};

#[derive(Debug)]
pub enum Error {
    Enabled,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot modify an enabled simulation")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
