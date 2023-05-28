#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Attribute {
    pub id: u64,
    pub level: Option<usize>,
}

impl Attribute {
    #[must_use]
    pub fn new(id: u64, level: Option<usize>) -> Self {
        Self { id, level }
    }
}
