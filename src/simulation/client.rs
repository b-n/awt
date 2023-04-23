use core::time::Duration;

use super::ClientProfile;
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

impl From<&ClientProfile> for Client {
    fn from(cp: &ClientProfile) -> Self {
        Self {
            required_attributes: cp.required_attributes.clone(),
            handle_time: cp.handle_time,
            clean_up_time: cp.clean_up_time,
            abandon_time: cp.abandon_time,
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self {
            required_attributes: vec![],
            handle_time: FIVE_MINUTES,
            clean_up_time: Duration::ZERO,
            abandon_time: THIRTY_SECONDS,
        }
    }
}
