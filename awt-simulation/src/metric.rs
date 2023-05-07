use core::time::Duration;
use core::{fmt, fmt::Display, fmt::Formatter};

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
pub enum Aggregate {
    Meanable(Meanable),
    Countable(Countable),
    Percentable(Percentable),
}

impl Display for Aggregate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Meanable(a) => write!(f, "{a}"),
            Self::Countable(a) => write!(f, "{a}"),
            Self::Percentable(a) => write!(f, "{a}"),
        }
    }
}

// Meanable are metrics which we count a total of ticks for, and we want the average of those values
// Report: Just provide a value. a usize report_usize(value: usize)
#[derive(Clone, Debug)]
pub struct Meanable {
    sum: Duration,
    count: u32,
    target: Duration,
}

impl Display for Meanable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.count == 0 {
            return write!(f, "None");
        }

        write!(f, "{:?}", self.sum / self.count)
    }
}

impl Meanable {
    pub fn report_duration(&mut self, value: Duration) {
        self.sum += value;
        self.count += 1;
    }

    #[must_use]
    pub fn with_target(target: Duration) -> Self {
        Self {
            sum: Duration::ZERO,
            count: 0,
            target,
        }
    }

    #[must_use]
    pub fn on_target(&self) -> bool {
        match self.count {
            0 => false,
            _ => self.target < (self.sum / self.count),
        }
    }
}

// Countable are metrics which we only want to count. e.g we have X requests
// Report: Just report()
#[derive(Clone, Debug)]
pub struct Countable {
    count: usize,
    target: usize,
}

impl Display for Countable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.count)
    }
}

impl Countable {
    pub fn report(&mut self) {
        self.count += 1;
    }

    #[must_use]
    pub fn with_target(target: usize) -> Self {
        Self { count: 0, target }
    }

    #[must_use]
    pub fn on_target(&self) -> bool {
        match self.count {
            0 => false,
            _ => self.target < self.count,
        }
    }
}

// Percentable is a count of matching values against a total available
// Report: Position and negative. e.g. Things that in bounds, and things that
//         out of bounds
//         report_bool(value: bool)
#[derive(Clone, Debug)]
pub struct Percentable {
    sum: usize,
    count: usize,
    target: f64,
}

impl Display for Percentable {
    #[allow(clippy::cast_precision_loss)]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.count == 0 {
            return write!(f, "None");
        }

        write!(f, "{}", self.sum as f64 / self.count as f64)
    }
}

impl Percentable {
    pub fn report_bool(&mut self, value: bool) {
        if value {
            self.sum += 1;
        }
        self.count += 1;
    }

    #[must_use]
    pub fn with_target(target: f64) -> Self {
        Self {
            sum: 0,
            count: 0,
            target,
        }
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn on_target(&self) -> bool {
        match self.count {
            0 => false,
            _ => self.target < (self.sum as f64 / self.count as f64),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Metric {
    metric_type: MetricType,
    aggregate: Aggregate,
}

impl Display for Metric {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.aggregate)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct MetricError {}

// Structure and setup
impl Metric {
    /// Create a `Metric` that has a target based on `Duration`. Supported metrics:
    ///
    /// - `MetricType::AverageWorkTime`
    /// - `MetricType::AverageSpeedAnswer`
    /// - `MetricType::AverageTimeInQueue`
    /// - `MetricType::AverageTimeToAbandon`
    ///
    /// # Errors
    ///
    /// Will error if not using one of the above metrics
    #[allow(clippy::match_wildcard_for_single_variants)]
    pub fn with_target_duration(
        metric_type: MetricType,
        target: Duration,
    ) -> Result<Self, MetricError> {
        match metric_type {
            MetricType::AverageWorkTime
            | MetricType::AverageSpeedAnswer
            | MetricType::AverageTimeInQueue
            | MetricType::AverageTimeToAbandon => Ok(Self {
                metric_type,
                aggregate: Aggregate::Meanable(Meanable::with_target(target)),
            }),
            _ => Err(MetricError {}),
        }
    }

    /// Create a `Metric` that has a target based on `f64`. Supported metrics:
    ///
    /// - `MetricType::ServiceLevel(Duration)`
    /// - `MetricType::AbandonRate`
    ///
    /// In the future this will also support `MetricType::UtilisationTime`
    ///
    /// # Errors
    ///
    /// Will error if not using one of the above metrics
    #[allow(clippy::match_wildcard_for_single_variants)]
    pub fn with_target_f64(metric_type: MetricType, target: f64) -> Result<Self, MetricError> {
        match metric_type {
            MetricType::UtilisationTime | MetricType::ServiceLevel(_) | MetricType::AbandonRate => {
                Ok(Self {
                    metric_type,
                    aggregate: Aggregate::Percentable(Percentable::with_target(target)),
                })
            }
            _ => Err(MetricError {}),
        }
    }

    /// Create a `Metric` that has a target based on `usize`. Supported metrics:
    ///
    /// - `MetricType::AnswerCount`
    ///
    /// # Errors
    ///
    /// Will error if not using one of the above metrics
    pub fn with_target_usize(metric_type: MetricType, target: usize) -> Result<Self, MetricError> {
        match metric_type {
            MetricType::AnswerCount => Ok(Self {
                metric_type,
                aggregate: Aggregate::Countable(Countable::with_target(target)),
            }),
            _ => Err(MetricError {}),
        }
    }

    #[must_use]
    pub fn metric(&self) -> MetricType {
        self.metric_type
    }

    #[must_use]
    pub fn on_target(&self) -> bool {
        match &self.aggregate {
            Aggregate::Meanable(a) => a.on_target(),
            Aggregate::Percentable(a) => a.on_target(),
            Aggregate::Countable(a) => a.on_target(),
        }
    }
}

// Reporting functions
impl Metric {
    /// Report a valuye for Metrics that support `Aggregate::Countable`
    ///
    /// # Panics
    ///
    /// Will panic if attempt to report for other aggregate types
    pub fn report(&mut self) {
        match &mut self.aggregate {
            Aggregate::Countable(a) => a.report(),
            _ => todo!(),
        }
    }

    /// Report a valuye for Metrics that support `Aggregate::Countable`
    ///
    /// # Panics
    ///
    /// Will panic if attempt to report for other aggregate types
    pub fn report_bool(&mut self, value: bool) {
        match &mut self.aggregate {
            Aggregate::Percentable(a) => a.report_bool(value),
            _ => todo!(),
        }
    }

    /// Report a valuye for Metrics that support `Aggregate::Countable`
    ///
    /// # Panics
    ///
    /// Will panic if attempt to report for other aggregate types
    pub fn report_duration(&mut self, value: Duration) {
        match &mut self.aggregate {
            Aggregate::Meanable(a) => a.report_duration(value),
            _ => todo!(),
        }
    }
}
