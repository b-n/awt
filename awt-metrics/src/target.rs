use core::time::Duration;

use crate::value::{Count, MeanDuration, Percent, Value};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub enum TargetCondition {
    LessOrEqual,
    GreaterOrEqual,
    Equal,
}

pub type Target = Value;

impl Target {
    pub fn mean_duration(duration: Duration) -> Self {
        Self::MeanDuration(MeanDuration {
            sum: duration,
            count: 1,
        })
    }

    pub fn count(count: usize) -> Self {
        Self::Count(Count { count })
    }

    pub fn percent(percent: f64) -> Self {
        Self::Percent(Percent {
            sum: percent,
            count: 1f64,
        })
    }
}
