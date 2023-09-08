use awt_simulation::attribute::Attribute as SimulationAttribute;
use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Default, Clone, Deserialize, Debug, Eq, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub level: Option<usize>,
}

impl From<&Attribute> for SimulationAttribute {
    fn from(attr: &Attribute) -> Self {
        let mut s = DefaultHasher::new();
        attr.name.hash(&mut s);
        let hash = s.finish();

        Self {
            id: hash,
            level: attr.level,
        }
    }
}
