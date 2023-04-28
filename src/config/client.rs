use core::time::Duration;
use serde::Deserialize;

use crate::Attribute;

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
