use alloc::vec::Vec;
use core::time::Duration;

use crate::Attribute;

const FIVE_MINUTES: Duration = Duration::new(300, 0);
const THIRTY_SECONDS: Duration = Duration::new(30, 0);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Client {
    pub required_attributes: Vec<Attribute>,
    pub handle_time: Duration,
    pub clean_up_time: Duration,
    pub abandon_time: Duration,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            required_attributes: Vec::new(),
            handle_time: FIVE_MINUTES,
            clean_up_time: Duration::ZERO,
            abandon_time: THIRTY_SECONDS,
        }
    }
}
