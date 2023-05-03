#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct Attribute {
    name: String,
    level: Option<usize>,
}

impl Attribute {
    #[must_use]
    pub fn new(name: &str, level: Option<usize>) -> Self {
        Self {
            name: name.to_string(),
            level,
        }
    }
}
