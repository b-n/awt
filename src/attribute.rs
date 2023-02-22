#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    name: String,
    level: Option<usize>,
}

impl Attribute {
    pub fn new(name: &str, level: Option<usize>) -> Self {
        Self { name: name.to_string(), level }
    }
}
