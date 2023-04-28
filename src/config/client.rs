use core::time::Duration;
use serde::Deserialize;

use crate::Attribute;
use crate::Client as SimulationClient;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Client {
    /// Default is an empty attribute array.
    #[serde(default)]
    pub required_attributes: Vec<Attribute>,
    pub handle_time: Duration,
    /// Default is 0 secs, 0 nanos.
    #[serde(default)]
    pub clean_up_time: Duration,
    pub abandon_time: Duration,
    pub quantity: usize,
}

impl From<&Client> for SimulationClient {
    fn from(c: &Client) -> Self {
        Self {
            required_attributes: c.required_attributes.clone(),
            handle_time: c.handle_time,
            clean_up_time: c.clean_up_time,
            abandon_time: c.abandon_time,
        }
    }
}
