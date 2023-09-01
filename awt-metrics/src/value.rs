use core::cmp::Ordering;
use core::time::Duration;
use core::{fmt, fmt::Display, fmt::Formatter};

#[derive(Clone, Debug)]
pub enum Value {
    MeanDuration(MeanDuration),
    Count(Count),
    Percent(Percent),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MeanDuration(a) => write!(f, "{a}"),
            Self::Count(a) => write!(f, "{a}"),
            Self::Percent(a) => write!(f, "{a}"),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::MeanDuration(a), Value::MeanDuration(b)) => Some(a.cmp(b)),
            (Value::Count(a), Value::Count(b)) => Some(a.cmp(b)),
            (Value::Percent(a), Value::Percent(b)) => Some(a.cmp(b)),
            _ => None,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::MeanDuration(a), Value::MeanDuration(b)) => a == b,
            (Value::Count(a), Value::Count(b)) => a == b,
            (Value::Percent(a), Value::Percent(b)) => a == b,
            _ => false,
        }
    }
}

impl Value {
    pub fn default_mean_duration() -> Self {
        Self::MeanDuration(MeanDuration::default())
    }
    pub fn default_count() -> Self {
        Self::Count(Count::default())
    }
    pub fn default_percent() -> Self {
        Self::Percent(Percent::default())
    }
}

// MeanDuration Counters are used to provide a Mean of the provided Duration values.
#[derive(Clone, Debug, Default, Eq)]
pub struct MeanDuration {
    pub sum: Duration,
    pub count: u32,
}

impl Display for MeanDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.count == 0 {
            return write!(f, "None");
        }

        write!(f, "{:?}", self.sum / self.count)
    }
}

impl Ord for MeanDuration {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.sum / self.count).cmp(&(other.sum / other.count))
    }
}

impl PartialOrd for MeanDuration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for MeanDuration {
    fn eq(&self, other: &Self) -> bool {
        (self.sum / self.count) == (other.sum / other.count)
    }
}

impl MeanDuration {
    pub fn report(&mut self, duration: Duration) {
        self.count += 1;
        self.sum += duration;
    }
}

// Countable are metrics which we only want to count. e.g we have X requests
// Report: Just report()
#[derive(Clone, Debug, Default, Eq)]
pub struct Count {
    pub count: usize,
}

impl Display for Count {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.count)
    }
}

impl Ord for Count {
    fn cmp(&self, other: &Self) -> Ordering {
        self.count.cmp(&other.count)
    }
}

impl PartialOrd for Count {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Count {
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count
    }
}

impl Count {
    pub fn report(&mut self) {
        self.count += 1;
    }
}

// Percentable is a count of matching values against a total available
// Report: Position and negative. e.g. Things that in bounds, and things that
//         out of bounds
//         report_bool(value: bool)
#[derive(Clone, Debug, Default)]
pub struct Percent {
    pub sum: f64,
    pub count: f64,
}

impl Display for Percent {
    #[allow(clippy::cast_precision_loss)]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.count == 0f64 {
            return write!(f, "None");
        }

        write!(f, "{}", self.sum / self.count)
    }
}

impl Ord for Percent {
    fn cmp(&self, other: &Self) -> Ordering {
        // we use partial_cmp since f64 doesn't implement Ord
        (self.sum / self.count)
            .partial_cmp(&(other.sum / other.count))
            .unwrap()
    }
}

impl PartialOrd for Percent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Percent {
    fn eq(&self, other: &Self) -> bool {
        self.sum == other.sum && self.count == other.count
    }
}

impl Eq for Percent {}

impl Percent {
    pub fn report(&mut self, in_range: bool) {
        if in_range {
            self.sum += 1f64;
        }
        self.count += 1f64;
    }
}
