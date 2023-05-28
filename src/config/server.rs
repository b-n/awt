use serde::Deserialize;

use super::Attribute;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Server {
    /// Default is an empty attribute array.
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    pub quantity: usize,
}

impl From<&Server> for crate::Server {
    fn from(s: &Server) -> Self {
        Self {
            attributes: s.attributes.iter().map(crate::Attribute::from).collect(),
            ..Self::default()
        }
    }
}
