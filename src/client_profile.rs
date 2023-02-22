use crate::{Attribute, TICKS_PER_SECOND};

#[derive(Debug, Clone)]
pub struct ClientProfile {
    pub required_attributes: Vec<Attribute>,
    base_handle_time: usize,
    base_clean_up_time: usize,
    base_abandon_tick: usize,
    pub quantity: usize,
}

impl Default for ClientProfile {
    fn default() -> Self {
        Self {
            required_attributes: vec![],
            quantity: 1,
            base_handle_time: 0,
            base_clean_up_time: 0,
            base_abandon_tick: TICKS_PER_SECOND * 30, 
        }
    }
}

