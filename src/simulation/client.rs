use crate::Attribute;

use super::ClientProfile;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Client {
    pub required_attributes: Vec<Attribute>,
    pub handle_time: usize,
    pub clean_up_time: usize,
    pub abandon_time: usize,
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
            handle_time: 300_000,
            clean_up_time: 0,
            abandon_time: 30_000,
        }
    }
}
