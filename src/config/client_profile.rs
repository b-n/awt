use core::time::Duration;
use serde::Deserialize;

use crate::Attribute;
use crate::Client;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ClientProfile {
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

impl Default for ClientProfile {
    fn default() -> Self {
        let client = Client::default();
        Self {
            required_attributes: client.required_attributes.clone(),
            quantity: 1,
            handle_time: client.handle_time,
            clean_up_time: client.clean_up_time,
            abandon_time: client.abandon_time,
        }
    }
}
