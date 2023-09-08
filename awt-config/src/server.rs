use serde::Deserialize;

use crate::Attribute;
use awt_simulation::{
    attribute::Attribute as SimulationAttribute, server::Server as SimulationServer,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Server {
    /// Default is an empty attribute array.
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    pub quantity: usize,
}

impl From<&Server> for SimulationServer {
    fn from(s: &Server) -> Self {
        Self {
            attributes: s.attributes.iter().map(SimulationAttribute::from).collect(),
            ..Self::default()
        }
    }
}
