use serde::Deserialize;

use crate::Attribute;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Server {
    /// Default is an empty attribute array.
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    pub quantity: usize,
}
