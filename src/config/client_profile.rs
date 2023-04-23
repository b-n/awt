use crate::Attribute;
use crate::Client;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientProfile {
    pub required_attributes: Vec<Attribute>,
    pub handle_time: usize,
    pub clean_up_time: usize,
    pub abandon_time: usize,
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
