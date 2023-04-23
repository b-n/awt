use crate::Attribute;

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
        Self {
            required_attributes: vec![],
            quantity: 1,
            handle_time: 300_000,
            clean_up_time: 0,
            abandon_time: 30_000,
        }
    }
}
