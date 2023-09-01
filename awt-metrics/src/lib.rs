#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(unknown_lints)]
#![warn(missing_debug_implementation)]
#![warn(missing_copy_implementation)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(variant_size_difference)]

extern crate alloc;

use core::time::Duration;
use core::{fmt, fmt::Display, fmt::Formatter};

use awt_simulation::request::{Data as RequestData, Status};

mod aggregator;
mod target;
mod value;

pub use aggregator::Aggregator;
pub use target::{Target, TargetCondition};
use value::Value;

/// Enumerates a metric to trace on a `Request`.
#[allow(clippy::module_name_repetitions)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum MetricType {
    /// Percent of `Client`s answered in `tick`.
    ServiceLevel(Duration),
    /// Mean of work time of answered `Request`.
    AverageWorkTime,
    /// Mean for `tick` of answered `Request`.
    AverageSpeedAnswer,
    /// Mean of `tick` of abandoned `Request` (at abandon).
    AverageTimeToAbandon,
    /// Percent of `Request` abandoned vs. total `Request` count.
    AbandonRate,
    /// Mean of `tick` of `Request` in queue for both answered and abandoned.
    AverageTimeInQueue,
    /// Percent of `Server` time spent answering `Request`s.
    UtilisationTime,
    /// Count of Answered
    AnswerCount,
}

#[derive(Clone, Debug)]
pub struct Metric {
    metric_type: MetricType,
    value: Value,
    target: Target,
    target_condition: TargetCondition,
}

impl Display for Metric {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct MetricError {}

// Structure and setup
impl Metric {
    /// Create a `Metric` with the provided Target. Metrics require a specific type of target,
    /// otherwise it will Error. The supported target to metric types are:
    ///
    /// `Target::MeanDuration`:
    ///
    /// - `MetricType::AverageWorkTime`
    /// - `MetricType::AverageSpeedAnswer`
    /// - `MetricType::AverageTimeInQueue`
    /// - `MetricType::AverageTimeToAbandon`
    ///
    /// `Target::Percent`:
    ///
    /// - `MetricType::UtilisationTime`
    /// - `MetricType::ServiceLevel(_)`
    /// - `MetricType::AbandonRate`
    ///
    /// `Target::Count`:
    ///
    /// - `MetricType::AnswerCount`
    ///
    /// # Errors
    ///
    /// Will error if not using the correct target mapping
    #[allow(clippy::match_same_arms)]
    pub fn with_target(metric_type: MetricType, target: Target) -> Result<Self, MetricError> {
        match (metric_type, target.clone()) {
            (
                MetricType::AverageWorkTime
                | MetricType::AverageSpeedAnswer
                | MetricType::AverageTimeInQueue
                | MetricType::AverageTimeToAbandon,
                Target::MeanDuration(_),
            ) => Ok(Self {
                metric_type,
                value: Value::default_mean_duration(),
                target,
                target_condition: TargetCondition::LessOrEqual,
            }),
            (MetricType::UtilisationTime | MetricType::ServiceLevel(_), Target::Percent(_)) => {
                Ok(Self {
                    metric_type,
                    value: Value::default_percent(),
                    target,
                    target_condition: TargetCondition::GreaterOrEqual,
                })
            }
            (MetricType::AbandonRate, Target::Percent(_)) => Ok(Self {
                metric_type,
                value: Value::default_percent(),
                target,
                target_condition: TargetCondition::LessOrEqual,
            }),

            (MetricType::AnswerCount, Target::Count(_)) => Ok(Self {
                metric_type,
                value: Value::default_count(),
                target,
                target_condition: TargetCondition::Equal,
            }),
            (_, Target::MeanDuration(_) | Target::Percent(_) | Target::Count(_)) => {
                Err(MetricError {})
            }
        }
    }

    #[must_use]
    pub fn metric(&self) -> MetricType {
        self.metric_type
    }

    #[must_use]
    pub fn on_target(&self) -> bool {
        if let TargetCondition::Equal = self.target_condition {
            return self.value == self.target;
        }

        true
    }
}

// Reporting functions
impl Metric {
    /// Report a value for this metric
    ///
    /// # Panics
    ///
    /// Will panic if attempt to report for other aggregate types
    #[allow(clippy::match_same_arms)]
    pub fn report(&mut self, r: &RequestData) {
        match (self.metric_type, r.status, &mut self.value) {
            (MetricType::ServiceLevel(ticks), Status::Answered, Value::Percent(m)) => {
                if let Some(tick) = r.wait_time {
                    m.report(tick <= ticks);
                }
            }
            (MetricType::AverageWorkTime, Status::Answered, Value::MeanDuration(m)) => {
                if let Some(tick) = r.handle_time {
                    m.report(tick);
                }
            }
            (MetricType::AverageSpeedAnswer, Status::Answered, Value::MeanDuration(m)) => {
                if let Some(tick) = r.wait_time {
                    m.report(tick);
                }
            }
            (MetricType::AverageTimeToAbandon, Status::Abandoned, Value::MeanDuration(m)) => {
                if let Some(tick) = r.wait_time {
                    m.report(tick);
                }
            }
            (MetricType::AbandonRate, _, Value::Percent(m)) => {
                m.report(Status::Abandoned == r.status);
            }
            (MetricType::AverageTimeInQueue, _, Value::MeanDuration(m)) => {
                if let Some(tick) = r.wait_time {
                    m.report(tick);
                }
            }
            (MetricType::AnswerCount, Status::Answered, Value::Count(m)) => m.report(),
            (MetricType::UtilisationTime, _, _) => todo!(),
            _ => (),
        }
    }
}
